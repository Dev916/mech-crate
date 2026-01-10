+++
title = "Markdown Showcase"
date = 2024-01-10
description = "A comprehensive guide to Markdown features supported by Zola."
[taxonomies]
tags = ["markdown", "reference"]
categories = ["tutorials"]
+++

This post demonstrates all the Markdown features available in Zola.

<!-- more -->

## Headings

Headings from h1 to h6 are supported. Use `#` symbols:

```markdown
# Heading 1
## Heading 2
### Heading 3
```

## Emphasis

You can make text **bold**, *italic*, or ***both***. ~~Strikethrough~~ is also supported.

```markdown
**bold**
*italic*
***both***
~~strikethrough~~
```

## Lists

### Unordered Lists

- Item one
- Item two
  - Nested item
  - Another nested item
- Item three

### Ordered Lists

1. First item
2. Second item
   1. Nested numbered item
   2. Another nested item
3. Third item

### Task Lists

- [x] Completed task
- [ ] Incomplete task
- [ ] Another task

## Links and Images

[Link to Zola](https://www.getzola.org/)

![Placeholder image](/images/placeholder.jpg "Image title")

## Blockquotes

> This is a blockquote. It can span
> multiple lines and contains wisdom.
>
> — Someone Famous

## Code

Inline `code` uses backticks.

Code blocks use triple backticks with optional language highlighting:

```rust
fn main() {
    println!("Hello, Zola!");
    
    let numbers: Vec<i32> = (1..=5).collect();
    for n in numbers {
        println!("Number: {}", n);
    }
}
```

```javascript
// JavaScript example
const greet = (name) => {
  console.log(`Hello, ${name}!`);
};

greet('World');
```

## Tables

| Feature | Supported | Notes |
|---------|-----------|-------|
| Markdown | ✓ | Full CommonMark support |
| Sass | ✓ | Compiled automatically |
| Syntax Highlighting | ✓ | 100+ themes |
| Search | ✓ | Static search index |

## Horizontal Rule

---

## Footnotes

Here's a sentence with a footnote[^1].

[^1]: This is the footnote content.

## Definition Lists

Zola
: A fast static site generator written in Rust.

Tera
: The templating engine used by Zola.

## Summary

Zola's Markdown support is comprehensive and includes all standard features plus extensions like task lists, footnotes, and syntax highlighting with over 100 themes.
