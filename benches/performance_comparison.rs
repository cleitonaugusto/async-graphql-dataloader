// benches/performance_comparison.rs
use async_graphql_dataloader::{BatchLoad, DataLoader};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashMap;
use tokio::runtime::Runtime;

#[derive(Clone, Debug)]
struct TestItem {
    id: u64,
    data: String,
}

struct TestLoader {
    operation_cost: u64,
}

impl TestLoader {
    fn new(operation_cost: u64) -> Self {
        Self { operation_cost }
    }
}

#[async_trait::async_trait]
impl BatchLoad for TestLoader {
    type Key = u64;
    type Value = TestItem;
    type Error = String;

    async fn load(&self, keys: &[u64]) -> HashMap<u64, Result<TestItem, String>> {
        // Simula custo operacional
        if self.operation_cost > 0 {
            tokio::time::sleep(tokio::time::Duration::from_micros(self.operation_cost)).await;
        }

        keys.iter()
            .map(|&id| {
                let item = TestItem {
                    id,
                    data: format!("data_{}", id),
                };
                (id, Ok(item))
            })
            .collect()
    }
}

fn bench_batch_sizes(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("batch_sizes");

    for batch_size in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("requests", batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    rt.block_on(async {
                        let loader =
                            DataLoader::new(TestLoader::new(100)).with_max_batch_size(size);

                        let futures: Vec<_> = (0..size).map(|i| loader.load(i as u64)).collect();

                        let results = futures::future::join_all(futures).await;
                        criterion::black_box(results);
                    });
                });
            },
        );
    }

    group.finish();
}

// Benchmark simplificado SEM spawn de tasks
fn bench_concurrent_access(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_access");

    group.bench_function("multiple_sequential_loads", |b| {
        b.iter(|| {
            rt.block_on(async {
                let loader = DataLoader::new(TestLoader::new(50));

                // Múltiplas loads sequenciais (simula concorrência sem spawn)
                let mut results = Vec::new();
                for i in 0..100 {
                    let result = loader.load(i as u64 % 20).await; // Apenas 20 itens únicos
                    results.push(result);
                }
                criterion::black_box(results);
            });
        });
    });

    group.bench_function("parallel_loads_with_clones", |b| {
        b.iter(|| {
            rt.block_on(async {
                let loader = DataLoader::new(TestLoader::new(50));

                // Usa clones do DataLoader para tasks paralelas
                let handles: Vec<_> = (0..10) // Apenas 10 tasks para não sobrecarregar
                    .map(|i| {
                        let loader_clone = loader.clone();
                        tokio::spawn(async move { loader_clone.load(i as u64).await })
                    })
                    .collect();

                let results = futures::future::join_all(handles).await;
                criterion::black_box(results);
            });
        });
    });

    group.finish();
}

criterion_group!(benches, bench_batch_sizes, bench_concurrent_access);
criterion_main!(benches);
