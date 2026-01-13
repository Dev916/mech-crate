#!/usr/bin/env tsx
/**
 * MechCrate md2pdf - Markdown to PDF Compiler
 * 
 * Converts markdown files to professional PDFs with:
 * - Mermaid diagram rendering (as PNG images)
 * - Table of contents
 * - Professional styling
 * - Syntax highlighting
 * 
 * Usage:
 *   npx tsx md2pdf.ts <input.md> [options]
 *   npx tsx md2pdf.ts <input-dir/> [options]
 * 
 * Options:
 *   --output, -o       Output PDF path (default: <name>-output/<name>.pdf)
 *   --markdown-only    Only generate processed markdown, skip PDF
 *   --html-only        Only generate HTML, skip PDF
 *   --title            Document title (default: extracted from first H1)
 *   --subtitle         Document subtitle
 *   --author           Document author (default: "MechCrate")
 *   --no-toc           Disable table of contents
 *   --no-numbers       Disable section numbering
 *   --theme            Mermaid theme: dark, light, forest, neutral (default: dark)
 *   --order            Comma-separated list of files for directory mode
 *   --help, -h         Show help
 */

import { readFileSync, writeFileSync, mkdirSync, existsSync, copyFileSync, rmSync, readdirSync, statSync } from 'fs';
import { join, dirname, basename, resolve, extname } from 'path';
import { spawn, spawnSync } from 'child_process';
import { fileURLToPath } from 'url';
import { parseArgs } from 'util';
import { cpus } from 'os';

// ═══════════════════════════════════════════════════════════════════════════════
// CONFIGURATION
// ═══════════════════════════════════════════════════════════════════════════════

const __dirname = dirname(fileURLToPath(import.meta.url));

interface Options {
  input: string;
  output: string;
  outputDir: string;
  markdownOnly: boolean;
  htmlOnly: boolean;
  title?: string;
  subtitle?: string;
  author: string;
  toc: boolean;
  numbers: boolean;
  theme: 'dark' | 'light' | 'forest' | 'neutral';
  isDirectory: boolean;
  fileOrder?: string[];
}

interface DiagramInfo {
  id: string;
  type: 'mermaid';
  content: string;
  caption?: string;
}

// ═══════════════════════════════════════════════════════════════════════════════
// ARGUMENT PARSING
// ═══════════════════════════════════════════════════════════════════════════════

function printHelp(): void {
  console.log(`
🦝 MechCrate md2pdf - Markdown to PDF Compiler

Usage:
  npx tsx md2pdf.ts <input.md> [options]      # Single file
  npx tsx md2pdf.ts <input-dir/> [options]    # Directory of files

Examples:
  npx tsx md2pdf.ts docs/README.md
  npx tsx md2pdf.ts docs/guide/ --title "User Guide"
  npx tsx md2pdf.ts docs/spec.md --output artifacts/spec.pdf
  npx tsx md2pdf.ts docs/api/ --order "overview.md,endpoints.md"

Options:
  --output, -o       Output PDF path (default: <name>-output/<name>.pdf)
  --markdown-only    Only generate processed markdown, skip PDF
  --html-only        Only generate HTML, skip PDF
  --title            Document title (default: extracted from first H1)
  --subtitle         Document subtitle
  --author           Document author (default: "MechCrate")
  --no-toc           Disable table of contents
  --no-numbers       Disable section numbering  
  --theme            Mermaid theme: dark, light, forest, neutral (default: dark)
  --order            File order for directories (comma-separated)
  --help, -h         Show this help

Output Structure:
  <name>-output/
  ├── <name>.pdf           # Final PDF
  ├── <name>.md            # Processed markdown
  ├── <name>.html          # HTML (if applicable)
  └── diagrams/            # Rendered Mermaid PNGs

Directory Mode:
  When given a directory, all .md files are combined into one PDF.
  Files are sorted alphabetically unless --order is specified.

Features:
  • Renders all Mermaid diagrams as high-resolution PNG images
  • Generates table of contents with configurable depth
  • Syntax highlighting for code blocks
  • Professional PDF styling via Pandoc + LaTeX/WeasyPrint
  • Preserves markdown tables and formatting
`);
}

