#!/usr/bin/env tsx
/**
 * unyform.ai Document Compiler
 * 
 * Compiles Markdown documents to PDF using Pandoc + XeLaTeX
 * with Mermaid diagram rendering support.
 * 
 * Usage:
 *   npx tsx compile.ts                       # Compile all unyform docs
 *   npx tsx compile.ts --doc=whitepaper      # Compile specific document
 *   npx tsx compile.ts --list                # List available documents
 *   npx tsx compile.ts --folder=./my-docs    # Compile all .md files in folder
 *   npx tsx compile.ts --file=./doc.md       # Compile a single file
 *   npx tsx compile.ts --folder=./docs --output=./pdfs  # Custom output dir
 * 
 * Frontmatter Support:
 *   Documents can include YAML frontmatter for metadata:
 *   ---
 *   title: My Document
 *   subtitle: A great document
 *   author: Author Name
 *   toc: true
 *   ---
 */

import { execSync, spawnSync } from 'child_process';
import { existsSync, mkdirSync, readFileSync, writeFileSync, readdirSync, rmSync, statSync } from 'fs';
import { join, basename, dirname, extname, relative, resolve } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Configuration
const ROOT_DIR = join(__dirname, '../..');
const DEFAULT_DOCS_DIR = join(ROOT_DIR, 'docs/unyform');
const DEFAULT_ARTIFACTS_DIR = join(ROOT_DIR, 'artifacts/unyform');
const TEMPLATE_FILE = join(__dirname, 'template.latex');

// Document configuration interface
interface DocConfig {
  file: string;
  title: string;
  subtitle?: string;
  author: string;
  toc: boolean;
  date?: string;
  abstract?: string;
}

// Frontmatter interface
interface Frontmatter {
  title?: string;
  subtitle?: string;
  author?: string;
  toc?: boolean;
  date?: string;
  abstract?: string;
  [key: string]: unknown;
}

// Predefined unyform documents
const UNYFORM_DOCUMENTS: Record<string, DocConfig> = {
  'whitepaper': {
    file: 'WHITEPAPER.md',
    title: 'unyform.ai Technical Whitepaper',
    subtitle: 'Enterprise AI Trust and Consistency Layer',
    author: 'unyform.ai',
    toc: true,
  },
  'executive-summary': {
    file: 'EXECUTIVE_SUMMARY.md',
    title: 'unyform.ai Executive Summary',
    subtitle: 'AI Infrastructure Governance for Engineering Teams',
    author: 'unyform.ai',
    toc: false,
  },
  'roadmap': {
    file: 'ROADMAP.md',
    title: 'unyform.ai Product Roadmap',
    subtitle: '2024-2026 Strategic Plan',
    author: 'unyform.ai',
    toc: true,
  },
  'competitive-analysis': {
    file: 'COMPETITIVE_ANALYSIS.md',
    title: 'unyform.ai Competitive Analysis',
    subtitle: 'Market Positioning and Strategy',
    author: 'unyform.ai',
    toc: true,
  },
  'mvp-prd': {
    file: 'MVP_PRD.md',
    title: 'unyform.ai MVP Product Requirements',
    subtitle: 'Phase 1 Specification',
    author: 'unyform.ai',
    toc: true,
  },
  'pitch-deck': {
    file: 'PITCH_DECK.md',
    title: 'unyform.ai Pitch Deck',
    subtitle: 'Investor Presentation',
    author: 'unyform.ai',
    toc: false,
  },
  'gtm-playbook': {
    file: 'GTM_PLAYBOOK.md',
    title: 'unyform.ai Go-to-Market Playbook',
    subtitle: 'Sales and Marketing Strategy',
    author: 'unyform.ai',
    toc: true,
  },
  'tech-architecture': {
    file: 'TECHNICAL_ARCHITECTURE.md',
    title: 'unyform.ai Technical Architecture',
    subtitle: 'System Design Specification',
    author: 'unyform.ai',
    toc: true,
  },
  'pricing-strategy': {
    file: 'PRICING_STRATEGY.md',
    title: 'unyform.ai Pricing Strategy',
    subtitle: 'Packaging and Unit Economics',
    author: 'unyform.ai',
    toc: true,
  },
};

// CLI arguments interface
interface CliArgs {
  doc?: string;
  folder?: string;
  file?: string;
  output?: string;
  all: boolean;
  list: boolean;
  markdownOnly: boolean;
  recursive: boolean;
  prefix?: string;
  author?: string;
  verbose: boolean;
}

