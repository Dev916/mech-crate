#!/usr/bin/env tsx
/**
 * Portable Markdown to PDF Compiler
 * 
 * A generic tool for converting markdown documents to PDF with:
 * - marked: Markdown to HTML
 * - puppeteer: HTML to PDF (bundles its own Chromium)
 * - mermaid-cli: Diagram rendering
 * - highlight.js: Code syntax highlighting
 * 
 * Optional: Pandoc + XeLaTeX for best quality PDFs
 * 
 * Usage:
 *   npx tsx compile.ts <input.md>           # Single file
 *   npx tsx compile.ts <input-dir/>         # Directory
 *   npx tsx compile.ts --config=<dir>       # Use docs.json from directory
 *   npx tsx compile.ts --list               # List docs from detected config
 *   npx tsx compile.ts --all                # Compile all docs from config
 *   npx tsx compile.ts --doc=<name>         # Compile specific doc from config
 * 
 * Config Detection:
 *   The compiler looks for a docs.json file in:
 *   1. The --config directory (if specified)
 *   2. The input directory (if compiling a folder)
 *   3. Common locations (docs/unyform/, docs/, etc.)
 */

import { readFileSync, writeFileSync, mkdirSync, existsSync, readdirSync, statSync, rmSync } from 'fs';
import { join, dirname, basename, resolve, extname } from 'path';
import { spawn } from 'child_process';
import { fileURLToPath } from 'url';
import { parseArgs } from 'util';
import { cpus } from 'os';
import matter from 'gray-matter';
import { Marked } from 'marked';
import { markedHighlight } from 'marked-highlight';
import hljs from 'highlight.js';

// ═══════════════════════════════════════════════════════════════════════════════
// CONFIGURATION
// ═══════════════════════════════════════════════════════════════════════════════

const __dirname = dirname(fileURLToPath(import.meta.url));
const WORKSPACE_ROOT = resolve(__dirname, '../..');

// docs.json schema
interface DocsConfig {
  name: string;
  description?: string;
  outputDir?: string;
  defaults?: {
    author?: string;
    theme?: string;
  };
  documents: Record<string, {
    file: string;
    title: string;
    description?: string;
  }>;
}

// Cached config
let loadedConfig: { config: DocsConfig; configDir: string } | null = null;

interface Options {
  input?: string;
  output?: string;
  title?: string;
  subtitle?: string;
  author?: string;
  prefix?: string;
  theme: 'dark' | 'light' | 'forest' | 'neutral';
  order?: string[];
  markdownOnly: boolean;
  htmlOnly: boolean;
  verbose: boolean;
  recursive: boolean;
  toc: boolean;
  list: boolean;
  all: boolean;
  doc?: string;
  config?: string;  // Path to docs.json or directory containing it
}

interface DiagramInfo {
  id: string;
  content: string;
  caption?: string;
}

interface DocumentMeta {
  title: string;
  subtitle?: string;
  author?: string;
  date?: string;
  toc: boolean;
  abstract?: string;
}

// ═══════════════════════════════════════════════════════════════════════════════
// ARGUMENT PARSING
// ═══════════════════════════════════════════════════════════════════════════════

function parseArguments(): Options {
  const args = process.argv.slice(2);
  
  if (args.includes('--help') || args.includes('-h')) {
    printHelp();
    process.exit(0);
  }

  const { values, positionals } = parseArgs({
    args,
    options: {
      output: { type: 'string', short: 'o' },
      title: { type: 'string' },
      subtitle: { type: 'string' },
      author: { type: 'string' },
      prefix: { type: 'string' },
      theme: { type: 'string', default: 'light' },
      order: { type: 'string' },
      config: { type: 'string', short: 'c' },
      'markdown-only': { type: 'boolean', default: false },
      'html-only': { type: 'boolean', default: false },
      verbose: { type: 'boolean', short: 'v', default: false },
      'no-recursive': { type: 'boolean', default: false },
      'no-toc': { type: 'boolean', default: false },
      list: { type: 'boolean', default: false },
      all: { type: 'boolean', default: false },
      doc: { type: 'string' },
      help: { type: 'boolean', short: 'h', default: false },
    },
    allowPositionals: true,
  });

  return {
    input: positionals[0],
    output: values.output,
    title: values.title,
    subtitle: values.subtitle,
    author: values.author,
    prefix: values.prefix,
    theme: (values.theme as Options['theme']) || 'dark',
    order: values.order ? values.order.split(',').map(f => f.trim()) : undefined,
    markdownOnly: values['markdown-only'] ?? false,
    htmlOnly: values['html-only'] ?? false,
    verbose: values.verbose ?? false,
    recursive: !(values['no-recursive'] ?? false),
    toc: !(values['no-toc'] ?? false),
    list: values.list ?? false,
    all: values.all ?? false,
    doc: values.doc,
    config: values.config,
  };
}

