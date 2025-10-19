use crate::batcher::Batcher;
use crate::cache::Cache;
use crate::error::DataLoaderError;
use crate::rate_limiting::RateLimiter;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

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
    batcher: Arc<Batcher<L>>,
    cache: Arc<Cache<L::Key, Result<L::Value, DataLoaderError>>>,
    max_batch_size: usize,
    delay: Duration,
    // NOVO: Rate limiting para enterprise features
    rate_limiter: Option<Arc<RateLimiter>>,
    rate_limit_key: Option<String>,
    rate_limit_max_requests: u64,
    rate_limit_window: Duration,
}

impl<L> DataLoader<L>
where
    L: BatchLoad + 'static,
    L::Key: Clone + Eq + std::hash::Hash + std::fmt::Debug,
    L::Value: Clone,
    L::Error: From<String> + std::fmt::Display,
{
    pub fn new(loader: L) -> Self {
        let loader_arc = Arc::new(loader);
        let batcher = Arc::new(Batcher::new(Arc::clone(&loader_arc)));

        Self {
            batcher,
            cache: Arc::new(Cache::new()),
            max_batch_size: 100,
            delay: Duration::from_millis(16),
            // Rate limiting desabilitado por padrão (free tier)
            rate_limiter: None,
            rate_limit_key: None,
            rate_limit_max_requests: 1000,
            rate_limit_window: Duration::from_secs(60),
        }
    }

    pub fn with_max_batch_size(mut self, size: usize) -> Self {
        self.max_batch_size = size;
        self.batcher = Arc::new(Batcher::new(Arc::clone(&self.batcher.loader))
            .with_max_batch_size(size));
        self
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self.batcher = Arc::new(Batcher::new(Arc::clone(&self.batcher.loader))
            .with_max_delay(delay));
        self
    }

    // NOVO: Métodos para rate limiting (ENTERPRISE FEATURES)
    pub fn with_rate_limiting(
        mut self,
        limiter: Arc<RateLimiter>,
        key: String
    ) -> Self {
        self.rate_limiter = Some(limiter);
        self.rate_limit_key = Some(key);
        self
    }

    pub fn with_rate_limit_config(
        mut self,
        max_requests: u64,
        window: Duration,
    ) -> Self {
        self.rate_limit_max_requests = max_requests;
        self.rate_limit_window = window;
        self
    }

    pub async fn load(&self, key: L::Key) -> Result<L::Value, DataLoaderError> {
        // NOVO: Verifica rate limiting se estiver habilitado
        if let (Some(limiter), Some(limit_key)) = (&self.rate_limiter, &self.rate_limit_key) {
            limiter
                .check_limit(
                    limit_key,
                    self.rate_limit_max_requests,
                    self.rate_limit_window
                )
                .await
                .map_err(DataLoaderError::from)?;
        }

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

    // NOVO: Método para verificar uso do rate limiting
    pub async fn get_rate_limit_usage(&self) -> Option<crate::rate_limiting::RateLimitUsage> {
        if let (Some(limiter), Some(limit_key)) = (&self.rate_limiter, &self.rate_limit_key) {
            limiter.get_usage(limit_key).await
        } else {
            None
        }
    }

    pub fn clear(&self) {
        self.cache.clear();
    }

    pub fn prime(&self, key: L::Key, value: Result<L::Value, DataLoaderError>) {
        self.cache.set(key, value);
    }

    // NOVO: Getter para métricas
    pub fn metrics(&self) -> Arc<crate::batcher::Metrics> {
        self.batcher.metrics()
    }
}

impl<L> Clone for DataLoader<L>
where
    L: BatchLoad,
{
    fn clone(&self) -> Self {
        Self {
            batcher: Arc::clone(&self.batcher),
            cache: Arc::clone(&self.cache),
            max_batch_size: self.max_batch_size,
            delay: self.delay,
            rate_limiter: self.rate_limiter.clone(),
            rate_limit_key: self.rate_limit_key.clone(),
            rate_limit_max_requests: self.rate_limit_max_requests,
            rate_limit_window: self.rate_limit_window,
        }
    }
}