use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod manager;
pub mod wal;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub operations: Vec<Operation>,
}

impl Transaction {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            operations: Vec::new(),
        }
    }

    pub fn add_insert(&mut self, collection: &str, id: &str, doc: Value) {
        self.operations.push(Operation::Insert {
            collection: collection.to_string(),
            id: id.to_string(),
            document: doc,
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    Insert {
        collection: String,
        id: String,
        document: Value,
    },
    Update {
        collection: String,
        id: String,
        document: Value,
    }, // J'utilise 'document' ici pour être cohérent
    Delete {
        collection: String,
        id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Committed,
    Rollback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLog {
    pub id: String,
    pub status: TransactionStatus,
    pub operations: Vec<Operation>,
    pub timestamp: i64,
}
