// src/batcher.rs
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::{Mutex, oneshot, RwLock};
use crate::loader::BatchLoad;
use crate::error::DataLoaderError;

type BatchResult<V> = oneshot::Sender<Result<V, DataLoaderError>>;
type PendingBatch<K, V> = HashMap<K, Vec<BatchResult<V>>>;

pub struct Batcher<L: BatchLoad> {
    loader: Arc<L>,
    pending: Mutex<PendingBatch<L::Key, L::Value>>,
    metrics: Arc<Metrics>,
}

#[derive(Clone, Debug)]
pub struct Metrics {
    pub batches_dispatched: Arc<RwLock<u64>>,
    pub keys_processed: Arc<RwLock<u64>>,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            batches_dispatched: Arc::new(RwLock::new(0)),
            keys_processed: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn get_stats(&self) -> BatchStats {
        BatchStats {
            batches_dispatched: *self.batches_dispatched.read().await,
            keys_processed: *self.keys_processed.read().await,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BatchStats {
    pub batches_dispatched: u64,
    pub keys_processed: u64,
}

impl<L> Batcher<L> 
where 
    L: BatchLoad + 'static,
    L::Key: Clone + Eq + std::hash::Hash + std::fmt::Debug,
    L::Value: Clone,
    L::Error: From<String> + std::fmt::Display,
{
    pub fn new(loader: Arc<L>) -> Self {
        Self {
            loader,
            pending: Mutex::new(HashMap::new()),
            metrics: Arc::new(Metrics::new()),
        }
    }

    pub fn metrics(&self) -> Arc<Metrics> {
        Arc::clone(&self.metrics)
    }

    pub async fn schedule(&self, key: L::Key) -> Result<L::Value, DataLoaderError> {
        let (tx, rx) = oneshot::channel();
        
        let should_dispatch = {
            let mut pending = self.pending.lock().await;
            let entry = pending.entry(key.clone()).or_insert_with(Vec::new);
            entry.push(tx);
            
            // Dispara imediatamente para batches pequenos (REMOVEMOS O DELAY)
            true // SEMPRE dispara imediatamente para teste
        };

        if should_dispatch {
            self.dispatch_batch_for_key(key.clone()).await;
        }

        match rx.await {
            Ok(result) => result,
            Err(_) => Err(DataLoaderError::ChannelClosed),
        }
    }

    async fn dispatch_batch_for_key(&self, key: L::Key) {
        let batch = {
            let mut pending = self.pending.lock().await;
            pending.remove(&key).map(|senders| {
                vec![(key, senders)]
            }).unwrap_or_else(Vec::new)
        };

        if !batch.is_empty() {
            self.process_batch(batch).await;
        }
    }

    async fn process_batch(&self, batch: Vec<(L::Key, Vec<BatchResult<L::Value>>)>) {
        let keys: Vec<L::Key> = batch.iter().map(|(key, _)| key.clone()).collect();
        
        if keys.is_empty() {
            return;
        }

        *self.metrics.keys_processed.write().await += keys.len() as u64;
        *self.metrics.batches_dispatched.write().await += 1;

        let results = self.loader.load(&keys).await;

        for (key, senders) in batch {
            let result = match results.get(&key) {
                Some(Ok(value)) => Ok(value.clone()),
                Some(Err(err)) => Err(DataLoaderError::BatchError(format!("{}", err))),
                None => Err(DataLoaderError::KeyNotFound),
            };

            for sender in senders {
                let _ = sender.send(result.clone());
            }
        }
    }
}

impl<L> Clone for Batcher<L>
where
    L: BatchLoad,
{
    fn clone(&self) -> Self {
        Self {
            loader: Arc::clone(&self.loader),
            pending: Mutex::new(HashMap::new()),
            metrics: Arc::clone(&self.metrics),
        }
    }
}