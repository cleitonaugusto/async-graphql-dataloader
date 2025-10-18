use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, oneshot, RwLock};
use tokio::time::{sleep, Instant};

use crate::error::DataLoaderError;
use crate::loader::BatchLoad;

type BatchResult<V> = oneshot::Sender<Result<V, DataLoaderError>>;

struct PendingRequest<K, V> {
    key: K,
    sender: BatchResult<V>,
}

pub struct Batcher<L: BatchLoad> {
    loader: Arc<L>,
    pending: Mutex<VecDeque<PendingRequest<L::Key, L::Value>>>,
    metrics: Arc<Metrics>,
    max_batch_size: usize,
    max_delay: Duration,
    batch_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

#[derive(Clone, Debug)]
pub struct Metrics {
    pub batches_dispatched: Arc<RwLock<u64>>,
    pub keys_processed: Arc<RwLock<u64>>,
    pub average_batch_size: Arc<RwLock<f64>>,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            batches_dispatched: Arc::new(RwLock::new(0)),
            keys_processed: Arc::new(RwLock::new(0)),
            average_batch_size: Arc::new(RwLock::new(0.0)),
        }
    }

    pub async fn get_stats(&self) -> BatchStats {
        BatchStats {
            batches_dispatched: *self.batches_dispatched.read().await,
            keys_processed: *self.keys_processed.read().await,
            average_batch_size: *self.average_batch_size.read().await,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BatchStats {
    pub batches_dispatched: u64,
    pub keys_processed: u64,
    pub average_batch_size: f64,
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
            pending: Mutex::new(VecDeque::new()),
            metrics: Arc::new(Metrics::new()),
            max_batch_size: 100,
            max_delay: Duration::from_millis(16), // Otimizado para performance
            batch_task: Mutex::new(None),
        }
    }

    pub fn with_max_batch_size(mut self, size: usize) -> Self {
        self.max_batch_size = size;
        self
    }

    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    pub fn metrics(&self) -> Arc<Metrics> {
        Arc::clone(&self.metrics)
    }

    pub async fn schedule(&self, key: L::Key) -> Result<L::Value, DataLoaderError> {
        let (tx, rx) = oneshot::channel();
        
        let should_start_batch = {
            let mut pending = self.pending.lock().await;
            pending.push_back(PendingRequest { key: key.clone(), sender: tx });
            
            // Lógica inteligente de batching:
            // - Se batch está cheio, processa imediatamente
            // - Se é o primeiro item, inicia task background
            pending.len() >= self.max_batch_size || pending.len() == 1
        };

        if should_start_batch {
            self.start_batch_task().await;
        }

        match rx.await {
            Ok(result) => result,
            Err(_) => Err(DataLoaderError::ChannelClosed),
        }
    }

    async fn start_batch_task(&self) {
        let mut task_guard = self.batch_task.lock().await;
        
        if task_guard.is_none() {
            let batcher = self.clone();
            let delay = self.max_delay;
            
            let handle = tokio::spawn(async move {
                // Aguarda para agrupar mais requests
                sleep(delay).await;
                batcher.process_pending().await;
            });
            
            *task_guard = Some(handle);
        }
    }

    async fn process_pending(&self) {
        let batch = {
            let mut pending = self.pending.lock().await;
            let batch_size = std::cmp::min(pending.len(), self.max_batch_size);
            pending.drain(..batch_size).collect::<Vec<_>>()
        };

        if !batch.is_empty() {
            self.process_batch(batch).await;
        }

        // Limpa a task para permitir novos batches
        *self.batch_task.lock().await = None;
    }

    async fn process_batch(&self, batch: Vec<PendingRequest<L::Key, L::Value>>) {
        let keys: Vec<L::Key> = batch.iter().map(|req| req.key.clone()).collect();
        
        if keys.is_empty() {
            return;
        }

        // Atualiza métricas
        {
            let mut metrics = self.metrics.keys_processed.write().await;
            *metrics += keys.len() as u64;
        }
        {
            let mut metrics = self.metrics.batches_dispatched.write().await;
            *metrics += 1;
        }
        {
            let mut avg_size = self.metrics.average_batch_size.write().await;
            *avg_size = (*avg_size * 0.9) + (keys.len() as f64 * 0.1); // Média móvel
        }

        let results = self.loader.load(&keys).await;

        for request in batch {
            let result = match results.get(&request.key) {
                Some(Ok(value)) => Ok(value.clone()),
                Some(Err(err)) => Err(DataLoaderError::BatchError(format!("{}", err))),
                None => Err(DataLoaderError::KeyNotFound),
            };

            let _ = request.sender.send(result);
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
            pending: Mutex::new(VecDeque::new()),
            metrics: Arc::clone(&self.metrics),
            max_batch_size: self.max_batch_size,
            max_delay: self.max_delay,
            batch_task: Mutex::new(None),
        }
    }
}
