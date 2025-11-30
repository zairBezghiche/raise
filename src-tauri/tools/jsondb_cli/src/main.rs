// FICHIER : src-tauri/tools/jsondb_cli/src/main.rs

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

// Imports depuis le cœur
use genaptitude::json_db::collections::manager::CollectionsManager;
use genaptitude::json_db::query::{Query, QueryEngine};
use genaptitude::json_db::storage::{file_storage, JsonDbConfig, StorageEngine};
use genaptitude::json_db::transactions::manager::TransactionManager;
// CORRECTION : Import explicite de Operation
use genaptitude::json_db::transactions::Operation;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "default_space")]
    space: String,

    #[arg(short, long, default_value = "default_db")]
    db: String,

    #[arg(long, env = "JSONDB_DATA_ROOT", default_value = "./data")]
    root: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Crée une collection
    CreateCollection {
        name: String,
        schema: Option<String>,
    },

    /// Liste les collections
    ListCollections,

    /// Insère un document
    Insert { collection: String, data: String },

    /// Requête simple (tout ou filtre JSON)
    Query {
        collection: String,
        filter: Option<String>,
    },

    /// Requête SQL
    Sql { query: String },

    /// Importe un dossier de fichiers JSON
    Import { collection: String, path: PathBuf },

    /// Exécute une transaction (JSON file)
    Transaction { file: PathBuf },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Init Config
    let config = JsonDbConfig {
        data_root: cli.root.clone(),
    };

    // Assure l'existence de la DB
    if !config.db_root(&cli.space, &cli.db).exists() {
        file_storage::create_db(&config, &cli.space, &cli.db)?;
    }

    let storage = StorageEngine::new(config.clone());
    let mgr = CollectionsManager::new(&storage, &cli.space, &cli.db);

    match cli.command {
        Commands::CreateCollection { name, schema } => {
            mgr.create_collection(&name, schema)?;
            println!("Collection '{}' created.", name);
        }
        Commands::ListCollections => {
            let cols = mgr.list_collections()?;
            println!("Collections: {:?}", cols);
        }
        Commands::Insert { collection, data } => {
            let doc: Value = serde_json::from_str(&data)?;
            let res = mgr.insert_with_schema(&collection, doc)?;
            println!("Inserted: {}", res);
        }
        Commands::Query { collection, filter } => {
            let filter_obj = if let Some(_f) = filter {
                println!("Warning: Filter parsing not fully implemented in CLI");
                None
            } else {
                None
            };

            let query = Query {
                collection: collection.clone(),
                filter: filter_obj,
                sort: None,
                limit: None,
                offset: None,
                projection: None,
            };

            let engine = QueryEngine::new(&mgr);
            let result = engine.execute_query(query).await?;
            println!("Found {} documents.", result.total_count);
            for doc in result.documents {
                println!("{}", doc);
            }
        }
        Commands::Sql { query } => {
            let q = genaptitude::json_db::query::sql::parse_sql(&query)?;
            let engine = QueryEngine::new(&mgr);
            let result = engine.execute_query(q).await?;
            println!("SQL Result: {} docs found", result.total_count);
            for doc in result.documents {
                println!("{}", doc);
            }
        }
        Commands::Import { collection, path } => {
            if path.is_dir() {
                for entry in fs::read_dir(path)? {
                    let entry = entry?;
                    if entry.path().extension().map_or(false, |e| e == "json") {
                        let content = fs::read_to_string(entry.path())?;
                        let doc: Value = serde_json::from_str(&content)?;
                        mgr.insert_with_schema(&collection, doc)?;
                    }
                }
            } else {
                let content = fs::read_to_string(path)?;
                let doc: Value = serde_json::from_str(&content)?;
                mgr.insert_with_schema(&collection, doc)?;
            }
            println!("Import done.");
        }
        Commands::Transaction { file } => {
            let content = fs::read_to_string(file)?;
            let ops: Vec<TxOp> = serde_json::from_str(&content)?;

            let tm = TransactionManager::new(&config, &cli.space, &cli.db);

            tm.execute(|tx| {
                for op in ops {
                    match op {
                        TxOp::Insert {
                            collection,
                            id,
                            document,
                        } => {
                            tx.operations.push(Operation::Insert {
                                collection,
                                id,
                                document,
                            });
                        }
                        TxOp::Update {
                            collection,
                            id,
                            document,
                        } => {
                            tx.operations.push(Operation::Update {
                                collection,
                                id,
                                document,
                            });
                        }
                        TxOp::Delete { collection, id } => {
                            tx.operations.push(Operation::Delete { collection, id });
                        }
                    }
                }
                Ok(())
            })?;
            println!("Transaction executed.");
        }
    }

    Ok(())
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum TxOp {
    Insert {
        collection: String,
        id: String,
        document: Value,
    },
    Update {
        collection: String,
        id: String,
        document: Value,
    },
    Delete {
        collection: String,
        id: String,
    },
}
