//! Module de gestion de cache générique en mémoire.
//!
//! Fournit une structure `Cache<K, V>` thread-safe pour stocker temporairement
//! des ressources coûteuses à charger (Index, Schémas, Documents).

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Une entrée dans le cache, contenant la valeur et ses métadonnées de cycle de vie.
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    /// Moment de création pour le calcul du TTL
    #[allow(dead_code)]
    created_at: Instant,
    /// Moment de dernière utilisation pour l'éviction (LRU)
    #[allow(dead_code)]
    last_accessed: Instant,
    /// Date d'expiration optionnelle
    expires_at: Option<Instant>,
}

/// Cache générique Thread-Safe (K -> V).
///
/// Conçu pour être partagé entre plusieurs threads (Clonable via Arc interne).
///
/// # Exemple
/// ```rust
/// use genaptitude::json_db::storage::cache::Cache;
/// use std::time::Duration;
///
/// // Création d'un cache avec une capacité de 100 et un TTL de 60s
/// let cache = Cache::new(100, Some(Duration::from_secs(60)));
///
/// let user_data = "data".to_string();
/// cache.put("user_1", user_data.clone());
///
/// if let Some(user) = cache.get(&"user_1") {
///     assert_eq!(user, user_data);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Cache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Stockage interne protégé par un verrou lecture/écriture
    store: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    /// Nombre maximum d'entrées
    capacity: usize,
    /// Durée de vie par défaut des entrées
    default_ttl: Option<Duration>,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    /// Crée une nouvelle instance de cache.
    ///
    /// * `capacity`: Nombre max d'éléments avant éviction.
    /// * `default_ttl`: Durée de vie par défaut (None = infini).
    pub fn new(capacity: usize, default_ttl: Option<Duration>) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::with_capacity(capacity))),
            capacity,
            default_ttl,
        }
    }

    /// Récupère une valeur du cache.
    /// Retourne `None` si la clé n'existe pas ou si l'entrée est expirée.
    /// Met à jour le timestamp `last_accessed`.
    pub fn get(&self, key: &K) -> Option<V> {
        // 1. Tentative de lecture rapide (Read Lock)
        let is_expired = {
            let read_guard = self.store.read().ok()?;
            if let Some(entry) = read_guard.get(key) {
                if let Some(expiry) = entry.expires_at {
                    if Instant::now() > expiry {
                        true
                    } else {
                        // Hit valide !
                        return Some(entry.value.clone());
                    }
                } else {
                    return Some(entry.value.clone());
                }
            } else {
                false // Pas trouvé
            }
        };

        // 2. Si expiré, on nettoie (Write Lock)
        if is_expired {
            if let Ok(mut write_guard) = self.store.write() {
                write_guard.remove(key);
            }
        }

        None
    }

    /// Insère une valeur dans le cache.
    /// Applique la politique d'éviction si nécessaire.
    pub fn put(&self, key: K, value: V) {
        // Calcul de l'expiration
        let now = Instant::now();
        let expires_at = self.default_ttl.map(|ttl| now + ttl);

        let entry = CacheEntry {
            value,
            created_at: now,
            last_accessed: now,
            expires_at,
        };

        if let Ok(mut guard) = self.store.write() {
            // Gestion de la capacité (Éviction simplifiée)
            if guard.len() >= self.capacity && !guard.contains_key(&key) {
                // 1. Nettoyage des expirés
                guard.retain(|_, v| v.expires_at.map(|exp| exp > now).unwrap_or(true));

                // 2. Si toujours plein, on supprime arbitrairement pour faire de la place
                if guard.len() >= self.capacity {
                    if let Some(k_to_remove) = guard.keys().next().cloned() {
                        guard.remove(&k_to_remove);
                    }
                }
            }

            guard.insert(key, entry);
        }
    }

    /// Supprime explicitement une entrée (invalidation).
    pub fn remove(&self, key: &K) {
        if let Ok(mut guard) = self.store.write() {
            guard.remove(key);
        }
    }

    /// Vide entièrement le cache.
    pub fn clear(&self) {
        if let Ok(mut guard) = self.store.write() {
            guard.clear();
        }
    }

    /// Retourne le nombre d'éléments (approximatif si certains sont expirés mais pas encore purgés).
    pub fn len(&self) -> usize {
        self.store.read().map(|g| g.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
