+++
title = "Getting Started with Zola"
date = 2024-01-15
description = "A quick introduction to building static sites with Zola."
[taxonomies]
tags = ["zola", "static-site", "tutorial"]
categories = ["tutorials"]
+++

Welcome to your first Zola blog post! This article will walk you through the basics of creating content with Zola.

<!-- more -->

## Creating Content

Content in Zola is written in Markdown files with a TOML frontmatter. The frontmatter is enclosed between `+++` markers and contains metadata about the page.

```markdown
+++
title = "My Post Title"
date = 2024-01-15
description = "A brief description for SEO and previews."
[taxonomies]
tags = ["tag1", "tag2"]
+++

Your content goes here...
```

## Directory Structure

Zola uses a simple directory structure:

```
├── config.toml       # Main configuration
├── content/          # Your content (Markdown files)
│   ├── _index.md     # Homepage content
│   └── blog/         # Blog section
│       ├── _index.md # Section configuration
│       └── post.md   # A blog post
├── sass/             # Sass/SCSS stylesheets
├── static/           # Static assets (images, fonts, etc.)
└── templates/        # Tera templates
```

## Useful Commands

```bash
# Create a new site
zola init my-site

# Start development server with live reload
zola serve

# Build for production
zola build

# Check for errors without building
zola check
```

## What's Next?

- Explore the [Zola documentation](https://www.getzola.org/documentation/)
- Browse available [themes](https://www.getzola.org/themes/)
- Learn about [shortcodes](https://www.getzola.org/documentation/content/shortcodes/)
- Set up [taxonomies](https://www.getzola.org/documentation/content/taxonomies/)

Happy writing!
