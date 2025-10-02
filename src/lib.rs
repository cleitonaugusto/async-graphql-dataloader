// src/lib.rs
#![allow(unused_imports)] // Silencia warnings de imports não utilizados
mod batcher;
mod cache;
mod error;
pub mod integrations;
mod loader;
mod metrics;

pub use batcher::{BatchStats, Batcher, Metrics};
pub use cache::Cache;
pub use error::DataLoaderError;
pub use loader::{BatchLoad, DataLoader};
pub use metrics::TelemetryCollector;

// Re-exports comuns
pub use async_trait::async_trait;