// Parse command line arguments
function parseArgs(): CliArgs {
  const args = process.argv.slice(2);
  const result: CliArgs = {
    all: false,
    list: false,
    markdownOnly: process.env.MARKDOWN_ONLY === '1',
    recursive: true,
    verbose: false,
  };

  for (const arg of args) {
    if (arg === '--all') result.all = true;
    else if (arg === '--list') result.list = true;
    else if (arg === '--markdown-only') result.markdownOnly = true;
    else if (arg === '--no-recursive') result.recursive = false;
    else if (arg === '--verbose' || arg === '-v') result.verbose = true;
    else if (arg.startsWith('--doc=')) result.doc = arg.split('=')[1];
    else if (arg.startsWith('--folder=') || arg.startsWith('--dir=')) {
      result.folder = arg.split('=')[1];
    }
    else if (arg.startsWith('--file=')) result.file = arg.split('=')[1];
    else if (arg.startsWith('--output=') || arg.startsWith('--out=')) {
      result.output = arg.split('=')[1];
    }
    else if (arg.startsWith('--prefix=')) result.prefix = arg.split('=')[1];
    else if (arg.startsWith('--author=')) result.author = arg.split('=')[1];
  }

  // Default to --all if no specific action specified
  if (!result.doc && !result.folder && !result.file && !result.list) {
    result.all = true;
  }

  return result;
}

// Check dependencies
function checkDependencies(): boolean {
  const deps = [
    { cmd: 'pandoc', check: 'pandoc --version' },
    { cmd: 'xelatex', check: 'xelatex --version' },
  ];

  let allGood = true;
  for (const dep of deps) {
    try {
      execSync(dep.check, { stdio: 'pipe' });
      console.log(`  ✅ ${dep.cmd} found`);
    } catch {
      console.log(`  ❌ ${dep.cmd} not found`);
      allGood = false;
    }
  }

  return allGood;
}

// Parse YAML frontmatter from markdown content
function parseFrontmatter(content: string): { frontmatter: Frontmatter; body: string } {
  const frontmatterRegex = /^---\s*\n([\s\S]*?)\n---\s*\n/;
  const match = content.match(frontmatterRegex);
  
  if (!match) {
    return { frontmatter: {}, body: content };
  }
  
  const yamlContent = match[1];
  const body = content.slice(match[0].length);
  
  // Simple YAML parser for common cases
  const frontmatter: Frontmatter = {};
  const lines = yamlContent.split('\n');
  
  for (const line of lines) {
    const colonIndex = line.indexOf(':');
    if (colonIndex === -1) continue;
    
    const key = line.slice(0, colonIndex).trim();
    let value: string | boolean = line.slice(colonIndex + 1).trim();
    
    // Handle quoted strings
    if ((value.startsWith('"') && value.endsWith('"')) ||
        (value.startsWith("'") && value.endsWith("'"))) {
      value = value.slice(1, -1);
    }
    
    // Handle booleans
    if (value.toLowerCase() === 'true') value = true as any;
    else if (value.toLowerCase() === 'false') value = false as any;
    
    frontmatter[key] = value;
  }
  
  return { frontmatter, body };
}

// Generate title from filename
function titleFromFilename(filename: string): string {
  const name = basename(filename, extname(filename));
  return name
    .replace(/[-_]/g, ' ')
    .replace(/([a-z])([A-Z])/g, '$1 $2')
    .split(' ')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
    .join(' ');
}

// Generate slug from filename
function slugFromFilename(filename: string): string {
  const name = basename(filename, extname(filename));
  return name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '');
}

// Extract and render Mermaid diagrams
async function renderMermaidDiagrams(
  content: string, 
  outputDir: string,
  verbose: boolean
): Promise<string> {
  mkdirSync(outputDir, { recursive: true });

  const mermaidRegex = /```mermaid\n([\s\S]*?)```/g;
  let match;
  let index = 0;
  let result = content;

  while ((match = mermaidRegex.exec(content)) !== null) {
    const diagram = match[1];
    const diagramFile = join(outputDir, `diagram-${index}.mmd`);
    const outputFile = join(outputDir, `diagram-${index}.png`);
    
    writeFileSync(diagramFile, diagram);

    try {
      // Use mermaid-cli to render
      execSync(`npx mmdc -i "${diagramFile}" -o "${outputFile}" -t dark -b transparent -w 1200`, {
        cwd: __dirname,
        stdio: 'pipe',
      });

      // Replace mermaid block with image reference
      const relativePath = `diagrams/diagram-${index}.png`;
      result = result.replace(match[0], `![Diagram ${index + 1}](${relativePath})`);
      
      if (verbose) {
        console.log(`    📊 Rendered diagram ${index + 1}`);
      }
    } catch (err) {
      console.log(`    ⚠️  Failed to render diagram ${index + 1}, keeping as code block`);
    }

    index++;
  }

  return result;
}

