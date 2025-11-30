// src-tauri/tests/json_db_suite.rs
// Suite d'intégration unique pour toute la JSON DB.

#[path = "json_db_suite/mod.rs"]
mod common;

// Chaque fichier de tests concret devient un sous-module
#[path = "json_db_suite/json_db_errors.rs"]
mod json_db_errors;

#[path = "json_db_suite/json_db_idempotent.rs"]
mod json_db_idempotent;

#[path = "json_db_suite/json_db_integration.rs"]
mod json_db_integration;

#[path = "json_db_suite/json_db_lifecycle.rs"]
mod json_db_lifecycle;

#[path = "json_db_suite/json_db_query_integration.rs"]
mod json_db_query_integration;

#[path = "json_db_suite/workunits_x_compute.rs"]
mod workunits_x_compute;

#[path = "json_db_suite/schema_minimal.rs"]
mod schema_minimal;

#[path = "json_db_suite/dataset_integration.rs"]
mod dataset_integration;

#[path = "json_db_suite/json_db_sql.rs"]
mod json_db_sql;

// Rien d'autre : les #[test] dans ces modules seront automatiquement exécutés.
