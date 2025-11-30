pub mod executor;
pub mod sql;

// Modules désactivés temporairement (Broken/Obsolètes)
// pub mod optimizer;
// pub mod parser;

// #[cfg(test)]
// mod tests;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use executor::QueryEngine;

// --- Structures de Données du Moteur de Requête ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub collection: String,
    pub filter: Option<QueryFilter>,
    pub sort: Option<Vec<SortField>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub projection: Option<Vec<String>>,
}

impl Query {
    pub fn new(collection: &str) -> Self {
        Self {
            collection: collection.to_string(),
            filter: None,
            sort: None,
            limit: None,
            offset: None,
            projection: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryFilter {
    pub operator: FilterOperator,
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    And,
    Or,
    Not,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub field: String,
    pub operator: ComparisonOperator,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    In,
    Contains,
    StartsWith,
    EndsWith,
    Matches,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortField {
    pub field: String,
    pub order: SortOrder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResult {
    pub documents: Vec<Value>,
    pub total_count: u64,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}
