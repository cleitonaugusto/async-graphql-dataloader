use std::fmt;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum DataLoaderError {
    ChannelClosed,
    BatchError(String),
    KeyNotFound,
    Timeout,
    RateLimitExceeded(String),
    QueryCostExceeded(String),
}

impl fmt::Display for DataLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataLoaderError::ChannelClosed => {
                write!(f, "Channel closed while waiting for batch result")
            }
            DataLoaderError::BatchError(msg) => write!(f, "Batch load error: {}", msg),
            DataLoaderError::KeyNotFound => write!(f, "Key not found in batch results"),
            DataLoaderError::Timeout => write!(f, "Timeout waiting for batch"),
            DataLoaderError::RateLimitExceeded(details) => write!(f, "Rate limit exceeded: {}", details),
            DataLoaderError::QueryCostExceeded(details) => write!(f, "Query cost exceeded: {}", details),
        }
    }
}

impl std::error::Error for DataLoaderError {}

impl From<String> for DataLoaderError {
    fn from(err: String) -> Self {
        DataLoaderError::BatchError(err)
    }
}

// ✅ CORREÇÃO: MUDAR para pub e manter Display manual
#[derive(Debug, Clone)]
pub enum RateLimitError { 
    LimitExceeded {
        key: String,
        max_requests: u64,
        reset_in: Duration,
        retry_after: u64,
    },
}

impl fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RateLimitError::LimitExceeded { key, max_requests, reset_in, retry_after } => {
                write!(
                    f,
                    "Rate limit exceeded for '{}': {}/{} requests. Reset in {:?} (retry after {}s)",
                    key, max_requests, max_requests, reset_in, retry_after
                )
            }
        }
    }
}

impl std::error::Error for RateLimitError {}

impl From<RateLimitError> for DataLoaderError {
    fn from(err: RateLimitError) -> Self {
        DataLoaderError::RateLimitExceeded(err.to_string())
    }
}

impl From<crate::query_cost::QueryCostError> for DataLoaderError {
    fn from(err: crate::query_cost::QueryCostError) -> Self {
        DataLoaderError::QueryCostExceeded(err.to_string())
    }
}
