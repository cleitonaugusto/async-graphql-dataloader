use async_graphql_dataloader::{BatchLoad, DataLoader};
use std::collections::HashMap;

#[derive(Clone, Debug)]
struct User {
    id: i32,
    name: String,
}

struct UserLoader;

#[async_trait::async_trait]
impl BatchLoad for UserLoader {
    type Key = i32;
    type Value = User;
    type Error = String;

    async fn load(&self, keys: &[i32]) -> HashMap<i32, Result<User, String>> {
        println!("🚀 BATCH LOADING {} users: {:?}", keys.len(), keys);

        // Simula latência de banco de dados
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        keys.iter()
            .map(|&id| {
                let user = User {
                    id,
                    name: format!("User {}", id),
                };
                (id, Ok(user))
            })
            .collect()
    }
}

#[tokio::main]
async fn main() {
    println!("🚀 Starting BASIC DataLoader example...");

    let user_loader = DataLoader::new(UserLoader);

    // Teste 1: Batch loading
    println!("📦 Testing batch loading...");
    let futures = vec![
        user_loader.load(1),
        user_loader.load(2),
        user_loader.load(3),
    ];

    let results = futures::future::join_all(futures).await;

    for result in results {
        match result {
            Ok(user) => println!("✅ User: {} - {}", user.id, user.name),
            Err(e) => println!("❌ Error: {}", e),
        }
    }

    // Teste 2: Cache
    println!("💾 Testing cache...");
    let cached_result = user_loader.load(1).await;
    match cached_result {
        Ok(user) => println!("✅ Cached User 1: {}", user.name),
        Err(e) => println!("❌ Cached Error: {}", e),
    }

    // Teste 3: Clear cache
    println!("🗑️ Testing cache clear...");
    user_loader.clear();
    let after_clear = user_loader.load(1).await;
    match after_clear {
        Ok(user) => println!("✅ After clear User 1: {}", user.name),
        Err(e) => println!("❌ After clear Error: {}", e),
    }

    println!("🎉 BASIC example completed successfully!");
}
