# MechCrate docs (md2pdf)

Markdown to PDF compiler with Mermaid diagram rendering. Part of the MechCrate toolkit.

## Features

- **Mermaid Diagram Rendering**: All Mermaid diagrams are rendered as high-resolution PNG images
- **Multiple Themes**: Dark, light, forest, and neutral themes for diagrams
- **Table of Contents**: Automatic TOC generation with configurable depth
- **Syntax Highlighting**: Code blocks are syntax highlighted with zenburn theme
- **Professional Styling**: Clean PDF output via Pandoc + LaTeX or WeasyPrint
- **Markdown Tables**: Full support for pipe tables
- **Directory Mode**: Combine multiple markdown files into one document
- **HTML Output**: Generate standalone HTML as intermediate format

## Quick Start

```bash
# Via mx command (recommended)
mx docs docs/README.md

# Direct usage
cd templates/scripts/md2pdf
npm install
npx tsx md2pdf.ts path/to/file.md
```

## Usage

### Single File

```bash
mx docs docs/guide.md
mx docs docs/spec.md --output artifacts/spec.pdf
mx docs docs/README.md --title "Project Documentation"
```

### Directory of Files

```bash
mx docs docs/api/
mx docs docs/manual/ --title "User Manual" --order "intro.md,setup.md,usage.md"
```

## Options

| Option | Description |
|--------|-------------|
| `--output, -o` | Output PDF path (default: `<name>-output/<name>.pdf`) |
| `--markdown-only` | Only generate processed markdown, skip PDF |
| `--html-only` | Only generate HTML, skip PDF |
| `--title` | Document title (default: extracted from first H1) |
| `--subtitle` | Document subtitle |
| `--author` | Document author (default: "MechCrate") |
| `--theme` | Mermaid theme: `dark`, `light`, `forest`, `neutral` (default: dark) |
| `--order` | File order for directories (comma-separated) |
| `--no-toc` | Disable table of contents |
| `--no-numbers` | Disable section numbering |
| `-h, --help` | Show help |

## Output Structure

Each conversion creates an output directory:

```
<name>-output/
├── <name>.pdf           # Final PDF
├── <name>.md            # Processed markdown with diagram references
├── <name>.html          # HTML version
├── diagrams/            # Rendered Mermaid PNGs
│   ├── diagram-1.png
│   ├── diagram-1.mmd    # Original Mermaid source
│   └── ...
├── style.css            # PDF/HTML styling
├── template.latex       # LaTeX template
└── mermaid-config.json  # Mermaid rendering config
```

## Mermaid Themes

| Theme | Description |
|-------|-------------|
| `dark` | MechCrate dark theme (default) - Purple accents on dark background |
| `light` | Light theme - Dark accents on white background |
| `forest` | Mermaid forest theme |
| `neutral` | Mermaid neutral theme |

## Dependencies

### Required

- Node.js 18+
- npm

### For PDF Generation (at least one)

- [Pandoc](https://pandoc.org/) - `brew install pandoc`
- [XeLaTeX](https://tug.org/mactex/) - `brew install --cask mactex-no-gui` (best quality)
- [WeasyPrint](https://weasyprint.org/) - `pip install weasyprint` (alternative)

### Auto-installed

- `@mermaid-js/mermaid-cli` - Mermaid diagram renderer
- `tsx` - TypeScript execution

## Examples

### Basic Conversion

```bash
mx docs docs/README.md
```

### Full Customization

```bash
mx docs docs/api-spec.md \
  --output artifacts/api-spec-v1.pdf \
  --title "API Specification v1.0" \
  --subtitle "REST API Reference" \
  --author "Engineering Team" \
  --theme dark
```

### Without TOC and Numbering

```bash
mx docs docs/notes.md --no-toc --no-numbers
```

### Light Theme for Printing

```bash
mx docs docs/guide.md --theme light
```

### Directory with Custom Order

```bash
mx docs docs/manual/ \
  --title "User Manual" \
  --order "00-intro.md,01-installation.md,02-configuration.md,03-usage.md"
```

### Generate Only Markdown (for inspection)

```bash
mx docs docs/spec.md --markdown-only
```

### Generate Only HTML

```bash
mx docs docs/spec.md --html-only
```

## Diagram Captions

Captions are automatically extracted from text immediately before Mermaid blocks:

```markdown
**System Architecture**

```mermaid
graph TD
    A --> B
```
```

This will render as a diagram with the caption "System Architecture".

## ASCII Diagrams

ASCII art using box-drawing characters is automatically detected and styled:

```
┌─────────┐     ┌─────────┐
│  Client │────▶│  Server │
└─────────┘     └─────────┘
```

## Integration with MechCrate

This tool is integrated into the `mx` CLI:

```bash
mx docs <file>
mx docs --help
```

## Troubleshooting

### Mermaid Diagrams Not Rendering

1. Ensure Node.js 18+ is installed
2. Run `npm install` in the md2pdf directory
3. Check for syntax errors in your Mermaid diagrams

### PDF Generation Fails

1. Install Pandoc: `brew install pandoc`
2. For best results, install LaTeX: `brew install --cask mactex-no-gui`
3. Alternative: Install WeasyPrint: `pip install weasyprint`

### Font Issues in PDF

On macOS, the default fonts (Helvetica Neue, Menlo) should be available.
On Linux, you may need to install equivalent fonts or modify the template.

## Contributing

This tool is part of MechCrate. See the main [MechCrate README](../../../README.md) for contribution guidelines.
