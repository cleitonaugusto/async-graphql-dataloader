// benches/minimal_bench.rs
use async_graphql_dataloader::{BatchLoad, DataLoader};
use criterion::{criterion_group, criterion_main, Criterion};
use std::collections::HashMap;

#[derive(Clone, Debug)]
struct User {
    id: u32,
    name: String,
}

struct UserLoader;

#[async_trait::async_trait]
impl BatchLoad for UserLoader {
    type Key = u32;
    type Value = User;
    type Error = String;

    async fn load(&self, keys: &[u32]) -> HashMap<u32, Result<User, String>> {
        keys.iter()
            .map(|&id| {
                (
                    id,
                    Ok(User {
                        id,
                        name: format!("User {}", id),
                    }),
                )
            })
            .collect()
    }
}

fn bench_simple(c: &mut Criterion) {
    c.bench_function("simple_load", |b| {
        b.iter(|| {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let loader = DataLoader::new(UserLoader);
                let result = loader.load(1).await;
                criterion::black_box(result);
            });
        });
    });
}

criterion_group!(benches, bench_simple);
criterion_main!(benches);
