// src/metrics.rs
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize)]
pub struct DataLoaderMetrics {
    pub total_requests: u64,
    pub batch_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub average_batch_size: f64,
    pub error_count: u64,
    pub success_rate: f64,
}

#[derive(Clone)]
pub struct TelemetryCollector {
    metrics: Arc<RwLock<InternalMetrics>>,
}

struct InternalMetrics {
    total_requests: u64,
    batch_requests: u64,
    cache_hits: u64,
    cache_misses: u64,
    batch_sizes: Vec<usize>,
    errors: u64,
}

impl TelemetryCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(InternalMetrics {
                total_requests: 0,
                batch_requests: 0,
                cache_hits: 0,
                cache_misses: 0,
                batch_sizes: Vec::new(),
                errors: 0,
            })),
        }
    }

    pub async fn record_request(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;
    }

    pub async fn record_batch(&self, batch_size: usize) {
        let mut metrics = self.metrics.write().await;
        metrics.batch_requests += 1;
        metrics.batch_sizes.push(batch_size);

        if metrics.batch_sizes.len() > 1000 {
            metrics.batch_sizes.remove(0);
        }
    }

    pub async fn get_metrics(&self) -> DataLoaderMetrics {
        let metrics = self.metrics.read().await;

        let total_requests = metrics.total_requests;
        let success_count = total_requests - metrics.errors;
        let success_rate = if total_requests > 0 {
            (success_count as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        let average_batch_size = if !metrics.batch_sizes.is_empty() {
            metrics.batch_sizes.iter().sum::<usize>() as f64 / metrics.batch_sizes.len() as f64
        } else {
            0.0
        };

        DataLoaderMetrics {
            total_requests,
            batch_requests: metrics.batch_requests,
            cache_hits: metrics.cache_hits,
            cache_misses: metrics.cache_misses,
            average_batch_size,
            error_count: metrics.errors,
            success_rate,
        }
    }
}
