pub mod cache;
pub mod file_storage;

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::Duration;

// --- CORRECTION : Import des types depuis les sous-modules ---
use self::cache::Cache;
use self::file_storage::DbIndex;

#[derive(Clone, Debug)]
pub struct JsonDbConfig {
    pub domain_root: PathBuf,
    pub schemas_dev_root: PathBuf,
}

impl JsonDbConfig {
    pub fn from_env(repo_root: impl AsRef<Path>) -> anyhow::Result<Self> {
        // 1. Charge le .env
        let _ = dotenvy::dotenv();

        // 2. Récupère la variable brute
        let domain_path_str = std::env::var("PATH_GENAPTITUDE_DOMAIN")
            .map_err(|e| anyhow::anyhow!("ENV PATH_GENAPTITUDE_DOMAIN manquant: {e}"))?;

        // 3. Expansion manuelle de $HOME
        let domain_root = expand_path(&domain_path_str);

        let schemas_dev_root = repo_root.as_ref().join("schemas").join("v1");

        Ok(Self {
            domain_root,
            schemas_dev_root,
        })
    }

    #[inline]
    pub fn space_root(&self, space: &str) -> PathBuf {
        self.domain_root.join(space)
    }
    #[inline]
    pub fn db_root(&self, space: &str, db: &str) -> PathBuf {
        self.space_root(space).join(db)
    }
    #[inline]
    pub fn index_path(&self, space: &str, db: &str) -> PathBuf {
        self.db_root(space, db).join("_system.json")
    }
    #[inline]
    pub fn db_schemas_root(&self, space: &str, db: &str) -> PathBuf {
        self.db_root(space, db).join("schemas").join("v1")
    }
}

/// Helper local pour remplacer $HOME ou ~ par le vrai chemin home
fn expand_path(path: &str) -> PathBuf {
    let mut p = path.to_string();

    // Si le chemin contient $HOME ou commence par ~
    if p.contains("$HOME") || p.starts_with("~/") {
        // On récupère le HOME du système
        if let Ok(home) = std::env::var("HOME") {
            p = p.replace("$HOME", &home);
            if p.starts_with("~/") {
                p = p.replacen("~", &home, 1);
            }
        }
    }

    PathBuf::from(p)
}

/// Moteur de stockage avec gestion de cache.
/// Cette structure est Thread-Safe (Clone pas cher grâce aux Arc internes du Cache).
#[derive(Clone, Debug)]
pub struct StorageEngine {
    pub config: JsonDbConfig,

    /// Cache pour les manifestes de base de données (_system.json)
    /// Clé: "space/db"
    index_cache: Cache<String, DbIndex>,
}

impl StorageEngine {
    pub fn new(config: JsonDbConfig) -> Self {
        Self {
            config,
            // Cache de 50 index, expiration après 5 minutes
            index_cache: Cache::new(50, Some(Duration::from_secs(300))),
        }
    }

    /// Lecture optimisée de l'index DB (Cache-First)
    pub fn get_index(&self, space: &str, db: &str) -> Result<DbIndex> {
        let key = format!("{}/{}", space, db);

        // 1. Cache Hit ?
        if let Some(cached_index) = self.index_cache.get(&key) {
            return Ok(cached_index);
        }

        // 2. Cache Miss : Lecture disque
        // On appelle la fonction bas niveau de file_storage
        let index = file_storage::read_index(&self.config, space, db)?;

        // 3. Mise en cache
        self.index_cache.put(key, index.clone());

        Ok(index)
    }

    /// Invalidation du cache (à appeler après une écriture)
    pub fn invalidate_index(&self, space: &str, db: &str) {
        let key = format!("{}/{}", space, db);
        self.index_cache.remove(&key);
    }

    /// Met à jour le cache explicitement (après une écriture réussie par exemple)
    pub fn update_cached_index(&self, space: &str, db: &str, index: DbIndex) {
        let key = format!("{}/{}", space, db);
        self.index_cache.put(key, index);
    }
}
