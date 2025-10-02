# async-graphql-dataloader

🚀 **High-performance DataLoader implementation for async-graphql in Rust**

[![Crates.io](https://img.shields.io/crates/v/async-graphql-dataloader)](https://crates.io/crates/async-graphql-dataloader)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.60%2B-orange.svg)](https://www.rust-lang.org)

## 🎯 **About the Project**

**Created and developed by: [Cleiton Augusto Correa Bezerra](https://github.com/cleitonaugusto)**

This project solves one of the most common problems in GraphQL applications: the **N+1 problem**.

### ⚡ **Why use this DataLoader?**

- **🚀 Performance**: Intelligent batch loading and caching
- **🦀 Safety**: Rust's memory safety guarantees
- **⚡ Concurrency**: Native async/await with Tokio
- **🔧 Flexible**: Easy integration with any data source

## 📦 **Installation**

```toml
[dependencies]
async-graphql-dataloader = "0.1.0"
