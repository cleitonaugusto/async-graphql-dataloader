// src/loader.rs
use crate::batcher::Batcher;
use crate::cache::Cache;
use crate::error::DataLoaderError;
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait BatchLoad: Send + Sync {
    type Key: Send + Sync + Clone + Eq + std::hash::Hash + std::fmt::Debug + 'static;
    type Value: Send + Sync + Clone + 'static;
    type Error: Send + Sync + From<String> + std::fmt::Display + 'static;

    async fn load(
        &self,
        keys: &[Self::Key],
    ) -> HashMap<Self::Key, Result<Self::Value, Self::Error>>;
}

pub struct DataLoader<L: BatchLoad> {
    batcher: std::sync::Arc<Batcher<L>>,
    cache: std::sync::Arc<Cache<L::Key, Result<L::Value, DataLoaderError>>>,
    max_batch_size: usize,
    delay: std::time::Duration,
}

impl<L> DataLoader<L>
where
    L: BatchLoad + 'static,
    L::Key: Clone + Eq + std::hash::Hash + std::fmt::Debug,
    L::Value: Clone,
    L::Error: From<String> + std::fmt::Display,
{
    pub fn new(loader: L) -> Self {
        let loader_arc = std::sync::Arc::new(loader);
        let batcher = std::sync::Arc::new(Batcher::new(std::sync::Arc::clone(&loader_arc)));

        Self {
            batcher,
            cache: std::sync::Arc::new(Cache::new()),
            max_batch_size: 100,
            delay: std::time::Duration::from_millis(10),
        }
    }

    pub fn with_max_batch_size(mut self, size: usize) -> Self {
        self.max_batch_size = size;
        self
    }

    pub fn with_delay(mut self, delay: std::time::Duration) -> Self {
        self.delay = delay;
        self
    }

    pub async fn load(&self, key: L::Key) -> Result<L::Value, DataLoaderError> {
        // Verifica cache primeiro
        if let Some(cached) = self.cache.get(&key) {
            return cached;
        }

        // Agenda no batcher
        let result = self.batcher.schedule(key.clone()).await;

        // Cache o resultado
        if let Ok(ref value) = result {
            self.cache.set(key.clone(), Ok(value.clone()));
        } else if let Err(ref error) = result {
            self.cache.set(key.clone(), Err(error.clone()));
        }

        result
    }

    pub fn clear(&self) {
        self.cache.clear();
    }

    pub fn prime(&self, key: L::Key, value: Result<L::Value, DataLoaderError>) {
        self.cache.set(key, value);
    }
}

impl<L> Clone for DataLoader<L>
where
    L: BatchLoad,
{
    fn clone(&self) -> Self {
        Self {
            batcher: std::sync::Arc::clone(&self.batcher),
            cache: std::sync::Arc::clone(&self.cache),
            max_batch_size: self.max_batch_size,
            delay: self.delay,
        }
    }
}
