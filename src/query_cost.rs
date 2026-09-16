use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct QueryCostAnalyzer {
    cost_rules: Arc<RwLock<HashMap<String, CostRule>>>,
    max_cost_per_request: u64,
    cost_history: Arc<RwLock<Vec<CostHistoryEntry>>>,
    max_history_size: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CostRule {
    pub base_cost: u64,
    pub per_item_cost: u64,
    pub max_items: Option<usize>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCost {
    pub total_cost: u64,
    pub breakdown: HashMap<String, u64>,
    pub timestamp: SystemTime,
    pub operation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CostHistoryEntry {
    pub timestamp: SystemTime,
    pub operation: String,
    pub cost: u64,
    pub item_count: usize,
    pub client_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryCostError {
    CostExceeded {
        actual: u64,
        max_allowed: u64,
        breakdown: HashMap<String, u64>,
        operation: String,
    },
    TooManyItems {
        actual: usize,
        max_allowed: usize,
        operation: String,
    },
    InvalidOperation {
        operation: String,
        available_operations: Vec<String>,
    },
}

impl QueryCostAnalyzer {
    pub fn new() -> Self {
        Self {
            cost_rules: Arc::new(RwLock::new(HashMap::new())),
            max_cost_per_request: 1000,
            cost_history: Arc::new(RwLock::new(Vec::new())),
            max_history_size: 10_000,
        }
    }

    pub fn with_max_cost_per_request(self, max_cost: u64) -> Self {
        // ⬅️ REMOVER mut
        Self {
            max_cost_per_request: max_cost,
            ..self
        }
    }

    pub fn with_max_history_size(self, size: usize) -> Self {
        // ⬅️ REMOVER mut
        Self {
            max_history_size: size,
            ..self
        }
    }

    // CORREÇÃO: Método async para adicionar regras
    pub async fn add_cost_rule(&self, operation: String, rule: CostRule) {
        let mut rules = self.cost_rules.write().await; // ⬅️ MUDAR para write().await
        rules.insert(operation, rule);
    }

    // CORREÇÃO: Factory method em vez de builder pattern
    pub async fn with_default_rules() -> Self {
        let analyzer = Self::new();

        let default_rules = vec![
            (
                "load".to_string(),
                CostRule {
                    base_cost: 1,
                    per_item_cost: 1,
                    max_items: Some(1000),
                    description: "Single item load".to_string(),
                },
            ),
            (
                "load_many".to_string(),
                CostRule {
                    base_cost: 5,
                    per_item_cost: 1,
                    max_items: Some(500),
                    description: "Batch load multiple items".to_string(),
                },
            ),
            (
                "load_batch".to_string(),
                CostRule {
                    base_cost: 10,
                    per_item_cost: 1,
                    max_items: Some(1000),
                    description: "Optimized batch load".to_string(),
                },
            ),
        ];

        for (op, rule) in default_rules {
            analyzer.add_cost_rule(op, rule).await;
        }

        analyzer
    }

    // MANTER o método calculate_cost como está (já está correto)
    pub async fn calculate_cost(
        &self,
        operation: &str,
        item_count: usize,
        client_id: Option<&str>,
    ) -> Result<QueryCost, QueryCostError> {
        let rules = self.cost_rules.read().await;

        if !rules.contains_key(operation) {
            let available_operations: Vec<String> = rules.keys().cloned().collect();
            return Err(QueryCostError::InvalidOperation {
                operation: operation.to_string(),
                available_operations,
            });
        }

        let rule = rules.get(operation).unwrap();

        if let Some(max_items) = rule.max_items {
            if item_count > max_items {
                return Err(QueryCostError::TooManyItems {
                    actual: item_count,
                    max_allowed: max_items,
                    operation: operation.to_string(),
                });
            }
        }

        let operation_cost = rule.base_cost + (rule.per_item_cost * item_count as u64);
        let total_cost = operation_cost;

        if total_cost > self.max_cost_per_request {
            let mut breakdown = HashMap::new();
            breakdown.insert(operation.to_string(), operation_cost);

            Err(QueryCostError::CostExceeded {
                actual: total_cost,
                max_allowed: self.max_cost_per_request,
                breakdown,
                operation: operation.to_string(),
            })
        } else {
            let mut breakdown = HashMap::new();
            breakdown.insert(operation.to_string(), operation_cost);

            let cost = QueryCost {
                total_cost,
                breakdown,
                timestamp: SystemTime::now(),
                operation: operation.to_string(),
            };

            self.record_cost_history(&cost, item_count, client_id).await;

            Ok(cost)
        }
    }

    // MANTER o record_cost_history como está (já está correto)
    async fn record_cost_history(
        &self,
        cost: &QueryCost,
        item_count: usize,
        client_id: Option<&str>,
    ) {
        let mut history = self.cost_history.write().await;

        history.push(CostHistoryEntry {
            timestamp: cost.timestamp,
            operation: cost.operation.clone(),
            cost: cost.total_cost,
            item_count,
            client_id: client_id.map(|s| s.to_string()),
        });

        if history.len() > self.max_history_size {
            history.remove(0);
        }
    }

    // MANTER get_cost_analytics como está (já está correto)
    pub async fn get_cost_analytics(&self, time_window: Option<Duration>) -> CostAnalytics {
        let history = self.cost_history.read().await;
        let now = SystemTime::now();

        let relevant_history: Vec<&CostHistoryEntry> = if let Some(window) = time_window {
            history
                .iter()
                .filter(|entry| {
                    now.duration_since(entry.timestamp)
                        .unwrap_or(Duration::from_secs(0))
                        <= window
                })
                .collect()
        } else {
            history.iter().collect()
        };

        let total_operations = relevant_history.len();
        let total_cost: u64 = relevant_history.iter().map(|entry| entry.cost).sum();
        let avg_cost = if total_operations > 0 {
            total_cost / total_operations as u64
        } else {
            0
        };

        let operations_by_type: HashMap<String, usize> =
            relevant_history
                .iter()
                .fold(HashMap::new(), |mut acc, entry| {
                    *acc.entry(entry.operation.clone()).or_insert(0) += 1;
                    acc
                });

        CostAnalytics {
            total_operations,
            total_cost,
            average_cost: avg_cost,
            operations_by_type,
            time_window,
        }
    }

    // MANTER get_available_operations como está (já está correto)
    pub async fn get_available_operations(&self) -> Vec<String> {
        let rules = self.cost_rules.read().await;
        rules.keys().cloned().collect()
    }

    // MANTER get_cost_rule como está (já está correto)
    pub async fn get_cost_rule(&self, operation: &str) -> Option<CostRule> {
        let rules = self.cost_rules.read().await;
        rules.get(operation).cloned()
    }

    pub fn get_max_cost(&self) -> u64 {
        self.max_cost_per_request
    }

    // MANTER clear_history como está (já está correto)
    pub async fn clear_history(&self) {
        self.cost_history.write().await.clear();
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CostAnalytics {
    pub total_operations: usize,
    pub total_cost: u64,
    pub average_cost: u64,
    pub operations_by_type: HashMap<String, usize>,
    pub time_window: Option<Duration>,
}

impl std::fmt::Display for QueryCostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryCostError::CostExceeded {
                actual,
                max_allowed,
                breakdown,
                operation,
            } => {
                write!(
                    f,
                    "Query cost exceeded for '{}': {}/{}. Breakdown: {:?}",
                    operation, actual, max_allowed, breakdown
                )
            }
            QueryCostError::TooManyItems {
                actual,
                max_allowed,
                operation,
            } => {
                write!(
                    f,
                    "Too many items in '{}': {}/{}",
                    operation, actual, max_allowed
                )
            }
            QueryCostError::InvalidOperation {
                operation,
                available_operations,
            } => {
                write!(
                    f,
                    "Invalid operation '{}'. Available: {:?}",
                    operation, available_operations
                )
            }
        }
    }
}

impl std::error::Error for QueryCostError {}

// CORREÇÃO: Implementação Default sem with_default_rules (pois é async)
impl Default for QueryCostAnalyzer {
    fn default() -> Self {
        Self::new() // ⬅️ SIMPLES, sem regras padrão
    }
}

// MANTER Default para CostRule como está
impl Default for CostRule {
    fn default() -> Self {
        Self {
            base_cost: 1,
            per_item_cost: 1,
            max_items: Some(1000),
            description: "Default cost rule".to_string(),
        }
    }
}
