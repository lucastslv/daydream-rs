+++
title = "Construindo o daydream-rs: Um SSG Minimalista em Rust"
slug = "hello-daydream"
date = "2026-11-12"
summary = "Por que e como construí um gerador de sites estáticos leve e bilíngue em Rust para o meu site pessoal."
tags = ["rust", "ssg", "web"]
draft = false
+++

Bem-vindo à seção de blog do meu site, gerada pelo **daydream-rs** — um gerador de sites estáticos minimalista escrito em Rust.

### Objetivos de Projeto

Ao adicionar um blog ao meu site estático, quis preservar sua filosofia original:

1. **Zero JavaScript**: HTML semântico puro e CSS clássico.
2. **Suporte Bilíngue Nativo**: Cada artigo em português (`pt-br/blog/<slug>.html`) conecta-se automaticamente à sua versão em inglês (`blog/<slug>.html`) pelo seletor de idioma no topo.
3. **Dependências Mínimas**: Construído com `pulldown-cmark`, `minijinja` e `toml`.

```rust
fn main() {
    println!("Quod erat demonstrandum.");
}
```

> A simplicidade é um pré-requisito para a confiabilidade. — Edsger W. Dijkstra
