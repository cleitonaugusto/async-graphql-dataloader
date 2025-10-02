// src/cache.rs
use std::{
    hash::Hash,
    time::{Duration, Instant},
};

pub struct Cache<K, V> {
    store: dashmap::DashMap<K, (V, Instant)>,
    ttl: Option<Duration>,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            store: dashmap::DashMap::new(),
            ttl: None,
        }
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let entry = self.store.get(key)?;

        if let Some(ttl) = self.ttl {
            if entry.1.elapsed() > ttl {
                self.store.remove(key);
                return None;
            }
        }

        Some(entry.0.clone())
    }

    pub fn set(&self, key: K, value: V) {
        self.store.insert(key, (value, Instant::now()));
    }

    pub fn clear(&self) {
        self.store.clear();
    }
}
