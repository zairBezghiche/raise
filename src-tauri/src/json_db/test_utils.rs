// FICHIER : src-tauri/src/json_db/test_utils.rs

use crate::json_db::storage::{JsonDbConfig, StorageEngine};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Once;

static INIT: Once = Once::new();

pub const TEST_SPACE: &str = "test_space";
pub const TEST_DB: &str = "test_db";

pub struct TestEnv {
    pub cfg: JsonDbConfig,
    pub storage: StorageEngine,
    pub space: String,
    pub db: String,
    // Le TempDir est supprimé quand cette struct est drop, nettoyant le test
    pub tmp_dir: tempfile::TempDir,
}

pub fn init_test_env() -> TestEnv {
    INIT.call_once(|| {
        // Initialisation du logger pour les tests (optionnel)
        let _ = tracing_subscriber::fmt()
            .with_env_filter("info")
            .with_test_writer()
            .try_init();
    });

    // 1. Création du dossier temporaire isolé
    let tmp_dir = tempfile::tempdir().expect("create temp dir");
    let data_root = tmp_dir.path().to_path_buf();

    let cfg = JsonDbConfig {
        data_root: data_root.clone(),
    };

    // --- 2. COPIE DES SCHÉMAS RÉELS ---
    // On localise le dossier 'schemas/v1' à la racine du repo pour le copier dans l'env de test
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")); // src-tauri

    // Recherche robuste : src-tauri/../schemas/v1 ou ./schemas/v1
    let possible_paths = vec![
        manifest_dir.join("../schemas/v1"),
        manifest_dir.join("schemas/v1"),
        PathBuf::from("schemas/v1"),
    ];

    let src_schemas = possible_paths.into_iter().find(|p| p.exists());

    // Destination : <tmp>/test_space/test_db/_system/schemas/v1
    let dest_schemas = cfg.db_schemas_root(TEST_SPACE, TEST_DB).join("v1");
    fs::create_dir_all(&dest_schemas).expect("Failed to create schema dir in temp");

    if let Some(src) = src_schemas {
        copy_dir_recursive(&src, &dest_schemas).expect("Failed to copy schemas to test env");
    } else {
        eprintln!("⚠️ WARNING: Schemas source not found. Tests dependent on x_compute might fail.");
    }

    // --- 3. CRÉATION DES DATASETS MOCKS ---
    // On prépare des données de test directement dans le dossier temporaire
    let dataset_root = data_root.join("dataset");
    fs::create_dir_all(&dataset_root).unwrap();

    // Mock Article (sans ID, pour tester la génération)
    let article_path = dataset_root.join("arcadia/v1/data/articles/article.json");
    if let Some(p) = article_path.parent() {
        fs::create_dir_all(p).unwrap();
    }
    let mock_article = r#"{
        "handle": "mock-handle",
        "displayName": "Mock Article",
        "slug": "mock-slug",
        "title": "Mock Title",
        "status": "draft",
        "authorId": "00000000-0000-0000-0000-000000000000"
    }"#;
    fs::write(&article_path, mock_article).unwrap();

    // Mock Exchange Item
    let exchange_path = dataset_root.join("arcadia/v1/data/exchange-items/position_gps.json");
    if let Some(p) = exchange_path.parent() {
        fs::create_dir_all(p).unwrap();
    }
    fs::write(
        &exchange_path,
        r#"{ "name": "GPS Position", "exchangeMechanism": "Flow" }"#,
    )
    .unwrap();

    let storage = StorageEngine::new(cfg.clone());

    TestEnv {
        cfg,
        storage,
        space: TEST_SPACE.to_string(),
        db: TEST_DB.to_string(),
        tmp_dir,
    }
}

pub fn ensure_db_exists(cfg: &JsonDbConfig, space: &str, db: &str) {
    let db_path = cfg.db_root(space, db);
    if !db_path.exists() {
        std::fs::create_dir_all(&db_path).unwrap();
    }
}

pub fn get_dataset_file(cfg: &JsonDbConfig, rel_path: &str) -> PathBuf {
    let root = cfg.data_root.join("dataset");
    let path = root.join(rel_path);
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).unwrap();
        }
    }
    path
}

/// Helper récursif pour copier les dossiers
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
