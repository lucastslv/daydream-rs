# Daydream

A minimalist static blog generator written in Rust.

`daydream-rs` is a small personal tool built to generate the bilingual (`en` and `pt-BR`) blog section of my website ([lucastslv.com](https://lucastslv.com)), keeping its classic static HTML/CSS structure.

## Installation

```bash
cargo install --path .
```

## Usage

1. Copy [`daydream.example.toml`](daydream.example.toml) to `daydream.toml` inside your website directory and adjust your metadata.
2. Create a new post (generates Markdown skeletons in `content/en/` and `content/pt-br/`):
   ```bash
   daydream new my-first-post --title "My First Post"
   ```
   *(Use `--lang en` or `--lang pt-br` to create a single-language post.)*
3. Build the HTML pages (`blog.html`, `blog/<slug>.html`, and their `pt-br/` counterparts):
   ```bash
   daydream build
   ```
   *(Pass `--drafts` to include posts marked with `draft = true`.)*

## Post Format

Posts use TOML frontmatter delimited by `+++` followed by standard Markdown:

```markdown
+++
title = "Hello World"
slug = "hello-world"
date = "2026-11-12"
summary = "Short description of the post."
tags = ["rust", "notes"]
draft = false
+++

Write your content here...
```

Posts sharing the same `slug` across `content/en/` and `content/pt-br/` are automatically linked together via the language switcher.
