# {{SERVICE_NAME}}

A blazing fast static site built with [Zola](https://www.getzola.org/).

## Features

- **Single Binary**: Zola is a single executable with no dependencies
- **Fast Builds**: Average sites build in under a second
- **Sass/SCSS**: Built-in compilation, no configuration needed
- **Syntax Highlighting**: Over 100 themes for code blocks
- **Full-Text Search**: Static search index generated at build time
- **Live Reload**: Instant updates during development
- **Taxonomies**: Tags, categories, and custom taxonomies
- **RSS/Atom Feeds**: Auto-generated feeds for your content

## Quick Start

### Local Development (Recommended)

1. **Install Zola** - [Installation Guide](https://www.getzola.org/documentation/getting-started/installation/)

   ```bash
   # macOS (Homebrew)
   brew install zola

   # Windows (Chocolatey)
   choco install zola

   # Linux (Snap)
   snap install zola --edge
   ```

2. **Start Development Server**

   ```bash
   cd apps/{{SERVICE_NAME}}
   zola serve
   ```

   Open http://localhost:1111 in your browser.

### Docker Development

```bash
# Using Make
make dev s={{SERVICE_NAME}}

# Or Docker Compose directly
docker compose -f docker/compose/{{SERVICE_NAME}}.yml -f docker/compose/{{SERVICE_NAME}}.dev.yml up
```

## Project Structure

```
apps/{{SERVICE_NAME}}/
├── config.toml          # Site configuration
├── content/             # Markdown content
│   ├── _index.md        # Homepage content
│   ├── about.md         # About page
│   └── blog/            # Blog section
│       ├── _index.md    # Section config
│       └── *.md         # Blog posts
├── sass/                # Sass stylesheets
│   ├── styles.scss      # Main entry point
│   ├── _variables.scss  # Design tokens
│   ├── _base.scss       # Reset and typography
│   ├── _components.scss # UI components
│   ├── _layout.scss     # Page layouts
│   └── _utilities.scss  # Utility classes
├── static/              # Static assets
│   ├── favicon.svg      # Site icon
│   ├── search.js        # Search functionality
│   └── images/          # Images
├── templates/           # Tera templates
│   ├── base.html        # Base layout
│   ├── index.html       # Homepage
│   ├── section.html     # Section listing
│   ├── page.html        # Single page
│   ├── 404.html         # Error page
│   └── partials/        # Reusable components
└── themes/              # Optional themes
```

## Creating Content

### Blog Posts

Create a new file in `content/blog/`:

```markdown
+++
title = "My Post Title"
date = 2024-01-15
description = "A brief description for SEO."
[taxonomies]
tags = ["tutorial", "zola"]
categories = ["guides"]
+++

Your content here...

<!-- more -->

Content after the break appears on the full page only.
```

### Pages

Create a file in `content/`:

```markdown
+++
title = "Contact"
+++

Contact page content...
```

## Commands

```bash
# Start development server with live reload
zola serve

# Build for production
zola build

# Check for errors without building
zola check

# Build with drafts included
zola build --drafts
```

## Configuration

Edit `config.toml` to customize:

- `base_url` - Your production domain
- `title` - Site title
- `description` - Site description
- `taxonomies` - Tags, categories, etc.
- `[extra]` - Custom variables for templates

## Deployment

### Build Static Files

```bash
zola build
# Output in public/
```

### Docker Production

```bash
# Build production image
docker compose -f docker/compose/{{SERVICE_NAME}}.yml build

# Run production container
docker compose -f docker/compose/{{SERVICE_NAME}}.yml up -d
```

The production image uses Nginx to serve static files with:
- Gzip compression
- Long cache headers for assets
- Security headers
- Pretty URLs

## Learn More

- [Zola Documentation](https://www.getzola.org/documentation/)
- [Tera Templates](https://tera.netlify.app/docs/)
- [Zola Themes](https://www.getzola.org/themes/)
- [Shortcodes](https://www.getzola.org/documentation/content/shortcodes/)
