use crate::batcher::Batcher;
use crate::cache::Cache;
use crate::error::DataLoaderError;
use crate::rate_limiting::RateLimiter;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

// NOVO: Import para Query Cost Analysis
use crate::query_cost::{QueryCostAnalyzer, QueryCostError};

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
    pub(crate) max_batch_size: usize,
    pub(crate) delay: Duration,
    // Rate limiting para enterprise features
    pub(crate) rate_limiter: Option<Arc<RateLimiter>>,
    rate_limit_key: Option<String>,
    rate_limit_max_requests: u64,
    rate_limit_window: Duration,
    // NOVO: Query Cost Analysis (FEATURE PREMIUM!)
    pub(crate) query_cost_analyzer: Option<Arc<QueryCostAnalyzer>>,
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
            // Query Cost Analysis desabilitado por padrão
            query_cost_analyzer: None,
        }
    }

    pub fn with_max_batch_size(mut self, size: usize) -> Self {
        self.max_batch_size = size;
        // CORREÇÃO: Criar novo batcher com o tamanho correto
        let new_batcher = Batcher::new(Arc::clone(&self.batcher.loader))
            .with_max_batch_size(size);
        self.batcher = Arc::new(new_batcher);
        self
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        // CORREÇÃO: Criar novo batcher com o delay correto
        let new_batcher = Batcher::new(Arc::clone(&self.batcher.loader))
            .with_max_delay(delay);
        self.batcher = Arc::new(new_batcher);
        self
    }

    // Métodos para rate limiting (ENTERPRISE FEATURES)
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

    // NOVO: Query Cost Analysis (ENTERPRISE PREMIUM!)
    pub fn with_query_cost_analysis(mut self, analyzer: Arc<QueryCostAnalyzer>) -> Self {
        self.query_cost_analyzer = Some(analyzer);
        self
    }

    pub async fn load(&self, key: L::Key) -> Result<L::Value, DataLoaderError> {
        // CORREÇÃO: Verifica rate limiting corretamente
        if let (Some(limiter), Some(limit_key)) = (&self.rate_limiter, &self.rate_limit_key) {
            if let Err(e) = limiter.check_limit(
                limit_key,
                self.rate_limit_max_requests,
                self.rate_limit_window
            ).await {
                return Err(DataLoaderError::from(e));
            }
        }

        // CORREÇÃO: Verifica custo da query corretamente
        if let Some(analyzer) = &self.query_cost_analyzer {
            if let Err(e) = analyzer.calculate_cost("load", 1, None).await {
                return Err(DataLoaderError::from(e));
            }
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

    // NOVO: Método load_many com Query Cost Analysis
    pub async fn load_many(&self, keys: Vec<L::Key>) -> Vec<Result<L::Value, DataLoaderError>> {
        // CORREÇÃO: Usar clone() para os erros
        if let (Some(limiter), Some(limit_key)) = (&self.rate_limiter, &self.rate_limit_key) {
            if let Err(e) = limiter.check_limit(
                limit_key,
                self.rate_limit_max_requests,
                self.rate_limit_window
            ).await {
                // Se rate limit excedido, retorna erro para todas as keys
                return keys.into_iter()
                    .map(|_| Err(DataLoaderError::from(e.clone()))) // ⬅️ CORREÇÃO: e.clone()
                    .collect();
            }
        }

        // CORREÇÃO: Usar clone() para os erros
        if let Some(analyzer) = &self.query_cost_analyzer {
            if let Err(e) = analyzer.calculate_cost("load_many", keys.len(), None).await {
                // Se custo excedido, retorna erro para todas as keys
                return keys.into_iter()
                    .map(|_| Err(DataLoaderError::from(e.clone()))) // ⬅️ CORREÇÃO: e.clone()
                    .collect();
            }
        }

        // Processa normalmente
        let mut results = Vec::new();
        for key in keys {
            results.push(self.load(key).await);
        }
        results
    }

    // NOVO: Método para carregamento em lote com custo otimizado
    pub async fn load_batch(&self, keys: Vec<L::Key>) -> Result<Vec<L::Value>, DataLoaderError> {
        // CORREÇÃO: Usar clone() para os erros
        if let (Some(limiter), Some(limit_key)) = (&self.rate_limiter, &self.rate_limit_key) {
            if let Err(e) = limiter.check_limit(
                limit_key,
                self.rate_limit_max_requests,
                self.rate_limit_window
            ).await {
                return Err(DataLoaderError::from(e.clone())); // ⬅️ CORREÇÃO: e.clone()
            }
        }

        // CORREÇÃO: Usar clone() para os erros
        if let Some(analyzer) = &self.query_cost_analyzer {
            if let Err(e) = analyzer.calculate_cost("load_batch", keys.len(), None).await {
                return Err(DataLoaderError::from(e.clone())); // ⬅️ CORREÇÃO: e.clone()
            }
        }

        // CORREÇÃO: Acessa o loader através de um método público ou trait
        // Como não temos acesso direto, vamos usar o batcher para cada key individualmente
        let mut values = Vec::new();
        let mut errors = Vec::new();

        for key in &keys {
            match self.batcher.schedule(key.clone()).await {
                Ok(value) => {
                    // CORREÇÃO: Clonar value antes de usar duas vezes
                    let value_clone = value.clone(); // ⬅️ CORREÇÃO: Clonar primeiro
                    values.push(value);
                    self.cache.set(key.clone(), Ok(value_clone)); // ⬅️ CORREÇÃO: Usar clone
                }
                Err(e) => {
                    errors.push((key.clone(), e));
                }
            }
        }

        // Se houve algum erro, retorna o primeiro
        if let Some((key, error)) = errors.into_iter().next() {
            self.cache.set(key, Err(error.clone()));
            Err(error)
        } else {
            Ok(values)
        }
    }

    // Método para verificar uso do rate limiting
    pub async fn get_rate_limit_usage(&self) -> Option<crate::rate_limiting::RateLimitUsage> {
        if let (Some(limiter), Some(limit_key)) = (&self.rate_limiter, &self.rate_limit_key) {
            limiter.get_usage(limit_key).await
        } else {
            None
        }
    }

    // NOVO: Método para verificar custo de uma query sem executá-la
    pub async fn estimate_cost(&self, operation: &str, item_count: usize) -> Result<crate::query_cost::QueryCost, QueryCostError> {
        if let Some(analyzer) = &self.query_cost_analyzer {
            analyzer.calculate_cost(operation, item_count, None).await
        } else {
            // Se não há analyzer, retorna custo zero (free tier)
            Ok(crate::query_cost::QueryCost {
                total_cost: 0,
                breakdown: HashMap::from([(operation.to_string(), 0)]),
                timestamp: std::time::SystemTime::now(),
                operation: operation.to_string(),
            })
        }
    }

    pub fn clear(&self) {
        self.cache.clear();
    }

    pub fn prime(&self, key: L::Key, value: Result<L::Value, DataLoaderError>) {
        self.cache.set(key, value);
    }

    // Getter para métricas
    pub fn metrics(&self) -> Arc<crate::batcher::Metrics> {
        self.batcher.metrics()
    }

    // NOVO: Getter para verificar se Query Cost Analysis está ativo
    pub fn has_query_cost_analysis(&self) -> bool {
        self.query_cost_analyzer.is_some()
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
            query_cost_analyzer: self.query_cost_analyzer.clone(),
        }
    }
}
