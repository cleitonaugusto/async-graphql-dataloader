//! Regression tests for batching, caching and dispatch behaviour.
//!
//! Every test that awaits a `load` wraps it in a timeout: prior to 0.2.0 the
//! batcher dispatched against a cloned, empty queue, so `load` never resolved
//! and a plain `await` here would hang the whole test run instead of failing.

use async_graphql_dataloader::{BatchLoad, DataLoader};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(5);

/// Records how many times `load` was invoked and with which key sets.
struct RecordingLoader {
    calls: Arc<AtomicUsize>,
    batches: Arc<Mutex<Vec<Vec<i32>>>>,
}

impl RecordingLoader {
    fn new() -> (Self, Arc<AtomicUsize>, Arc<Mutex<Vec<Vec<i32>>>>) {
        let calls = Arc::new(AtomicUsize::new(0));
        let batches = Arc::new(Mutex::new(Vec::new()));
        let loader = Self {
            calls: Arc::clone(&calls),
            batches: Arc::clone(&batches),
        };
        (loader, calls, batches)
    }
}

#[async_trait::async_trait]
impl BatchLoad for RecordingLoader {
    type Key = i32;
    type Value = String;
    type Error = String;

    async fn load(&self, keys: &[i32]) -> HashMap<i32, Result<String, String>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.batches.lock().unwrap().push(keys.to_vec());
        keys.iter().map(|&k| (k, Ok(format!("v{}", k)))).collect()
    }
}

#[tokio::test]
async fn single_load_resolves() {
    let (loader, calls, _) = RecordingLoader::new();
    let dl = DataLoader::new(loader);

    let value = tokio::time::timeout(TIMEOUT, dl.load(1))
        .await
        .expect("load did not resolve — the batcher is not dispatching")
        .expect("load returned an error");

    assert_eq!(value, "v1");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn concurrent_loads_are_coalesced_into_one_call() {
    let (loader, calls, batches) = RecordingLoader::new();
    let dl = Arc::new(DataLoader::new(loader));

    let handles: Vec<_> = (1..=5)
        .map(|k| {
            let dl = Arc::clone(&dl);
            tokio::spawn(async move { dl.load(k).await })
        })
        .collect();

    for h in handles {
        let r = tokio::time::timeout(TIMEOUT, h)
            .await
            .expect("a concurrent load did not resolve")
            .expect("task panicked");
        assert!(r.is_ok());
    }

    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "5 concurrent loads should produce exactly 1 batch, got batches: {:?}",
        batches.lock().unwrap()
    );

    let mut keys = batches.lock().unwrap()[0].clone();
    keys.sort();
    assert_eq!(keys, vec![1, 2, 3, 4, 5]);
}

#[tokio::test]
async fn repeated_key_is_served_from_cache() {
    let (loader, calls, _) = RecordingLoader::new();
    let dl = DataLoader::new(loader);

    let first = tokio::time::timeout(TIMEOUT, dl.load(7)).await.unwrap();
    let second = tokio::time::timeout(TIMEOUT, dl.load(7)).await.unwrap();

    assert_eq!(first.unwrap(), "v7");
    assert_eq!(second.unwrap(), "v7");
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "the second load of the same key must hit the cache"
    );
}

#[tokio::test]
async fn batch_is_split_at_max_batch_size() {
    let (loader, calls, batches) = RecordingLoader::new();
    let dl = Arc::new(DataLoader::new(loader).with_max_batch_size(3));

    let handles: Vec<_> = (1..=7)
        .map(|k| {
            let dl = Arc::clone(&dl);
            tokio::spawn(async move { dl.load(k).await })
        })
        .collect();

    for h in handles {
        tokio::time::timeout(TIMEOUT, h)
            .await
            .expect("a load did not resolve")
            .expect("task panicked")
            .expect("load returned an error");
    }

    let batches = batches.lock().unwrap();
    assert!(
        batches.iter().all(|b| b.len() <= 3),
        "no batch may exceed max_batch_size, got: {:?}",
        batches
    );

    let total: usize = batches.iter().map(|b| b.len()).sum();
    assert_eq!(total, 7, "every key must be dispatched exactly once");
    assert!(calls.load(Ordering::SeqCst) >= 3);
}

#[tokio::test]
async fn load_many_resolves_every_key() {
    let (loader, _, _) = RecordingLoader::new();
    let dl = DataLoader::new(loader);

    let results = tokio::time::timeout(TIMEOUT, dl.load_many(vec![1, 2, 3]))
        .await
        .expect("load_many did not resolve");

    assert_eq!(results.len(), 3);
    for (i, r) in results.into_iter().enumerate() {
        assert_eq!(r.expect("load_many returned an error"), format!("v{}", i + 1));
    }
}