// Compile a single file to PDF
async function compileFile(
  inputPath: string,
  outputDir: string,
  options: {
    markdownOnly: boolean;
    prefix?: string;
    defaultAuthor?: string;
    verbose: boolean;
  }
): Promise<boolean> {
  if (!existsSync(inputPath)) {
    console.error(`❌ File not found: ${inputPath}`);
    return false;
  }

  const absoluteInput = resolve(inputPath);
  const filename = basename(inputPath);
  const slug = slugFromFilename(filename);
  const outputSlug = options.prefix ? `${options.prefix}-${slug}` : slug;
  
  // Read and parse content
  const rawContent = readFileSync(absoluteInput, 'utf-8');
  const { frontmatter, body } = parseFrontmatter(rawContent);
  
  // Build config from frontmatter or defaults
  const config: DocConfig = {
    file: filename,
    title: (frontmatter.title as string) || titleFromFilename(filename),
    subtitle: frontmatter.subtitle as string | undefined,
    author: (frontmatter.author as string) || options.defaultAuthor || 'Document Author',
    toc: frontmatter.toc !== undefined ? Boolean(frontmatter.toc) : true,
    date: frontmatter.date as string | undefined,
    abstract: frontmatter.abstract as string | undefined,
  };

  console.log(`\n📄 Compiling: ${config.title}`);
  console.log(`   Source: ${inputPath}`);

  // Create output directory
  mkdirSync(outputDir, { recursive: true });
  
  // Set up diagrams directory
  const diagramsDir = join(outputDir, 'diagrams', outputSlug);
  
  // Process content (render Mermaid diagrams)
  if (options.verbose) {
    console.log('   Processing diagrams...');
  }
  let processedContent = await renderMermaidDiagrams(body, diagramsDir, options.verbose);

  // Create temp file with processed content
  const tempFile = join(outputDir, `${outputSlug}-processed.md`);
  writeFileSync(tempFile, processedContent);

  if (options.markdownOnly) {
    const outputMd = join(outputDir, `${outputSlug}.md`);
    writeFileSync(outputMd, processedContent);
    console.log(`   ✅ Markdown: ${outputMd}`);
    return true;
  }

  // Build Pandoc command
  const outputPdf = join(outputDir, `${outputSlug}.pdf`);
  const date = config.date || new Date().toLocaleDateString('en-US', { 
    year: 'numeric', 
    month: 'long', 
    day: 'numeric' 
  });

  const pandocArgs = [
    tempFile,
    '-o', outputPdf,
    '--pdf-engine=xelatex',
    '--template=' + TEMPLATE_FILE,
    '-V', `title=${config.title}`,
    '-V', `author=${config.author}`,
    '-V', `date=${date}`,
    '-V', 'documentclass=article',
    '-V', 'colorlinks=true',
    '--resource-path=' + outputDir,
  ];

  if (config.subtitle) {
    pandocArgs.push('-V', `subtitle=${config.subtitle}`);
  }

  if (config.abstract) {
    pandocArgs.push('-V', `abstract=${config.abstract}`);
  }

  if (config.toc) {
    pandocArgs.push('--toc');
    pandocArgs.push('--toc-depth=3');
  }

  try {
    if (options.verbose) {
      console.log('   Running Pandoc...');
    }
    const result = spawnSync('pandoc', pandocArgs, {
      cwd: outputDir,
      stdio: 'pipe',
      encoding: 'utf-8',
    });

    if (result.status !== 0) {
      console.error(`   ❌ Pandoc failed:`, result.stderr);
      return false;
    }

    console.log(`   ✅ PDF: ${outputPdf}`);
  } catch (err) {
    console.error(`   ❌ Failed to compile:`, err);
    return false;
  }

  // Cleanup temp file
  try {
    rmSync(tempFile);
  } catch {}

  return true;
}

