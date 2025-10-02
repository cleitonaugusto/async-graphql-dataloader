// examples/axum_graphql.rs
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
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

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
    println!("🚀 Starting DataLoader example...");

    let user_loader = DataLoader::new(UserLoader);

    // Em vez de N queries, faz 1 batch query!
    let futures = vec![
        user_loader.load(1),
        user_loader.load(2),
        user_loader.load(3),
        user_loader.load(1), // Cache hit!
    ];

    let results = futures::future::join_all(futures).await;

    for result in results {
        match result {
            Ok(user) => println!("✅ User: {} - {}", user.id, user.name),
            Err(e) => println!("❌ Error: {}", e),
        }
    }

    println!("🎉 Example completed successfully!");
}
