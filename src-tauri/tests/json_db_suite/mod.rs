// src-tauri/tests/json_db_suite/mod.rs

use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use genaptitude::json_db::storage::file_storage::{create_db, open_db, DbHandle};
use genaptitude::json_db::storage::JsonDbConfig;

/// Espace / DB par défaut pour toute la suite json_db
pub const TEST_SPACE: &str = "tests";
pub const TEST_DB: &str = "testU";

// 💡 VERROU GLOBAL : Empêche les tests de se marcher dessus lors de l'init de l'env
static ENV_INIT_LOCK: Mutex<()> = Mutex::new(());

pub struct TestEnv {
    pub cfg: JsonDbConfig,
    pub tmp_root: PathBuf,
    pub space: String,
    pub db: String,
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        // Nettoyage best-effort
        let _ = fs::remove_dir_all(&self.tmp_root);
    }
}

/// Crée un répertoire racine temporaire pour PATH_GENAPTITUDE_DOMAIN
fn tmp_root() -> PathBuf {
    let base = std::env::temp_dir();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos(); // Nanos pour moins de collision
                     // On ajoute l'ID du thread pour garantir l'unicité
    let thread_id = std::thread::current().id();
    let p = base.join(format!("jsondb_ut_{ts}_{:?}", thread_id));
    fs::create_dir_all(&p).expect("création du domaine temporaire");
    p
}

/// Remonte dans l’arbo pour retrouver la racine du repo (celle qui contient `schemas/v1`)
fn find_repo_root(start: &Path) -> PathBuf {
    let mut cur = Some(start);
    while let Some(p) = cur {
        if p.join("schemas").join("v1").is_dir() {
            return p.to_path_buf();
        }
        cur = p.parent();
    }
    // Fallback dossier courant
    if start.join("schemas").join("v1").is_dir() {
        return start.to_path_buf();
    }
    panic!(
        "schemas/v1 introuvable en remontant depuis {}",
        start.display()
    );
}

/// Version paramétrable : tu peux lui passer n’importe quel couple (space, db).
pub fn init_test_env_for(space: &str, db: &str) -> TestEnv {
    // 1. On verrouille l'accès global pour configurer l'environnement
    let _guard = ENV_INIT_LOCK.lock().unwrap();

    let tmp = tmp_root();

    // 2. On définit la variable d'environnement (Critique : doit être fait sous verrou)
    unsafe {
        std::env::set_var("PATH_GENAPTITUDE_DOMAIN", &tmp);
    }

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = find_repo_root(&manifest);

    // 3. On charge la config IMMÉDIATEMENT (elle capture la valeur de l'env)
    let cfg = JsonDbConfig::from_env(&repo_root).expect("JsonDbConfig::from_env");

    // 4. On relâche le verrou (à la fin du scope de _guard) pour que les autres tests puissent s'initier.
    // La config `cfg` contient désormais le bon chemin en dur pour ce test.
    drop(_guard);

    TestEnv {
        cfg,
        tmp_root: tmp,
        space: space.to_string(),
        db: db.to_string(),
    }
}

/// Version "par défaut" : (space, db) = (TEST_SPACE, TEST_DB)
pub fn init_test_env() -> TestEnv {
    init_test_env_for(TEST_SPACE, TEST_DB)
}

pub fn ensure_db_exists(cfg: &JsonDbConfig, space: &str, db: &str) -> DbHandle {
    match open_db(cfg, space, db) {
        Ok(handle) => handle,
        Err(_) => create_db(cfg, space, db).expect("create_db in ensure_db_exists"),
    }
}

pub fn get_dataset_root() -> PathBuf {
    // Charge les variables du .env si ce n'est pas déjà fait
    dotenvy::dotenv().ok();

    let path_str = std::env::var("PATH_GENAPTITUDE_DATASET")
        .unwrap_or_else(|_| "$HOME/genaptitude_dataset".to_string());

    let path_string = if path_str.starts_with("$HOME") {
        let home = std::env::var("HOME").expect("HOME non défini");
        path_str.replace("$HOME", &home)
    } else {
        path_str
    };

    PathBuf::from(path_string)
}
