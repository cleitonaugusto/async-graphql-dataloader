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

Add this to your Cargo.toml:
[dependencies]
async-graphql-dataloader = "0.1.0"

For async-graphql integration:
[dependencies]
async-graphql-dataloader = { version = "0.1.0", features = ["graphql"] }

🚀 Quick Start
use async_graphql_dataloader::{DataLoader, Loader};
use std::collections::HashMap;

struct UserLoader;

#[async_trait::async_trait]
impl Loader<i32> for UserLoader {
    type Value = String;
    type Error = std::convert::Infallible;

    async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, Self::Value>, Self::Error> {
        let mut users = HashMap::new();
        for &key in keys {
            users.insert(key, format!("User {}", key));
        }
        Ok(users)
    }
}

#[tokio::main]
async fn main() {
    let loader = DataLoader::new(UserLoader);
    
    // Automatic batching - these will be batched into one call
    let user1 = loader.load(1).await;
    let user2 = loader.load(2).await;
    
    println!("User 1: {:?}", user1);
    println!("User 2: {:?}", user2);
}
📚 Features
✅ Automatic Batching: Multiple requests combined into single batches

✅ Intelligent Caching: Request-level caching with DashMap

✅ Async Ready: Built on Tokio async runtime

✅ Type Safe: Full Rust type safety

✅ Error Handling: Configurable error handling

✅ async-graphql Integration: Seamless integration with async-graphql

🔧 Advanced Usage
See the examples directory for more advanced usage patterns:

Basic Usage

Axum + GraphQL Integration

📖 Documentation
Full API documentation is available on docs.rs

🤝 Contributing
Contributions are welcome! Please feel free to submit pull requests or open issues.

📄 License
This project is licensed under the MIT License - see the LICENSE file for details.

Made with ❤️ and Rust