function parseArguments(): Options {
  const args = process.argv.slice(2);
  
  if (args.includes('--help') || args.includes('-h') || args.length === 0) {
    printHelp();
    process.exit(0);
  }

  const { values, positionals } = parseArgs({
    args,
    options: {
      output: { type: 'string', short: 'o' },
      'markdown-only': { type: 'boolean', default: false },
      'html-only': { type: 'boolean', default: false },
      title: { type: 'string' },
      subtitle: { type: 'string' },
      author: { type: 'string', default: 'MechCrate' },
      'no-toc': { type: 'boolean', default: false },
      'no-numbers': { type: 'boolean', default: false },
      theme: { type: 'string', default: 'dark' },
      order: { type: 'string' },
      help: { type: 'boolean', short: 'h', default: false },
    },
    allowPositionals: true,
  });

  if (positionals.length === 0) {
    console.error('❌ Error: No input file or directory specified');
    console.error('   Usage: npx tsx md2pdf.ts <input.md | input-dir/>');
    process.exit(1);
  }

  const input = resolve(positionals[0]);
  const isDirectory = existsSync(input) && statSync(input).isDirectory();
  const inputName = isDirectory ? basename(input) : basename(input, '.md');
  
  let output: string;
  let outputDir: string;
  
  if (values.output) {
    output = resolve(values.output);
    const outputName = basename(output, '.pdf');
    outputDir = join(dirname(output), `${outputName}-output`);
  } else {
    // Create output directory next to input
    const inputDir = isDirectory ? dirname(input) : dirname(input);
    outputDir = join(inputDir, `${inputName}-output`);
    output = join(outputDir, `${inputName}.pdf`);
  }

  const fileOrder = values.order 
    ? values.order.split(',').map(f => f.trim())
    : undefined;

  return {
    input,
    output,
    outputDir,
    markdownOnly: values['markdown-only'] ?? false,
    htmlOnly: values['html-only'] ?? false,
    title: values.title,
    subtitle: values.subtitle,
    author: values.author ?? 'MechCrate',
    toc: !(values['no-toc'] ?? false),
    numbers: !(values['no-numbers'] ?? false),
    theme: (values.theme as Options['theme']) ?? 'dark',
    isDirectory,
    fileOrder,
  };
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
  forest: {
    theme: 'forest',
  },
  neutral: {
    theme: 'neutral',
  },
};

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════════════════

let diagramCounter = 0;
const allDiagrams: DiagramInfo[] = [];

// ═══════════════════════════════════════════════════════════════════════════════
// DIRECTORY HANDLING
// ═══════════════════════════════════════════════════════════════════════════════

interface MarkdownFile {
  path: string;
  name: string;
  content: string;
}

function getMarkdownFiles(dirPath: string, fileOrder?: string[]): MarkdownFile[] {
  const files: MarkdownFile[] = [];
  
  const entries = readdirSync(dirPath)
    .filter(f => extname(f).toLowerCase() === '.md')
    .filter(f => !f.startsWith('_'));
  
  let sortedFiles: string[];
  
  if (fileOrder && fileOrder.length > 0) {
    const orderedSet = new Set(fileOrder);
    const ordered = fileOrder.filter(f => entries.includes(f));
    const remaining = entries.filter(f => !orderedSet.has(f)).sort();
    sortedFiles = [...ordered, ...remaining];
  } else {
    sortedFiles = entries.sort((a, b) => {
      if (a.toLowerCase() === 'readme.md') return -1;
      if (b.toLowerCase() === 'readme.md') return 1;
      const aNum = a.match(/^(\d+)/);
      const bNum = b.match(/^(\d+)/);
      if (aNum && bNum) {
        return parseInt(aNum[1]) - parseInt(bNum[1]);
      }
      if (aNum) return -1;
      if (bNum) return 1;
      return a.localeCompare(b);
    });
  }
  
  for (const fileName of sortedFiles) {
    const filePath = join(dirPath, fileName);
    files.push({
      path: filePath,
      name: fileName,
      content: readFileSync(filePath, 'utf-8'),
    });
  }
  
  return files;
}

function combineMarkdownFiles(files: MarkdownFile[]): string {
  const parts: string[] = [];
  
  for (let i = 0; i < files.length; i++) {
    const file = files[i];
    let content = file.content;
    
    // Remove YAML frontmatter
    content = content.replace(/^---[\s\S]*?---\n/, '');
    
    // Add page break between files (except before first)
    if (i > 0) {
      parts.push('\n\n---\n\n');
    }
    
    parts.push(content.trim());
  }
  
  return parts.join('\n\n');
}

