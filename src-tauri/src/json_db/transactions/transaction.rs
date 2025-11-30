use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Représente une opération atomique dans une transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionOperation {
    Insert {
        collection: String,
        id: String,
        document: Value,
    },
    Update {
        collection: String,
        id: String,
        old_document: Option<Value>,
        new_document: Value,
    },
    Delete {
        collection: String,
        id: String,
        old_document: Option<Value>,
    },
}

/// Une transaction active en cours de construction (Staging Area).
pub struct ActiveTransaction {
    pub id: String,
    pub operations: Vec<TransactionOperation>,
}

impl Default for ActiveTransaction {
    fn default() -> Self {
        Self::new()
    }
}

impl ActiveTransaction {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            operations: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    pub fn add_insert(&mut self, collection: &str, id: &str, doc: Value) {
        self.operations.push(TransactionOperation::Insert {
            collection: collection.to_string(),
            id: id.to_string(),
            document: doc,
        });
    }

    pub fn add_update(&mut self, collection: &str, id: &str, old: Option<Value>, new: Value) {
        self.operations.push(TransactionOperation::Update {
            collection: collection.to_string(),
            id: id.to_string(),
            old_document: old,
            new_document: new,
        });
    }

    pub fn add_delete(&mut self, collection: &str, id: &str, old: Option<Value>) {
        self.operations.push(TransactionOperation::Delete {
            collection: collection.to_string(),
            id: id.to_string(),
            old_document: old,
        });
    }
}
