#![allow(dead_code)] // Silencia warnings de código não utilizado
use async_graphql_dataloader::{DataLoader, BatchLoad};
use std::collections::HashMap;
use std::time::Instant;

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
        println!("🔥 BATCH EXECUTED - Loading {} users: {:?}", keys.len(), keys);
        
        keys.iter()
            .map(|&id| {
                (id, Ok(User {
                    id,
                    name: format!("User {}", id),
                }))
            })
            .collect()
    }
}

#[tokio::main]
async fn main() {
    println!("🚀 STARTING DEBUG EXAMPLE...");
    let start = Instant::now();

    let user_loader = DataLoader::new(UserLoader)
        .with_delay(std::time::Duration::from_millis(1)) // Delay muito curto
        .with_max_batch_size(10);

    println!("1. Loading single user...");
    let result1 = user_loader.load(1).await;
    println!("   Result 1: {:?}", result1);
    
    println!("2. Loading multiple users...");
    let result2 = user_loader.load(2).await;
    let result3 = user_loader.load(3).await;
    println!("   Result 2: {:?}", result2);
    println!("   Result 3: {:?}", result3);

    println!("3. Testing cache...");
    let cached = user_loader.load(1).await;
    println!("   Cached: {:?}", cached);

    let duration = start.elapsed();
    println!("🎉 DEBUG COMPLETED in {:?}", duration);
}
