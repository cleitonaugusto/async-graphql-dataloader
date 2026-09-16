//! # async-graphql-dataloader
//!
//! A DataLoader implementation for Rust with automatic request batching and
//! per-request caching, built on Tokio.
//!
//! It addresses the N+1 query problem: instead of issuing one backend call per
//! field resolution, concurrent `load` calls made within a short time window are
//! coalesced into a single batched call to your data source.
//!
//! ## Relationship to `async-graphql`
//!
//! [`async-graphql`] ships its own DataLoader behind its `dataloader` feature,
//! and for most projects that is the right default. This crate is a standalone
//! alternative that also works outside of GraphQL — any batchable key/value
//! lookup benefits from it.
//!
//! [`async-graphql`]: https://docs.rs/async-graphql
//!
//! ## Example
//!
//! ```
//! use async_graphql_dataloader::{BatchLoad, DataLoader};
//! use std::collections::HashMap;
//!
//! struct UserLoader;
//!
//! #[async_trait::async_trait]
//! impl BatchLoad for UserLoader {
//!     type Key = i32;
//!     type Value = String;
//!     type Error = String;
//!
//!     async fn load(&self, keys: &[i32]) -> HashMap<i32, Result<String, String>> {
//!         // One call to your database for the whole batch.
//!         keys.iter().map(|&k| (k, Ok(format!("User {}", k)))).collect()
//!     }
//! }
//!
//! # #[tokio::main]
//! # async fn main() {
//! let loader = DataLoader::new(UserLoader);
//!
//! // These two are coalesced into a single `load` call.
//! let (a, b) = tokio::join!(loader.load(1), loader.load(2));
//! assert_eq!(a.unwrap(), "User 1");
//! assert_eq!(b.unwrap(), "User 2");
//! # }
//! ```

mod batcher;
mod cache;
mod error;
pub mod integrations;
mod loader;
mod metrics;
mod query_cost;
mod rate_limiting;

pub use batcher::{BatchStats, Batcher, Metrics};
pub use cache::Cache;
pub use error::{DataLoaderError, RateLimitError};
pub use loader::{BatchLoad, DataLoader};
pub use metrics::{DataLoaderMetrics, TelemetryCollector};
pub use query_cost::{
    CostAnalytics, CostHistoryEntry, CostRule, QueryCost, QueryCostAnalyzer, QueryCostError,
};
pub use rate_limiting::{RateLimitUsage, RateLimiter};

pub use async_trait::async_trait;

/// Builds a [`DataLoader`], optionally configuring batch size and delay.
///
/// ```
/// # use async_graphql_dataloader::{dataloader, BatchLoad, DataLoader};
/// # use std::collections::HashMap;
/// # struct L;
/// # #[async_trait::async_trait]
/// # impl BatchLoad for L {
/// #     type Key = i32; type Value = i32; type Error = String;
/// #     async fn load(&self, k: &[i32]) -> HashMap<i32, Result<i32, String>> {
/// #         k.iter().map(|&k| (k, Ok(k))).collect()
/// #     }
/// # }
/// let loader = dataloader!(L, 50, 10); // max 50 keys per batch, 10ms window
/// ```
#[macro_export]
macro_rules! dataloader {
    ($loader:expr) => {
        $crate::DataLoader::new($loader)
    };
    ($loader:expr, $batch_size:expr) => {
        $crate::DataLoader::new($loader).with_max_batch_size($batch_size)
    };
    ($loader:expr, $batch_size:expr, $delay_ms:expr) => {
        $crate::DataLoader::new($loader)
            .with_max_batch_size($batch_size)
            .with_delay(std::time::Duration::from_millis($delay_ms))
    };
}

impl<L> std::fmt::Debug for DataLoader<L>
where
    L: BatchLoad,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataLoader")
            .field("max_batch_size", &self.max_batch_size)
            .field("delay_ms", &self.delay.as_millis())
            .field("has_rate_limiting", &self.rate_limiter.is_some())
            .field(
                "has_query_cost_analysis",
                &self.query_cost_analyzer.is_some(),
            )
            .finish()
    }
}

pub mod prelude {
    pub use super::{
        async_trait, BatchLoad, DataLoader, DataLoaderError, QueryCost, QueryCostAnalyzer,
        RateLimiter,
    };
}
