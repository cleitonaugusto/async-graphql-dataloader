// benches/real_world_benchmark.rs
use async_graphql_dataloader::{BatchLoad, DataLoader};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashMap;
use tokio::runtime::Runtime;

// Simula cenários reais
#[derive(Clone, Debug)]
struct Product {
    id: String,
    name: String,
    price: f64,
}

struct ProductLoader {
    latency_ms: u64,
}

impl ProductLoader {
    fn new(latency_ms: u64) -> Self {
        Self { latency_ms }
    }
}

#[async_trait::async_trait]
impl BatchLoad for ProductLoader {
    type Key = String;
    type Value = Product;
    type Error = String;

    async fn load(&self, keys: &[String]) -> HashMap<String, Result<Product, String>> {
        // Simula latência de banco de dados real
        tokio::time::sleep(tokio::time::Duration::from_millis(self.latency_ms)).await;

        keys.iter()
            .map(|key| {
                let product = Product {
                    id: key.clone(),
                    name: format!("Product {}", key),
                    price: key.parse::<f64>().unwrap_or(10.0) * 2.5,
                };
                (key.clone(), Ok(product))
            })
            .collect()
    }
}

// Benchmark para diferentes cenários
fn bench_dataloader_scenarios(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("dataloader_scenarios");

    // Cenário 1: Poucas requests com alta latência
    group.bench_function("10_requests_50ms_latency", |b| {
        b.iter(|| {
            rt.block_on(async {
                let loader = DataLoader::new(ProductLoader::new(50));
                let futures: Vec<_> = (0..10).map(|i| loader.load(i.to_string())).collect();

                let results = futures::future::join_all(futures).await;
                criterion::black_box(results);
            });
        });
    });

    // Cenário 2: Muitas requests com baixa latência
    group.bench_function("1000_requests_5ms_latency", |b| {
        b.iter(|| {
            rt.block_on(async {
                let loader = DataLoader::new(ProductLoader::new(5));
                let futures: Vec<_> = (0..1000)
                    .map(|i| loader.load((i % 100).to_string())) // 100 produtos únicos
                    .collect();

                let results = futures::future::join_all(futures).await;
                criterion::black_box(results);
            });
        });
    });

    // Cenário 3: Cache performance (requests repetidas)
    group.bench_function("cache_performance_100_repeats", |b| {
        b.iter(|| {
            rt.block_on(async {
                let loader = DataLoader::new(ProductLoader::new(20));

                // Primeiro carregamento
                let first_load = loader.load("1".to_string()).await;
                criterion::black_box(first_load);

                // 100 requests para o mesmo item (deve usar cache)
                for _ in 0..100 {
                    let result = loader.load("1".to_string()).await;
                    criterion::black_box(result);
                }
            });
        });
    });

    group.finish();
}

// Benchmark comparativo: Com vs Sem DataLoader
fn bench_with_vs_without_dataloader(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("with_vs_without_dataloader");

    // SEM DataLoader (N+1 problem)
    group.bench_function("without_dataloader_100_items", |b| {
        b.iter(|| {
            rt.block_on(async {
                let loader = ProductLoader::new(10);
                let mut results = Vec::new();

                for i in 0..100 {
                    let result = loader.load(&[i.to_string()]).await;
                    results.push(result); // Mova para o vetor
                    criterion::black_box(&results); // Use borrow do vetor
                }
            });
        });
    });

    // COM DataLoader
    group.bench_function("with_dataloader_100_items", |b| {
        b.iter(|| {
            rt.block_on(async {
                let loader = DataLoader::new(ProductLoader::new(10));
                let futures: Vec<_> = (0..100).map(|i| loader.load(i.to_string())).collect();

                let results = futures::future::join_all(futures).await;
                criterion::black_box(results);
            });
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_dataloader_scenarios,
    bench_with_vs_without_dataloader
);
criterion_main!(benches);