async function main() {
  const options = parseArguments();
  
  console.log('');
  console.log('🦝 MechCrate md2pdf - Markdown to PDF Compiler');
  console.log('════════════════════════════════════════════════════════════════');
  console.log('');

  // Validate input
  if (!existsSync(options.input)) {
    console.error(`❌ Input not found: ${options.input}`);
    process.exit(1);
  }

  console.log(`   📥 Input:  ${options.input}${options.isDirectory ? ' (directory)' : ''}`);
  console.log(`   📤 Output: ${options.output}`);
  console.log(`   🎨 Theme:  ${options.theme}`);
  console.log('');

  // Create output directories
  const diagramsDir = join(options.outputDir, 'diagrams');
  mkdirSync(options.outputDir, { recursive: true });
  if (existsSync(diagramsDir)) {
    rmSync(diagramsDir, { recursive: true });
  }
  mkdirSync(diagramsDir, { recursive: true });

  // Step 1: Read and process document(s)
  let originalContent: string;
  let processed: string;
  
  if (options.isDirectory) {
    console.log('📄 Processing directory...');
    const files = getMarkdownFiles(options.input, options.fileOrder);
    
    if (files.length === 0) {
      console.error(`❌ No markdown files found in: ${options.input}`);
      process.exit(1);
    }
    
    console.log(`   📚 Found ${files.length} markdown files:`);
    for (const file of files) {
      console.log(`      • ${file.name}`);
    }
    
    originalContent = combineMarkdownFiles(files);
    processed = processDocument(originalContent, options);
  } else {
    console.log('📄 Processing markdown...');
    originalContent = readFileSync(options.input, 'utf-8');
    processed = processDocument(originalContent, options);
  }
  
  console.log(`   ✅ Document processed`);
  console.log(`   📊 Found ${allDiagrams.length} Mermaid diagrams`);

  // Step 2: Render Mermaid diagrams
  if (allDiagrams.length > 0) {
    console.log('');
    console.log('🎨 Rendering Mermaid diagrams...');
    await renderMermaidDiagrams(allDiagrams, diagramsDir, options.theme);
  }

  // Step 3: Generate processed markdown
  console.log('');
  console.log('📝 Generating processed markdown...');
  
  const finalMarkdown = generateFinalMarkdown(originalContent, processed, options);
  const markdownPath = options.output.replace('.pdf', '.md');
  writeFileSync(markdownPath, finalMarkdown);
  console.log(`   ✅ Written: ${basename(markdownPath)}`);

  // Copy assets
  copyAssets(options.outputDir);

  if (options.markdownOnly) {
    console.log('');
    console.log('════════════════════════════════════════════════════════════════');
    console.log('✅ Markdown generation complete!');
    console.log(`   📄 Output: ${markdownPath}`);
    console.log(`   📁 Diagrams: ${diagramsDir}`);
    return;
  }

  // Step 4: Generate HTML (always, as intermediate step)
  console.log('');
  console.log('📄 Generating HTML...');
  const htmlPath = await generateHTML(markdownPath, options);

  if (options.htmlOnly) {
    console.log('');
    console.log('════════════════════════════════════════════════════════════════');
    console.log('✅ HTML generation complete!');
    console.log(`   📄 Markdown: ${markdownPath}`);
    console.log(`   🌐 HTML: ${htmlPath}`);
    console.log(`   📁 Diagrams: ${diagramsDir}`);
    return;
  }

  // Step 5: Generate PDF
  console.log('');
  console.log('📕 Generating PDF...');
  await generatePDF(markdownPath, options);

  console.log('');
  console.log('════════════════════════════════════════════════════════════════');
  console.log('✅ PDF generation complete!');
  console.log(`   📄 Markdown: ${markdownPath}`);
  console.log(`   🌐 HTML: ${htmlPath}`);
  console.log(`   📕 PDF: ${options.output}`);
  console.log(`   📁 Diagrams: ${diagramsDir}`);
  console.log('');
}

