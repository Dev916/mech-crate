# MechCrate docs

**Portable** Markdown to PDF compiler with Mermaid diagram support. Zero external dependencies - everything runs via npm packages.

## Features

- **Zero External Dependencies**: No need to install Pandoc, LaTeX, or any other system tools
- **Portable**: Works on any system with Node.js 18+
- **Mermaid Diagram Rendering**: Automatically converts Mermaid diagrams to images
- **Syntax Highlighting**: Code blocks are highlighted with highlight.js
- **YAML Frontmatter**: Extract metadata from documents
- **PDF Generation**: Uses Puppeteer with bundled Chromium - just works!
- **Unyform Integration**: Built-in support for compiling unyform.ai documents

## Quick Start

```bash
# Via mx command (recommended)
mx docs docs/README.md

# Direct usage
cd scripts/docs
npm install
npx tsx compile.ts --file=path/to/file.md
```

## Usage

### Single File

```bash
mx docs docs/guide.md
mx docs docs/spec.md -o artifacts/
```

### Folder

```bash
mx docs docs/api/
mx docs docs/manual/ -o artifacts/manuals/
```

### Unyform Documents

```bash
mx docs --list              # List available unyform docs
mx docs --unyform           # Compile all unyform docs
mx docs --doc=whitepaper    # Compile specific doc
```

## Options

| Option | Description |
|--------|-------------|
| `--file=<path>` | Single markdown file to compile |
| `--folder=<path>` | Folder containing markdown files |
| `-o, --output <path>` | Output directory for PDFs |
| `--prefix=<string>` | Add prefix to output filenames |
| `--author=<name>` | Default author for docs without frontmatter |
| `--markdown-only` | Only generate processed markdown, skip PDF |
| `--no-recursive` | Don't scan subfolders |
| `-v, --verbose` | Show detailed progress |
| `--list` | List available unyform documents |
| `--all` | Compile all unyform documents |
| `--doc=<name>` | Compile specific unyform document |

## YAML Frontmatter

Documents can include YAML frontmatter for metadata:

```markdown
---
title: My Document
subtitle: Optional Subtitle
author: Author Name
date: January 2026
toc: true
abstract: Brief description of the document
---

# Document Content

...
```

## Output Structure

```
output/
├── document.pdf           # Final PDF
├── document.html          # HTML version
├── document.md            # Processed markdown
└── diagrams/              # Rendered Mermaid PNGs
```

## How It Works

1. **Parse**: Read markdown and extract YAML frontmatter using `gray-matter`
2. **Diagrams**: Extract and render Mermaid diagrams using `mermaid-cli`
3. **Convert**: Convert markdown to HTML using `marked` with `highlight.js`
4. **PDF**: Generate PDF from HTML using Puppeteer's bundled Chromium

## Dependencies

All dependencies are installed via npm - no system-level packages required:

- `marked` - Markdown to HTML
- `marked-highlight` + `highlight.js` - Syntax highlighting
- `gray-matter` - YAML frontmatter parsing
- `puppeteer` - HTML to PDF (bundles its own Chromium)
- `@mermaid-js/mermaid-cli` - Diagram rendering

## Comparison to Previous Implementation

| Feature | Old (templates/scripts/md2pdf) | New (scripts/docs) |
|---------|-------------------------------|-------------------|
| Pandoc | Required | Not needed |
| LaTeX | Recommended | Not needed |
| WeasyPrint | Fallback | Not needed |
| Portability | Requires system tools | Just Node.js |
| First run | May fail without deps | Just works |

## Troubleshooting

### First Run is Slow

The first run downloads Puppeteer's bundled Chromium (~200MB). Subsequent runs are fast.

### Diagrams Not Rendering

Check that the Mermaid syntax is valid. The tool will show warnings for failed diagrams.

### PDF Generation Fails

Usually due to Puppeteer issues. Try:
```bash
rm -rf node_modules
npm install
```
