#![allow(unused_imports)] // Silencia warnings de imports não utilizados
mod batcher;
mod cache;
mod error;
pub mod integrations;
mod loader;
mod metrics;
mod rate_limiting;

pub use batcher::{BatchStats, Batcher, Metrics};
pub use cache::Cache;
pub use error::{DataLoaderError, RateLimitError}; // ATUALIZADO: RateLimitError vem do error.rs
pub use loader::{BatchLoad, DataLoader};
pub use metrics::TelemetryCollector;
pub use rate_limiting::{RateLimiter, RateLimitUsage}; // ATUALIZADO: removemos RateLimitError daqui

// Re-exports comuns
pub use async_trait::async_trait;