// Find all markdown files in a directory
function findMarkdownFiles(dir: string, recursive: boolean): string[] {
  const files: string[] = [];
  
  if (!existsSync(dir)) {
    return files;
  }

  const entries = readdirSync(dir);
  
  for (const entry of entries) {
    const fullPath = join(dir, entry);
    const stat = statSync(fullPath);
    
    if (stat.isFile() && extname(entry).toLowerCase() === '.md') {
      files.push(fullPath);
    } else if (stat.isDirectory() && recursive) {
      // Skip common non-doc directories
      if (!['node_modules', '.git', 'dist', 'build', 'coverage'].includes(entry)) {
        files.push(...findMarkdownFiles(fullPath, recursive));
      }
    }
  }
  
  return files.sort();
}

// Compile a predefined unyform document
async function compileUnyformDocument(
  docKey: string, 
  outputDir: string,
  options: { markdownOnly: boolean; verbose: boolean }
): Promise<boolean> {
  const config = UNYFORM_DOCUMENTS[docKey];
  if (!config) {
    console.error(`❌ Unknown document: ${docKey}`);
    console.log('   Available documents:', Object.keys(UNYFORM_DOCUMENTS).join(', '));
    return false;
  }

  const inputFile = join(DEFAULT_DOCS_DIR, config.file);
  if (!existsSync(inputFile)) {
    console.log(`⚠️  Skipping ${docKey}: ${config.file} not found`);
    return false;
  }

  console.log(`\n📄 Compiling: ${config.title}`);
  console.log(`   Source: ${config.file}`);

  // Read source file
  let content = readFileSync(inputFile, 'utf-8');
  
  // Remove any frontmatter (we use predefined config)
  const { body } = parseFrontmatter(content);

  // Render Mermaid diagrams
  if (options.verbose) {
    console.log('   Processing diagrams...');
  }
  const diagramsDir = join(outputDir, 'diagrams', docKey);
  content = await renderMermaidDiagrams(body, diagramsDir, options.verbose);

  // Create temp file with processed content
  mkdirSync(outputDir, { recursive: true });
  const tempFile = join(outputDir, `${docKey}-processed.md`);
  writeFileSync(tempFile, content);

  if (options.markdownOnly) {
    const outputMd = join(outputDir, `unyform-${docKey}.md`);
    writeFileSync(outputMd, content);
    console.log(`   ✅ Markdown: ${outputMd}`);
    return true;
  }

  // Build Pandoc command
  const outputPdf = join(outputDir, `unyform-${docKey}.pdf`);
  const date = new Date().toLocaleDateString('en-US', { 
    year: 'numeric', 
    month: 'long', 
    day: 'numeric' 
  });

  const pandocArgs = [
    tempFile,
    '-o', outputPdf,
    '--pdf-engine=xelatex',
    '--template=' + TEMPLATE_FILE,
    '-V', `title=${config.title}`,
    '-V', `author=${config.author}`,
    '-V', `date=${date}`,
    '-V', 'documentclass=article',
    '-V', 'colorlinks=true',
    '--resource-path=' + outputDir,
  ];

  if (config.subtitle) {
    pandocArgs.push('-V', `subtitle=${config.subtitle}`);
  }

  if (config.toc) {
    pandocArgs.push('--toc');
    pandocArgs.push('--toc-depth=3');
  }

  try {
    if (options.verbose) {
      console.log('   Running Pandoc...');
    }
    const result = spawnSync('pandoc', pandocArgs, {
      cwd: outputDir,
      stdio: 'pipe',
      encoding: 'utf-8',
    });

    if (result.status !== 0) {
      console.error(`   ❌ Pandoc failed:`, result.stderr);
      return false;
    }

    console.log(`   ✅ PDF: ${outputPdf}`);
  } catch (err) {
    console.error(`   ❌ Failed to compile:`, err);
    return false;
  }

  // Cleanup temp file
  try {
    rmSync(tempFile);
  } catch {}

  return true;
}