function printHelp(): void {
  console.log(`
Portable Markdown to PDF Compiler

Usage:
  npx tsx compile.ts <input.md>            # Single file
  npx tsx compile.ts <input-dir/>          # Directory  
  npx tsx compile.ts -c <dir> --list       # List docs from config
  npx tsx compile.ts -c <dir> --all        # Compile all docs from config
  npx tsx compile.ts -c <dir> --doc=<name> # Compile specific doc

Options:
  -o, --output <path>     Output directory for generated files
  -c, --config <dir>      Directory containing docs.json config
  --title <title>         Document title
  --subtitle <subtitle>   Document subtitle
  --author <author>       Document author
  --prefix <string>       Add prefix to output filenames
  --theme <theme>         Mermaid theme: dark, light, forest, neutral
  --order <files>         Comma-separated file order for directories
  --markdown-only         Only generate processed markdown
  --html-only             Only generate HTML, no PDF
  --no-toc                Disable table of contents
  --no-recursive          Don't scan subfolders
  -v, --verbose           Show detailed progress
  -h, --help              Show this help

Config Commands (requires docs.json):
  --list                  List all documents defined in config
  --all                   Compile all documents from config
  --doc=<name>            Compile specific document by key name

Config Detection:
  The compiler looks for docs.json in:
  1. Directory specified by --config
  2. Input directory (if compiling folder)
  3. Common locations: docs/unyform/, docs/, ./

docs.json Format:
  {
    "name": "My Docs",
    "outputDir": "artifacts/mydocs",
    "defaults": { "author": "Team", "theme": "dark" },
    "documents": {
      "readme": { "file": "README.md", "title": "Overview" }
    }
  }

Features:
  • Zero external dependencies for HTML output
  • PDF via Puppeteer (bundled Chromium) or Pandoc
  • Mermaid diagrams rendered as images
  • Syntax highlighting for code blocks
  • YAML frontmatter support for metadata
`);
}

// ═══════════════════════════════════════════════════════════════════════════════
// MERMAID THEMES
// ═══════════════════════════════════════════════════════════════════════════════

const MERMAID_THEMES: Record<string, object> = {
  dark: {
    theme: 'dark',
    themeVariables: {
      primaryColor: '#8b5cf6',
      primaryTextColor: '#f1f5f9',
      primaryBorderColor: '#7c3aed',
      lineColor: '#94a3b8',
      secondaryColor: '#1e293b',
      tertiaryColor: '#0f172a',
      background: '#020617',
      mainBkg: '#0f172a',
      nodeBorder: '#7c3aed',
      clusterBkg: '#1e293b',
      clusterBorder: '#7c3aed',
      titleColor: '#f1f5f9',
      edgeLabelBackground: '#0f172a',
      textColor: '#f1f5f9',
      nodeTextColor: '#f1f5f9',
    },
  },
  light: {
    theme: 'default',
    themeVariables: {
      primaryColor: '#7c3aed',
      primaryTextColor: '#1e293b',
      primaryBorderColor: '#8b5cf6',
      lineColor: '#64748b',
      secondaryColor: '#f1f5f9',
      tertiaryColor: '#e2e8f0',
      background: '#ffffff',
      mainBkg: '#f8fafc',
      nodeBorder: '#8b5cf6',
      clusterBkg: '#f1f5f9',
      clusterBorder: '#8b5cf6',
      titleColor: '#0f172a',
      edgeLabelBackground: '#ffffff',
      textColor: '#1e293b',
      nodeTextColor: '#1e293b',
    },
  },
  forest: { theme: 'forest' },
  neutral: { theme: 'neutral' },
};

// ═══════════════════════════════════════════════════════════════════════════════
// CONFIG DETECTION & LOADING
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Try to find and load docs.json from various locations
 */
function detectConfig(options: Options): { config: DocsConfig; configDir: string } | null {
  // Return cached config if already loaded
  if (loadedConfig) return loadedConfig;
  
  const searchPaths: string[] = [];
  
  // 1. Explicit --config path
  if (options.config) {
    const configPath = resolve(options.config);
    if (existsSync(configPath)) {
      if (statSync(configPath).isDirectory()) {
        searchPaths.push(join(configPath, 'docs.json'));
      } else if (configPath.endsWith('.json')) {
        searchPaths.push(configPath);
      }
    }
  }
  
  // 2. Input directory
  if (options.input) {
    const inputPath = resolve(options.input);
    if (existsSync(inputPath) && statSync(inputPath).isDirectory()) {
      searchPaths.push(join(inputPath, 'docs.json'));
    }
  }
  
  // 3. Common locations relative to workspace
  searchPaths.push(
    join(WORKSPACE_ROOT, 'docs/unyform/docs.json'),
    join(WORKSPACE_ROOT, 'docs/docs.json'),
    join(WORKSPACE_ROOT, 'docs.json'),
    join(process.cwd(), 'docs.json'),
  );
  
  // Try each path
  for (const configPath of searchPaths) {
    if (existsSync(configPath)) {
      try {
        const content = readFileSync(configPath, 'utf-8');
        const config = JSON.parse(content) as DocsConfig;
        const configDir = dirname(configPath);
        loadedConfig = { config, configDir };
        return loadedConfig;
      } catch (err) {
        // Invalid JSON, try next
        continue;
      }
    }
  }
  
  return null;
}