// ═══════════════════════════════════════════════════════════════════════════════
// DOCUMENT PROCESSING
// ═══════════════════════════════════════════════════════════════════════════════

function extractTitle(content: string): string {
  const match = content.match(/^#\s+(.+)$/m);
  return match ? match[1].replace(/[*_`]/g, '').trim() : 'Document';
}

function processDocument(content: string, options: Options): string {
  let result = content;

  // Remove HTML comments
  result = result.replace(/<!--[\s\S]*?-->/g, '');

  // Remove YAML frontmatter if present (we'll add our own)
  result = result.replace(/^---[\s\S]*?---\n/, '');

  // Process Mermaid code blocks
  result = processMermaidBlocks(result, options);

  // Process ASCII diagrams
  result = processAsciiDiagrams(result);

  // Clean up excessive newlines
  result = result.replace(/\n{4,}/g, '\n\n\n');

  return result;
}

function processMermaidBlocks(content: string, options: Options): string {
  const mermaidRegex = /```mermaid\n([\s\S]*?)```/g;
  
  return content.replace(mermaidRegex, (match, mermaidContent) => {
    diagramCounter++;
    const diagramId = `diagram-${diagramCounter}`;
    
    const caption = extractDiagramCaption(content, match);
    
    allDiagrams.push({
      id: diagramId,
      type: 'mermaid',
      content: mermaidContent.trim(),
      caption,
    });
    
    const captionText = caption ? `\n*${caption}*` : '';
    return `\n![${caption || 'Diagram'}](diagrams/${diagramId}.png)${captionText}\n`;
  });
}

function processAsciiDiagrams(content: string): string {
  const codeBlockRegex = /```([a-z]*)\n([\s\S]*?)```/g;
  
  return content.replace(codeBlockRegex, (match, lang, blockContent) => {
    const knownLangs = ['typescript', 'javascript', 'ts', 'js', 'rust', 'python', 'sql', 
                       'yaml', 'json', 'bash', 'sh', 'css', 'html', 'php', 'go', 'solidity',
                       'graphql', 'toml', 'lean', 'coq', 'tla', 'anchor', 'text', 'tsx', 'jsx',
                       'ruby', 'swift', 'kotlin', 'java', 'c', 'cpp', 'csharp', 'r', 'perl',
                       'lua', 'make', 'makefile', 'dockerfile', 'xml', 'markdown', 'md'];
    
    if (knownLangs.includes(lang.toLowerCase())) {
      return match;
    }
    
    // Check if it looks like ASCII art
    const boxChars = /[┌┐└┘├┤┬┴┼│─═║╔╗╚╝╠╣╦╩╬┃━▲▼◄►●○◉■□▪▫★☆]/;
    const hasBoxChars = boxChars.test(blockContent);
    const hasArrows = /[-=]+>|<[-=]+|[─═]+►|◄[─═]+|↑|↓|←|→/.test(blockContent);
    
    if (hasBoxChars || hasArrows) {
      return `\n<div class="ascii-diagram">\n\n\`\`\`\n${blockContent}\`\`\`\n\n</div>\n`;
    }
    
    return match;
  });
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

// ═══════════════════════════════════════════════════════════════════════════════
// MERMAID RENDERING (PARALLEL)
// ═══════════════════════════════════════════════════════════════════════════════

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

async function parallelLimit<T, R>(
  items: T[],
  limit: number,
  fn: (item: T) => Promise<R>
): Promise<R[]> {
  const results: R[] = [];
  let index = 0;

  async function worker() {
    while (index < items.length) {
      const currentIndex = index++;
      const result = await fn(items[currentIndex]);
      results[currentIndex] = result;
    }
  }

  const workers = Array(Math.min(limit, items.length))
    .fill(null)
    .map(() => worker());

  await Promise.all(workers);
  return results;
}

async function renderMermaidDiagrams(
  diagrams: DiagramInfo[], 
  diagramsDir: string,
  theme: string
): Promise<void> {
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
  const concurrency = Math.max(1, numCores - 1);
  
  console.log(`   🚀 Parallel rendering with ${concurrency} workers`);

  const results = await parallelLimit(diagrams, concurrency, (diagram) =>
    renderSingleDiagram(diagram, diagramsDir, configPath)
  );

  let successCount = 0;
  let failCount = 0;

  for (const result of results) {
    if (result.success) {
      console.log(`   ✅ ${result.id}`);
      successCount++;
    } else {
      console.log(`   ⚠️  ${result.id} (${result.error || 'render failed'})`);
      failCount++;
      // Create placeholder text file for failed diagrams
      const textPath = join(diagramsDir, `${result.id}.txt`);
      const diagram = diagrams.find(d => d.id === result.id);
      writeFileSync(textPath, `[Diagram rendering failed]\n\nMermaid source:\n${diagram?.content || ''}`);
    }
  }

  console.log(`   📊 Rendered: ${successCount} success, ${failCount} failed`);
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKDOWN GENERATION
// ═══════════════════════════════════════════════════════════════════════════════

function generateFinalMarkdown(originalContent: string, processedContent: string, options: Options): string {
  const date = new Date().toLocaleDateString('en-US', { 
    year: 'numeric', 
    month: 'long', 
    day: 'numeric' 
  });

  const title = options.title || extractTitle(originalContent);
  const subtitle = options.subtitle || '';
  
  // Extract abstract from first blockquote after title if available
  const abstractMatch = originalContent.match(/^#[^\n]+\n+(?:>([^\n]+)\n+)?/);
  const abstract = abstractMatch?.[1]?.replace(/^>\s*/, '') || '';

  // Build YAML header
  let header = `---
title: "${title}"
`;

  if (subtitle) {
    header += `subtitle: "${subtitle}"\n`;
  }
  
  header += `author: "${options.author}"
date: "${date}"
`;

  if (abstract) {
    header += `abstract: |
  ${abstract}
`;
  }

  header += `toc: ${options.toc}
toc-depth: 3
numbersections: ${options.numbers}
colorlinks: true
linkcolor: blue
urlcolor: blue
geometry: margin=1in
fontsize: 11pt
documentclass: report
header-includes:
  - \\usepackage{fancyhdr}
  - \\pagestyle{fancy}
  - \\fancyhead[L]{${title.replace(/"/g, '\\"')}}
  - \\fancyhead[R]{\\thepage}
  - \\fancyfoot[C]{}
---

`;

  // Remove original title if we're using metadata title
  let body = processedContent;
  body = body.replace(/^#\s+[^\n]+\n/, '');
  
  // Remove blockquote subtitle if present
  body = body.replace(/^>\s*[^\n]+\n\n/, '');

  return header + body;
}

// ═══════════════════════════════════════════════════════════════════════════════
// ASSET COPYING
// ═══════════════════════════════════════════════════════════════════════════════

function copyAssets(outputDir: string): void {
  const cssSrc = join(__dirname, 'style.css');
  const cssDst = join(outputDir, 'style.css');
  if (existsSync(cssSrc)) {
    copyFileSync(cssSrc, cssDst);
  }
  
  const templateSrc = join(__dirname, 'template.latex');
  const templateDst = join(outputDir, 'template.latex');
  if (existsSync(templateSrc)) {
    copyFileSync(templateSrc, templateDst);
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HTML GENERATION
// ═══════════════════════════════════════════════════════════════════════════════

async function generateHTML(markdownPath: string, options: Options): Promise<string> {
  const htmlPath = markdownPath.replace('.md', '.html');
  const diagramsDir = join(options.outputDir, 'diagrams');
  const cssPath = join(options.outputDir, 'style.css');

  // Check for pandoc
  const pandocCheck = spawnSync('which', ['pandoc']);
  if (pandocCheck.status !== 0) {
    console.log('   ⚠️  Pandoc not found, skipping HTML generation');
    return htmlPath;
  }

  const htmlArgs = [
    markdownPath,
    '-o', htmlPath,
    '--standalone',
    '--highlight-style=zenburn',
    '--resource-path=' + options.outputDir + ':' + diagramsDir,
  ];

  // Try self-contained first, fall back to embed-resources
  htmlArgs.push('--embed-resources');

  if (options.toc) {
    htmlArgs.push('--toc', '--toc-depth=3');
  }
  if (options.numbers) {
    htmlArgs.push('--number-sections');
  }
  if (existsSync(cssPath)) {
    htmlArgs.push('--css=' + cssPath);
  }

  const result = spawnSync('pandoc', htmlArgs, { 
    cwd: options.outputDir,
    stdio: ['pipe', 'pipe', 'pipe'],
  });

  if (result.status === 0) {
    console.log(`   ✅ HTML generated: ${basename(htmlPath)}`);
  } else {
    const stderr = result.stderr?.toString() || '';
    console.log(`   ⚠️  HTML generation failed: ${stderr.split('\n')[0]}`);
  }

  return htmlPath;
}

// ═══════════════════════════════════════════════════════════════════════════════
// PDF GENERATION
// ═══════════════════════════════════════════════════════════════════════════════

async function generatePDF(markdownPath: string, options: Options): Promise<void> {
  const pdfPath = options.output;
  const diagramsDir = join(options.outputDir, 'diagrams');
  
  // Check for pandoc
  const pandocCheck = spawnSync('which', ['pandoc']);
  if (pandocCheck.status !== 0) {
    console.log('   ⚠️  Pandoc not found.');
    console.log('   💡 Install with: brew install pandoc');
    return;
  }

  // Check for LaTeX
  const xelatexCheck = spawnSync('which', ['xelatex']);
  const hasLatex = xelatexCheck.status === 0;

  // Build pandoc args
  const baseArgs = [
    markdownPath,
    '-o', pdfPath,
    '--from', 'markdown+yaml_metadata_block+backtick_code_blocks+fenced_code_attributes+pipe_tables',
    '--highlight-style=zenburn',
    '--standalone',
    '--resource-path=' + options.outputDir + ':' + diagramsDir,
  ];

  if (options.toc) {
    baseArgs.push('--toc', '--toc-depth=3');
  }
  if (options.numbers) {
    baseArgs.push('--number-sections');
  }

  // Try LaTeX first
  if (hasLatex) {
    console.log('   🔄 Running pandoc with xelatex...');
    const result = spawnSync('pandoc', [
      ...baseArgs,
      '--pdf-engine=xelatex',
      '-V', 'geometry:margin=1in',
      '-V', 'fontsize=11pt',
      '-V', 'documentclass=report',
      '-V', 'colorlinks=true',
      '-V', 'linkcolor=blue',
      '-V', 'urlcolor=blue',
      '-V', 'mainfont=Helvetica Neue',
      '-V', 'monofont=Menlo',
      '-V', 'classoption=oneside',
    ], {
      cwd: options.outputDir,
      stdio: ['pipe', 'pipe', 'pipe'],
    });

    if (result.status === 0) {
      console.log('   ✅ PDF generated with LaTeX');
      return;
    }
    const stderr = result.stderr?.toString() || '';
    console.log('   ⚠️  LaTeX failed:', stderr.split('\n')[0]);
  }

  // Check for weasyprint
  const weasyprintCheck = spawnSync('which', ['weasyprint']);
  const hasWeasyprint = weasyprintCheck.status === 0;

  if (hasWeasyprint) {
    console.log('   🔄 Running WeasyPrint...');
    
    const htmlPath = markdownPath.replace('.md', '.html');
    
    if (existsSync(htmlPath)) {
      const weasyResult = spawnSync('weasyprint', [htmlPath, pdfPath], { 
        cwd: options.outputDir,
        stdio: ['pipe', 'pipe', 'pipe'],
      });
      
      if (weasyResult.status === 0) {
        console.log('   ✅ PDF generated with WeasyPrint');
        return;
      }
    }
  }

  // Last resort: basic pandoc
  console.log('   🔄 Trying basic pandoc...');
  const basicResult = spawnSync('pandoc', [
    markdownPath,
    '-o', pdfPath,
    '--resource-path=' + options.outputDir + ':' + diagramsDir,
    ...(options.toc ? ['--toc', '--toc-depth=3'] : []),
  ], {
    cwd: options.outputDir,
    stdio: ['pipe', 'pipe', 'pipe'],
  });

  if (basicResult.status === 0) {
    console.log('   ✅ PDF generated (basic)');
    return;
  }

  console.log('   ❌ PDF generation failed.');
  console.log('   📄 Markdown and HTML files are available in:', options.outputDir);
}

// ═══════════════════════════════════════════════════════════════════════════════
// RUN
// ═══════════════════════════════════════════════════════════════════════════════

main().catch((err) => {
  console.error('❌ Compilation failed:', err.message);
  process.exit(1);
});
