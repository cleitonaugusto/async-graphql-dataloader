// src/error.rs
use std::fmt;

#[derive(Debug, Clone)]
pub enum DataLoaderError {
    ChannelClosed,
    BatchError(String),
    KeyNotFound,
    Timeout,
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
        }
    }
}

impl std::error::Error for DataLoaderError {}

impl From<String> for DataLoaderError {
    fn from(err: String) -> Self {
        DataLoaderError::BatchError(err)
    }
}
