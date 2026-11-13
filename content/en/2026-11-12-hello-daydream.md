+++
title = "Building daydream-rs: A Minimalist SSG in Rust"
slug = "hello-daydream"
date = "2026-11-12"
summary = "Why and how I built a lightweight, bilingual static site generator in Rust for my personal website."
tags = ["rust", "ssg", "web"]
draft = false
+++

Welcome to the blog section of my website, generated with **daydream-rs** — a minimalist static site generator written in Rust.

### Design Goals

When adding a blog to my existing static website, I wanted to preserve its core philosophy:

1. **Zero JavaScript**: Pure semantic HTML and classic CSS.
2. **First-class Bilingual Support**: Every post in English (`blog/<slug>.html`) automatically links to its Portuguese counterpart (`pt-br/blog/<slug>.html`) in the top banner.
3. **Minimal Dependencies**: Built on top of `pulldown-cmark`, `minijinja`, and `toml`.

```rust
fn main() {
    println!("Quod erat demonstrandum.");
}
```

> Simplicity is prerequisite for reliability. — Edsger W. Dijkstra