// Print help
function printHelp(): void {
  console.log(`
Usage: npx tsx compile.ts [options]

Options:
  --all                   Compile all predefined unyform documents (default)
  --doc=<name>            Compile a specific predefined document
  --list                  List available predefined documents
  
  --folder=<path>         Compile all .md files in a folder
  --file=<path>           Compile a single markdown file
  --output=<path>         Output directory (default: artifacts/unyform or ./output)
  
  --prefix=<string>       Add prefix to output filenames
  --author=<string>       Default author for documents without frontmatter
  --no-recursive          Don't scan subfolders (for --folder)
  --markdown-only         Output processed markdown instead of PDF
  --verbose, -v           Show detailed progress
  
Examples:
  npx tsx compile.ts                              # Compile all unyform docs
  npx tsx compile.ts --doc=whitepaper             # Compile specific doc
  npx tsx compile.ts --folder=./my-docs           # Compile folder
  npx tsx compile.ts --file=./spec.md             # Compile single file
  npx tsx compile.ts --folder=./docs --output=./pdfs --prefix=v2
  
Frontmatter:
  Documents can include YAML frontmatter for metadata:
  
  ---
  title: My Document Title
  subtitle: Optional Subtitle
  author: Author Name
  toc: true
  date: January 2025
  abstract: Brief description of the document
  ---
`);
}

// Main
async function main() {
  const args = parseArgs();

  // Check for help flag
  if (process.argv.includes('--help') || process.argv.includes('-h')) {
    printHelp();
    return;
  }

  console.log('═══════════════════════════════════════════════════════════════');
  console.log('           Document Compiler');
  console.log('═══════════════════════════════════════════════════════════════');

  // List predefined documents
  if (args.list) {
    console.log('\nPredefined unyform documents:');
    for (const [key, config] of Object.entries(UNYFORM_DOCUMENTS)) {
      const exists = existsSync(join(DEFAULT_DOCS_DIR, config.file)) ? '✅' : '❌';
      console.log(`  ${exists} ${key}: ${config.title}`);
    }
    return;
  }

  // Check dependencies
  console.log('\nChecking dependencies...');
  if (!checkDependencies()) {
    console.error('\n❌ Missing dependencies. Please install:');
    console.error('   brew install pandoc');
    console.error('   brew install --cask mactex-no-gui');
    process.exit(1);
  }

  let successCount = 0;
  let failCount = 0;

  // Compile a single file
  if (args.file) {
    const outputDir = args.output || join(dirname(resolve(args.file)), 'output');
    const success = await compileFile(args.file, outputDir, {
      markdownOnly: args.markdownOnly,
      prefix: args.prefix,
      defaultAuthor: args.author,
      verbose: args.verbose,
    });
    successCount = success ? 1 : 0;
    failCount = success ? 0 : 1;
  }
  // Compile a folder
  else if (args.folder) {
    const folderPath = resolve(args.folder);
    const outputDir = args.output ? resolve(args.output) : join(folderPath, 'output');
    
    console.log(`\n📁 Scanning folder: ${folderPath}`);
    const files = findMarkdownFiles(folderPath, args.recursive);
    
    if (files.length === 0) {
      console.log('   No markdown files found.');
    } else {
      console.log(`   Found ${files.length} markdown file(s)`);
      
      for (const file of files) {
        const success = await compileFile(file, outputDir, {
          markdownOnly: args.markdownOnly,
          prefix: args.prefix,
          defaultAuthor: args.author,
          verbose: args.verbose,
        });
        if (success) successCount++;
        else failCount++;
      }
    }
  }
  // Compile a specific predefined document
  else if (args.doc) {
    const outputDir = args.output || DEFAULT_ARTIFACTS_DIR;
    const success = await compileUnyformDocument(args.doc, outputDir, {
      markdownOnly: args.markdownOnly,
      verbose: args.verbose,
    });
    successCount = success ? 1 : 0;
    failCount = success ? 0 : 1;
  }
  // Compile all predefined documents
  else if (args.all) {
    const outputDir = args.output || DEFAULT_ARTIFACTS_DIR;
    mkdirSync(outputDir, { recursive: true });
    
    for (const docKey of Object.keys(UNYFORM_DOCUMENTS)) {
      const success = await compileUnyformDocument(docKey, outputDir, {
        markdownOnly: args.markdownOnly,
        verbose: args.verbose,
      });
      if (success) successCount++;
      else failCount++;
    }
  }

  console.log('\n═══════════════════════════════════════════════════════════════');
  console.log(`   Compilation complete!`);
  console.log(`   ✅ Success: ${successCount}  ${failCount > 0 ? `❌ Failed: ${failCount}` : ''}`);
  if (args.output || args.folder || args.file) {
    console.log(`   Output: ${args.output || (args.folder ? join(resolve(args.folder), 'output') : join(dirname(resolve(args.file!)), 'output'))}`);
  } else {
    console.log(`   Output: ${DEFAULT_ARTIFACTS_DIR}`);
  }
  console.log('═══════════════════════════════════════════════════════════════\n');
}

main().catch(console.error);
