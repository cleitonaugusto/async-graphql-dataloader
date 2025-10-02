# async-graphql-dataloader

🚀 **Implementação de DataLoader de alta performance para async-graphql em Rust**

[![Crates.io](https://img.shields.io/crates/v/async-graphql-dataloader)](https://crates.io/crates/async-graphql-dataloader)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.60%2B-orange.svg)](https://www.rust-lang.org)

## 🎯 **Sobre o Projeto**

**Criado e desenvolvido por: [Cleiton Augusto Correa Bezerra](https://github.com/cleitonaugusto)**

Este projeto resolve um dos problemas mais comuns em aplicações GraphQL: o **problema N+1**.

### ⚡ **Por que usar este DataLoader?**

- **🚀 Performance**: Batch loading inteligente e cache
- **🦀 Segurança**: Garantias de segurança de memória do Rust
- **⚡ Concorrência**: Async/await nativo com Tokio
- **🔧 Flexível**: Fácil integração com qualquer fonte de dados

## 📦 **Instalação**

```toml
[dependencies]
async-graphql-dataloader = "0.1.0"
