use crate::json_db::indexes::IndexManager;
use crate::json_db::storage::{file_storage, JsonDbConfig, StorageEngine};
use crate::json_db::transactions::{Operation, Transaction, TransactionLog, TransactionStatus};
use anyhow::Result;
use std::fs;

pub struct TransactionManager<'a> {
    config: &'a JsonDbConfig,
    space: String,
    db: String,
}

impl<'a> TransactionManager<'a> {
    pub fn new(config: &'a JsonDbConfig, space: &str, db: &str) -> Self {
        Self {
            config,
            space: space.to_string(),
            db: db.to_string(),
        }
    }

    pub fn execute<F>(&self, op_block: F) -> Result<()>
    where
        F: FnOnce(&mut Transaction) -> Result<()>,
    {
        let mut tx = Transaction::new();
        op_block(&mut tx)?;
        self.write_wal(&tx)?;
        match self.apply_transaction(&tx) {
            Ok(_) => {
                self.commit_wal(&tx)?;
                Ok(())
            }
            Err(e) => {
                self.rollback_wal(&tx)?;
                Err(e)
            }
        }
    }

    fn write_wal(&self, tx: &Transaction) -> Result<()> {
        let wal_path = self.config.db_root(&self.space, &self.db).join("wal");
        if !wal_path.exists() {
            fs::create_dir_all(&wal_path)?;
        }
        let tx_file = wal_path.join(format!("{}.json", tx.id));

        let log = TransactionLog {
            id: tx.id.clone(),
            status: TransactionStatus::Pending,
            operations: tx.operations.clone(),
            timestamp: chrono::Utc::now().timestamp(),
        };
        fs::write(tx_file, serde_json::to_string_pretty(&log)?)?;
        Ok(())
    }

    fn apply_transaction(&self, tx: &Transaction) -> Result<()> {
        let storage = StorageEngine::new(self.config.clone());
        let mut idx = IndexManager::new(&storage, &self.space, &self.db);

        for op in &tx.operations {
            match op {
                Operation::Insert {
                    collection,
                    id,
                    document,
                } => {
                    file_storage::write_document(
                        self.config,
                        &self.space,
                        &self.db,
                        collection,
                        id,
                        document,
                    )?;
                    idx.index_document(collection, document)?;
                }
                Operation::Update {
                    collection,
                    id,
                    document,
                } => {
                    file_storage::write_document(
                        self.config,
                        &self.space,
                        &self.db,
                        collection,
                        id,
                        document,
                    )?;
                    idx.index_document(collection, document)?;
                }
                Operation::Delete { collection, id } => {
                    file_storage::delete_document(
                        self.config,
                        &self.space,
                        &self.db,
                        collection,
                        id,
                    )?;
                    idx.remove_document(collection, &serde_json::Value::Null)?; // Placeholder doc
                }
            }
        }
        Ok(())
    }

    fn commit_wal(&self, tx: &Transaction) -> Result<()> {
        let path = self
            .config
            .db_root(&self.space, &self.db)
            .join("wal")
            .join(format!("{}.json", tx.id));
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    fn rollback_wal(&self, tx: &Transaction) -> Result<()> {
        self.commit_wal(tx)
    }
}