/**
 * Require config to be present, exit with error if not found
 */
function requireConfig(options: Options): { config: DocsConfig; configDir: string } {
  const result = detectConfig(options);
  if (!result) {
    console.error('❌ No docs.json config found.');
    console.error('');
    console.error('   Searched locations:');
    if (options.config) {
      console.error(`   • ${resolve(options.config)}`);
    }
    console.error(`   • docs/unyform/docs.json`);
    console.error(`   • docs/docs.json`);
    console.error(`   • ./docs.json`);
    console.error('');
    console.error('   Create a docs.json or specify --config=<path>');
    process.exit(1);
  }
  return result;
}

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════════════════

async function main() {
  const options = parseArguments();

  // Handle --list (requires config)
  if (options.list) {
    listConfigDocs(options);
    return;
  }

  // Handle --all (compile all docs from config)
  if (options.all) {
    await compileAllConfigDocs(options);
    return;
  }

  // Handle --doc=<name> (compile specific doc from config)
  if (options.doc) {
    await compileConfigDoc(options.doc, options);
    return;
  }

  // Handle input (positional arg)
  if (options.input) {
    const inputPath = resolve(options.input);
    if (!existsSync(inputPath)) {
      console.error(`❌ Input not found: ${inputPath}`);
      process.exit(1);
    }
    
    if (statSync(inputPath).isDirectory()) {
      await compileFolder(inputPath, options);
    } else {
      await compileFile(inputPath, options);
    }
    return;
  }

  // No action specified
  printHelp();
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONFIG-BASED DOCUMENT HANDLING
// ═══════════════════════════════════════════════════════════════════════════════

function listConfigDocs(options: Options): void {
  const { config, configDir } = requireConfig(options);
  
  console.log('');
  console.log(`📚 ${config.name || 'Available documents'}:`);
  if (config.description) {
    console.log(`   ${config.description}`);
  }
  console.log(`   Config: ${configDir}/docs.json`);
  console.log('');
  
  for (const [key, doc] of Object.entries(config.documents)) {
    const filePath = join(configDir, doc.file);
    const exists = existsSync(filePath);
    const status = exists ? '✅' : '❌';
    console.log(`  ${status} ${key.padEnd(22)} ${doc.title}`);
    if (doc.description) {
      console.log(`     ${doc.description}`);
    }
    if (!exists) {
      console.log(`     (file not found: ${doc.file})`);
    }
    console.log('');
  }
}

async function compileAllConfigDocs(options: Options): Promise<void> {
  const { config, configDir } = requireConfig(options);
  
  console.log('');
  console.log(`${config.name || 'Docs'} - Compiling all documents`);
  console.log('════════════════════════════════════════════════════════════════');
  
  // Determine output directory
  const outputDir = options.output || (
    config.outputDir 
      ? join(WORKSPACE_ROOT, config.outputDir)
      : join(configDir, 'output')
  );
  mkdirSync(outputDir, { recursive: true });
  
  // Apply defaults from config
  const mergedOptions = {
    ...options,
    author: options.author || config.defaults?.author,
    theme: options.theme || (config.defaults?.theme as Options['theme']) || 'dark',
  };
  
  let compiled = 0;
  let skipped = 0;
  
  for (const [key, doc] of Object.entries(config.documents)) {
    const filePath = join(configDir, doc.file);
    if (!existsSync(filePath)) {
      console.log(`   ⏭️  Skipping ${key} (file not found)`);
      skipped++;
      continue;
    }
    
    console.log(`\n📄 Compiling ${key}...`);
    await compileFile(filePath, { ...mergedOptions, output: outputDir });
    compiled++;
  }
  
  console.log('');
  console.log('════════════════════════════════════════════════════════════════');
  console.log(`✅ Compiled ${compiled} documents, skipped ${skipped}`);
  console.log(`   Output: ${outputDir}`);
}

async function compileConfigDoc(name: string, options: Options): Promise<void> {
  const { config, configDir } = requireConfig(options);
  
  const doc = config.documents[name];
  if (!doc) {
    console.error(`❌ Unknown document: ${name}`);
    console.error(`   Available: ${Object.keys(config.documents).join(', ')}`);
    process.exit(1);
  }
  
  const filePath = join(configDir, doc.file);
  if (!existsSync(filePath)) {
    console.error(`❌ Document file not found: ${doc.file}`);
    process.exit(1);
  }
  
  // Apply defaults from config
  const mergedOptions = {
    ...options,
    author: options.author || config.defaults?.author,
    theme: options.theme || (config.defaults?.theme as Options['theme']) || 'dark',
    output: options.output || (
      config.outputDir 
        ? join(WORKSPACE_ROOT, config.outputDir)
        : undefined
    ),
  };
  
  await compileFile(filePath, mergedOptions);
}

// ═══════════════════════════════════════════════════════════════════════════════
// FILE/FOLDER COMPILATION
// ═══════════════════════════════════════════════════════════════════════════════

async function compileFile(filePath: string, options: Options): Promise<void> {
  const absolutePath = resolve(filePath);
  
  if (!existsSync(absolutePath)) {
    console.error(`❌ File not found: ${absolutePath}`);
    process.exit(1);
  }
  
  const fileName = basename(absolutePath, '.md');
  const fileDir = dirname(absolutePath);
  const parentFolderName = basename(fileDir); // e.g., "unyform" from docs/unyform/
  
  // Determine output directory: artifacts/<parent-folder>/<filename>/
  // e.g., docs/unyform/WHITEPAPER.md -> artifacts/unyform/WHITEPAPER/
  const outputDir = options.output 
    ? resolve(options.output)
    : join(WORKSPACE_ROOT, 'artifacts', parentFolderName, fileName);
  
  const diagramsDir = join(outputDir, 'diagrams');
  
  // Create directories
  mkdirSync(outputDir, { recursive: true });
  if (existsSync(diagramsDir)) {
    rmSync(diagramsDir, { recursive: true });
  }
  mkdirSync(diagramsDir, { recursive: true });
  
  console.log('');
  console.log('unyform.ai docs - Markdown to PDF Compiler');
  console.log('════════════════════════════════════════════════════════════════');
  console.log(`   📥 Input:  ${absolutePath}`);
  console.log(`   📤 Output: ${outputDir}`);
  console.log(`   🎨 Theme:  ${options.theme}`);
  console.log('');
  
  // Read and parse file
  const content = readFileSync(absolutePath, 'utf-8');
  const { data: frontmatter, content: markdownBody } = matter(content);
  
  // Extract metadata (command line overrides frontmatter)
  const meta: DocumentMeta = {
    title: options.title || frontmatter.title || extractTitle(markdownBody) || fileName,
    subtitle: options.subtitle || frontmatter.subtitle,
    author: options.author || frontmatter.author || 'MechCrate',
    date: frontmatter.date || new Date().toLocaleDateString('en-US', { 
      year: 'numeric', month: 'long', day: 'numeric' 
    }),
    toc: options.toc && frontmatter.toc !== false,
    abstract: frontmatter.abstract,
  };
  
  console.log(`   📋 Title: ${meta.title}`);
  
  // Process diagrams
  const diagrams: DiagramInfo[] = [];
  let diagramCounter = 0;
  
  const processedMarkdown = markdownBody.replace(/```mermaid\n([\s\S]*?)```/g, (match, mermaidContent) => {
    diagramCounter++;
    const diagramId = `diagram-${diagramCounter}`;
    diagrams.push({
      id: diagramId,
      content: mermaidContent.trim(),
      caption: extractDiagramCaption(markdownBody, match),
    });
    const caption = diagrams[diagrams.length - 1].caption;
    const captionText = caption ? `\n*${caption}*` : '';
    return `\n![${caption || 'Diagram'}](diagrams/${diagramId}.png)${captionText}\n`;
  });
  
  console.log(`   📊 Found ${diagrams.length} Mermaid diagrams`);
  
  // Render diagrams
  if (diagrams.length > 0) {
    console.log('');
    console.log('🎨 Rendering Mermaid diagrams...');
    await renderMermaidDiagrams(diagrams, diagramsDir, options.theme, options.verbose);
  }
  
  // Convert markdown to HTML
  console.log('');
  console.log('📝 Converting to HTML...');
  
  const marked = new Marked(
    markedHighlight({
      langPrefix: 'hljs language-',
      highlight(code, lang) {
        const language = hljs.getLanguage(lang) ? lang : 'plaintext';
        return hljs.highlight(code, { language }).value;
      }
    })
  );
  
  // Add IDs to headings for TOC links
  marked.use({
    renderer: {
      heading(text: string, level: number) {
        const slug = text.toLowerCase().replace(/[^\w]+/g, '-');
        return `<h${level} id="${slug}">${text}</h${level}>`;
      }
    }
  });
  
  const htmlBody = await marked.parse(processedMarkdown);
  const fullHtml = generateHtmlDocument(htmlBody, meta, outputDir);
  
  // Write HTML
  const outputPrefix = options.prefix ? `${options.prefix}-` : '';
  const htmlPath = join(outputDir, `${outputPrefix}${fileName}.html`);
  writeFileSync(htmlPath, fullHtml);
  console.log(`   ✅ HTML: ${basename(htmlPath)}`);
  
  // Write processed markdown
  const mdPath = join(outputDir, `${outputPrefix}${fileName}.md`);
  const finalMd = generateFinalMarkdown(processedMarkdown, meta);
  writeFileSync(mdPath, finalMd);
  console.log(`   ✅ Markdown: ${basename(mdPath)}`);
  
  if (options.markdownOnly) {
    console.log('');
    console.log('════════════════════════════════════════════════════════════════');
    console.log('✅ Markdown generation complete!');
    console.log(`   📄 Markdown: ${mdPath}`);
    return;
  }
  
  if (options.htmlOnly) {
    console.log('');
    console.log('════════════════════════════════════════════════════════════════');
    console.log('✅ HTML generation complete!');
    console.log(`   🌐 HTML: ${htmlPath}`);
    return;
  }
  
  // Generate PDF using Puppeteer (portable, always works)
  console.log('');
  console.log('📕 Generating PDF...');
  
  const pdfPath = join(outputDir, `${outputPrefix}${fileName}.pdf`);
  await generatePdfWithPuppeteer(htmlPath, pdfPath, meta, options.verbose);
  
  console.log('');
  console.log('════════════════════════════════════════════════════════════════');
  console.log('✅ Generation complete!');
  console.log(`   📄 Markdown: ${mdPath}`);
  console.log(`   🌐 HTML: ${htmlPath}`);
  console.log(`   📕 PDF: ${pdfPath}`);
}

async function compileFolder(folderPath: string, options: Options): Promise<void> {
  const absolutePath = resolve(folderPath);
  
  if (!existsSync(absolutePath) || !statSync(absolutePath).isDirectory()) {
    console.error(`❌ Folder not found: ${absolutePath}`);
    process.exit(1);
  }
  
  console.log('');
  console.log('unyform.ai docs - Compiling folder');
  console.log('════════════════════════════════════════════════════════════════');
  console.log(`   📁 Folder: ${absolutePath}`);
  
  // Find all markdown files
  const files = findMarkdownFiles(absolutePath, options.recursive, options.order);
  
  if (files.length === 0) {
    console.log('   ⚠️  No markdown files found');
    return;
  }
  
  console.log(`   📚 Found ${files.length} markdown files`);
  
  for (const file of files) {
    console.log(`\n${'─'.repeat(60)}`);
    await compileFile(file, options);
  }
  
  console.log('');
  console.log('════════════════════════════════════════════════════════════════');
  console.log(`✅ Compiled ${files.length} files`);
}

function findMarkdownFiles(dir: string, recursive: boolean, order?: string[]): string[] {
  const entries = readdirSync(dir, { withFileTypes: true });
  
  // Get .md files in this directory
  let mdFiles = entries
    .filter(e => e.isFile() && extname(e.name).toLowerCase() === '.md' && !e.name.startsWith('_'))
    .map(e => e.name);
  
  // Sort files
  if (order && order.length > 0) {
    const orderedSet = new Set(order);
    const ordered = order.filter(f => mdFiles.includes(f));
    const remaining = mdFiles.filter(f => !orderedSet.has(f)).sort();
    mdFiles = [...ordered, ...remaining];
  } else {
    // Default: README first, then numbered, then alphabetically
    mdFiles.sort((a, b) => {
      if (a.toLowerCase() === 'readme.md') return -1;
      if (b.toLowerCase() === 'readme.md') return 1;
      const aNum = a.match(/^(\d+)/);
      const bNum = b.match(/^(\d+)/);
      if (aNum && bNum) return parseInt(aNum[1]) - parseInt(bNum[1]);
      if (aNum) return -1;
      if (bNum) return 1;
      return a.localeCompare(b);
    });
  }
  
  const files = mdFiles.map(f => join(dir, f));
  
  // Recurse into subdirectories
  if (recursive) {
    for (const entry of entries) {
      if (entry.isDirectory() && !entry.name.startsWith('.') && entry.name !== 'node_modules' && entry.name !== 'output') {
        files.push(...findMarkdownFiles(join(dir, entry.name), recursive, undefined));
      }
    }
  }
  
  return files;
}

// ═══════════════════════════════════════════════════════════════════════════════
// MERMAID RENDERING
// ═══════════════════════════════════════════════════════════════════════════════

async function renderMermaidDiagrams(diagrams: DiagramInfo[], diagramsDir: string, theme: string, verbose: boolean): Promise<void> {
  const mermaidConfig = {
    ...MERMAID_THEMES[theme] || MERMAID_THEMES.dark,
    flowchart: { curve: 'basis', padding: 20 },
    sequence: { actorMargin: 50, boxMargin: 10, boxTextMargin: 5, noteMargin: 10, messageMargin: 35 },
    er: { fontSize: 14 },
    stateDiagram: { fontSize: 14 },
  };
  
  const configPath = join(dirname(diagramsDir), 'mermaid-config.json');
  writeFileSync(configPath, JSON.stringify(mermaidConfig, null, 2));
  
  const numCores = cpus().length;
  const concurrency = Math.max(1, Math.min(numCores - 1, 4));
  
  if (verbose) {
    console.log(`   🚀 Parallel rendering with ${concurrency} workers`);
  }
  
  let successCount = 0;
  let failCount = 0;
  
  // Process in batches
  for (let i = 0; i < diagrams.length; i += concurrency) {
    const batch = diagrams.slice(i, i + concurrency);
    const promises = batch.map(diagram => renderSingleDiagram(diagram, diagramsDir, configPath));
    const results = await Promise.all(promises);
    
    for (const result of results) {
      if (result.success) {
        if (verbose) console.log(`   ✅ ${result.id}`);
        successCount++;
      } else {
        console.log(`   ⚠️  ${result.id} (${result.error || 'failed'})`);
        failCount++;
      }
    }
  }
  
  console.log(`   📊 Rendered: ${successCount} success, ${failCount} failed`);
}

function renderSingleDiagram(
  diagram: DiagramInfo,
  diagramsDir: string,
  configPath: string
): Promise<{ id: string; success: boolean; error?: string }> {
  return new Promise((resolve) => {
    const mmdPath = join(diagramsDir, `${diagram.id}.mmd`);
    const pngPath = join(diagramsDir, `${diagram.id}.png`);
    
    writeFileSync(mmdPath, diagram.content);
    
    const child = spawn('npx', [
      'mmdc',
      '-i', mmdPath,
      '-o', pngPath,
      '-c', configPath,
      '-b', 'transparent',
      '-w', '1400',
      '-s', '2',
    ], {
      cwd: __dirname,
      stdio: ['pipe', 'pipe', 'pipe'],
    });

    let stderr = '';
    child.stderr?.on('data', (data) => {
      stderr += data.toString();
    });

    const timeout = setTimeout(() => {
      child.kill();
      resolve({ id: diagram.id, success: false, error: 'Timeout' });
    }, 60000);

    child.on('close', (code) => {
      clearTimeout(timeout);
      const success = code === 0 && existsSync(pngPath);
      resolve({ id: diagram.id, success, error: success ? undefined : stderr.split('\n')[0] });
    });

    child.on('error', (err) => {
      clearTimeout(timeout);
      resolve({ id: diagram.id, success: false, error: err.message });
    });
  });
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKDOWN GENERATION  
// ═══════════════════════════════════════════════════════════════════════════════

function generateFinalMarkdown(content: string, meta: DocumentMeta): string {
  // Build YAML header like the reference implementation
  let header = `---
title: "${meta.title}"
`;
  if (meta.subtitle) header += `subtitle: "${meta.subtitle}"\n`;
  header += `author: "${meta.author}"
date: "${meta.date}"
`;
  if (meta.abstract) header += `abstract: |\n  ${meta.abstract}\n`;
  header += `toc: ${meta.toc}
toc-depth: 3
numbersections: true
colorlinks: true
linkcolor: blue
urlcolor: blue
geometry: margin=1in
fontsize: 11pt
documentclass: report
---

`;
  
  // Remove original title if present (it's in metadata now)
  let body = content;
  body = body.replace(/^#\s+[^\n]+\n/, '');
  body = body.replace(/^>\s*[^\n]+\n\n/, ''); // Remove blockquote subtitle
  
  return header + body;
}

// ═══════════════════════════════════════════════════════════════════════════════
// HTML GENERATION
// ═══════════════════════════════════════════════════════════════════════════════

function generateHtmlDocument(body: string, meta: DocumentMeta, outputDir: string): string {
  const toc = meta.toc ? generateToc(body) : '';
  
  // Unyform logo SVG (corner variant) embedded inline
  const unyformLogoSvg = `
    <svg width="180" height="180" viewBox="0 0 512 512" xmlns="http://www.w3.org/2000/svg">
      <g transform="translate(96, 72)">
        <!-- TOP BAR - RIGHT HALF (navy) -->
        <line x1="160" y1="0" x2="320" y2="0" stroke="#1e1b4b" stroke-width="24" stroke-linecap="round"/>
        <!-- TOP BAR - LEFT HALF (accent) -->
        <line x1="0" y1="0" x2="160" y2="0" stroke="#6366f1" stroke-width="24" stroke-linecap="round"/>
        <!-- RIGHT SIDE + BOTTOM CURVE (navy) -->
        <path 
          d="M 320 0 L 320 240 Q 320 368 160 368 Q 0 368 0 240"
          fill="none"
          stroke="#1e1b4b"
          stroke-width="24"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
        <!-- LEFT VERTICAL (accent) -->
        <line x1="0" y1="0" x2="0" y2="240" stroke="#6366f1" stroke-width="24" stroke-linecap="round"/>
        <!-- HORIZONTAL CROSSBAR (accent) -->
        <line x1="0" y1="120" x2="112" y2="120" stroke="#6366f1" stroke-width="16" stroke-linecap="round"/>
        <!-- CORNER REINFORCEMENT -->
        <line x1="296" y1="24" x2="296" y2="48" stroke="#1e1b4b" stroke-width="5" stroke-linecap="round"/>
        <line x1="280" y1="24" x2="296" y2="24" stroke="#1e1b4b" stroke-width="5" stroke-linecap="round"/>
      </g>
    </svg>
  `;
  
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${escapeHtml(meta.title)}</title>
  <style>
${getStyles()}
  </style>
</head>
<body>
  <div class="cover-page">
    <div class="logo">${unyformLogoSvg}</div>
    <p class="company-name">unyform.ai</p>
    <h1 class="title">${escapeHtml(meta.title)}</h1>
    ${meta.subtitle ? `<p class="subtitle">${escapeHtml(meta.subtitle)}</p>` : ''}
    <p class="author">${escapeHtml(meta.author || '')}</p>
    <p class="date">${escapeHtml(meta.date || '')}</p>
    ${meta.abstract ? `<div class="abstract"><p>${escapeHtml(meta.abstract)}</p></div>` : ''}
  </div>
  
  ${toc ? `<div class="toc-page"><h2>Table of Contents</h2>${toc}</div>` : ''}
  
  <div class="content">
${body}
  </div>
</body>
</html>`;
}

function generateToc(html: string): string {
  const headingRegex = /<h([2-4])[^>]*id="([^"]*)"[^>]*>([^<]*)<\/h[2-4]>/gi;
  const headings: { level: number; id: string; text: string }[] = [];
  
  let match;
  while ((match = headingRegex.exec(html)) !== null) {
    headings.push({
      level: parseInt(match[1]),
      id: match[2],
      text: match[3].replace(/<[^>]+>/g, ''),
    });
  }
  
  if (headings.length === 0) return '';
  
  let toc = '<nav class="toc"><ul>';
  let currentLevel = 2;
  
  for (const h of headings) {
    while (h.level > currentLevel) {
      toc += '<ul>';
      currentLevel++;
    }
    while (h.level < currentLevel) {
      toc += '</ul></li>';
      currentLevel--;
    }
    toc += `<li><a href="#${h.id}">${escapeHtml(h.text)}</a>`;
  }
  
  while (currentLevel > 2) {
    toc += '</ul></li>';
    currentLevel--;
  }
  toc += '</ul></nav>';
  
  return toc;
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

// ═══════════════════════════════════════════════════════════════════════════════
// PDF GENERATION - PUPPETEER (PORTABLE, NO EXTERNAL DEPS)
// ═══════════════════════════════════════════════════════════════════════════════

async function generatePdfWithPuppeteer(htmlPath: string, pdfPath: string, meta: DocumentMeta, verbose: boolean): Promise<void> {
  try {
    // Dynamic import to handle puppeteer
    const puppeteer = await import('puppeteer');
    
    if (verbose) console.log('   🚀 Launching Puppeteer browser...');
    
    const browser = await puppeteer.default.launch({
      headless: true,
      args: ['--no-sandbox', '--disable-setuid-sandbox'],
    });
    
    const page = await browser.newPage();
    
    // Load the HTML file
    const htmlUrl = `file://${htmlPath}`;
    await page.goto(htmlUrl, { waitUntil: 'networkidle0' });
    
    if (verbose) console.log('   📄 Generating PDF...');
    
    // Generate PDF
    await page.pdf({
      path: pdfPath,
      format: 'Letter',
      margin: {
        top: '1in',
        right: '1in',
        bottom: '1in',
        left: '1in',
      },
      printBackground: true,
      displayHeaderFooter: true,
      headerTemplate: `
        <div style="font-size: 10px; color: #64748b; width: 100%; text-align: center; padding: 0 1in;">
          <span>${escapeHtml(meta.title)}</span>
        </div>
      `,
      footerTemplate: `
        <div style="font-size: 10px; color: #64748b; width: 100%; text-align: center; padding: 0 1in;">
          <span class="pageNumber"></span> / <span class="totalPages"></span>
        </div>
      `,
    });
    
    await browser.close();
    
    console.log(`   ✅ PDF generated with Puppeteer`);
  } catch (error) {
    console.error(`   ❌ PDF generation failed: ${(error as Error).message}`);
    console.log('   📄 HTML file is available for manual conversion');
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UTILITIES
// ═══════════════════════════════════════════════════════════════════════════════

function extractTitle(content: string): string | undefined {
  const match = content.match(/^#\s+(.+)$/m);
  return match ? match[1].replace(/[*_`]/g, '').trim() : undefined;
}

function extractDiagramCaption(content: string, match: string): string | undefined {
  const matchIndex = content.indexOf(match);
  if (matchIndex === -1) return undefined;
  
  const textBefore = content.substring(Math.max(0, matchIndex - 200), matchIndex);
  
  const patterns = [
    /\*\*([^*]+)\*\*\s*$/,
    /####?\s+(.+?)\s*$/,
    /(?:Figure|Diagram)(?:\s+\d+)?:?\s*(.+?)(?:\n|$)/i,
  ];
  
  for (const pattern of patterns) {
    const captionMatch = textBefore.match(pattern);
    if (captionMatch) {
      return captionMatch[1].trim();
    }
  }
  
  return undefined;
}

function getStyles(): string {
  return `
    @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap');
    
    * {
      box-sizing: border-box;
    }
    
    :root {
      --mx-primary: #8b5cf6;
      --mx-primary-dark: #7c3aed;
      --mx-green: #22c55e;
      --mx-dark: #0f172a;
      --mx-gray: #64748b;
      --mx-light-gray: #f1f5f9;
      --code-bg: #1e293b;
      --link-blue: #3b82f6;
    }
    
    body {
      font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      font-size: 11pt;
      line-height: 1.6;
      color: #1a1a1a;
      margin: 0;
      padding: 0;
    }
    
    .cover-page {
      page-break-after: always;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      min-height: 90vh;
      text-align: center;
      padding: 2rem;
    }
    
    .logo {
      margin-bottom: 1.5rem;
    }
    
    .logo svg {
      width: 180px;
      height: 180px;
    }
    
    .company-name {
      font-size: 28pt;
      font-weight: 700;
      color: #1e1b4b;
      margin: 0 0 0.5rem 0;
      letter-spacing: -0.02em;
    }
    
    .title {
      font-size: 32pt;
      font-weight: 700;
      color: var(--mx-primary);
      margin: 0 0 1rem 0;
    }
    
    .subtitle {
      font-size: 18pt;
      color: var(--mx-gray);
      margin: 0 0 2rem 0;
    }
    
    .author {
      font-size: 14pt;
      color: #333;
      margin: 0 0 0.5rem 0;
    }
    
    .date {
      font-size: 12pt;
      color: var(--mx-gray);
      margin: 0 0 2rem 0;
    }
    
    .abstract {
      max-width: 80%;
      font-style: italic;
      color: var(--mx-gray);
      border-top: 1px solid #e5e7eb;
      padding-top: 2rem;
      margin-top: 2rem;
    }
    
    .toc-page {
      page-break-after: always;
      padding: 2rem;
    }
    
    .toc-page h2 {
      color: var(--mx-primary);
      border-bottom: 2px solid var(--mx-primary);
      padding-bottom: 0.5rem;
    }
    
    .toc ul {
      list-style: none;
      padding-left: 1.5rem;
    }
    
    .toc > ul {
      padding-left: 0;
    }
    
    .toc li {
      margin: 0.5rem 0;
    }
    
    .toc a {
      color: #333;
      text-decoration: none;
    }
    
    .toc a:hover {
      color: var(--mx-primary);
    }
    
    .content {
      padding: 0 2rem;
    }
    
    h1, h2, h3, h4, h5, h6 {
      color: var(--mx-primary-dark);
      margin-top: 2rem;
      margin-bottom: 1rem;
      page-break-after: avoid;
    }
    
    h1 { font-size: 24pt; border-bottom: 2px solid var(--mx-primary); padding-bottom: 0.5rem; }
    h2 { font-size: 18pt; border-bottom: 1px solid #e5e7eb; padding-bottom: 0.25rem; }
    h3 { font-size: 14pt; }
    h4 { font-size: 12pt; }
    
    a {
      color: var(--link-blue);
      text-decoration: none;
    }
    
    a:hover {
      text-decoration: underline;
    }
    
    code {
      font-family: 'JetBrains Mono', 'SF Mono', Menlo, monospace;
      font-size: 0.9em;
      background: #f3f4f6;
      padding: 0.1em 0.3em;
      border-radius: 3px;
    }
    
    pre {
      background: var(--code-bg);
      color: #e5e7eb;
      padding: 1rem;
      border-radius: 8px;
      overflow-x: auto;
      font-size: 9pt;
      line-height: 1.5;
      page-break-inside: avoid;
      margin: 1rem 0;
    }
    
    pre code {
      background: none;
      padding: 0;
      color: inherit;
    }
    
    table {
      width: 100%;
      border-collapse: collapse;
      margin: 1rem 0;
      font-size: 10pt;
      page-break-inside: avoid;
    }
    
    th, td {
      padding: 0.5rem 0.75rem;
      text-align: left;
      border-bottom: 1px solid #e5e7eb;
    }
    
    th {
      background: var(--mx-primary);
      color: white;
      font-weight: 600;
    }
    
    tr:nth-child(even) {
      background: #f9fafb;
    }
    
    blockquote {
      border-left: 4px solid var(--mx-primary);
      margin: 1rem 0;
      padding: 0.5rem 1rem;
      background: #f3f4f6;
      font-style: italic;
    }
    
    blockquote p {
      margin: 0;
    }
    
    ul, ol {
      margin: 0.5rem 0;
      padding-left: 1.5rem;
    }
    
    li {
      margin: 0.25rem 0;
    }
    
    hr {
      border: none;
      border-top: 2px solid var(--mx-primary);
      margin: 2rem 0;
    }
    
    img {
      max-width: 100%;
      height: auto;
      display: block;
      margin: 1.5rem auto;
      border-radius: 8px;
    }
    
    /* Highlight.js theme adjustments */
    .hljs-keyword { color: #c792ea; }
    .hljs-string { color: #c3e88d; }
    .hljs-number { color: #f78c6c; }
    .hljs-comment { color: #676e95; font-style: italic; }
    .hljs-function { color: #82aaff; }
    .hljs-class { color: #ffcb6b; }
    .hljs-variable { color: #f07178; }
    .hljs-attr { color: #ffcb6b; }
    
    @media print {
      .cover-page { min-height: 100vh; }
      body { font-size: 10pt; }
      h1 { page-break-before: always; }
      h2, h3 { page-break-after: avoid; }
      pre, table, img { page-break-inside: avoid; }
    }
  `;
}

// ═══════════════════════════════════════════════════════════════════════════════
// RUN
// ═══════════════════════════════════════════════════════════════════════════════

main().catch((err) => {
  console.error('❌ Compilation failed:', err.message);
  process.exit(1);
});
