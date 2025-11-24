# 📦 Module `json_db`

## Vue d'Ensemble

Le module **`json_db`** est une base de données orientée documents JSON avec support de schémas, validation, et enrichissement automatique. C'est le cœur du système de persistance de GenAptitude, offrant une alternative légère et souveraine aux bases de données traditionnelles, optimisée pour l'architecture locale de Tauri.

### Caractéristiques Principales

- **Validation JSON Schema** : Support complet de JSON Schema avec résolution des `$ref` locaux et distants.
- **Architecture Async & Thread-Safe** : Conçu pour l'environnement multi-thread de Tauri (utilisation de `RwLock`, types `Send + Sync`).
- **Système `x_compute`** : Enrichissement automatique des documents (génération d'UUIDs, timestamps, concaténation, références).
- **Stockage Fichier Atomique** : Écritures sécurisées via le pattern _Write-Tmp-Rename_ (pas de corruption en cas de crash).
- **Moteur de Requêtes Asynchrone** : Filtrage, tri et pagination non-bloquants.
- **Isolation** : Structure hiérarchique `Space` > `Database` > `Collection`.

---

## 🏗️ Architecture Générale

### Structure Modulaire

```
json_db/
├── collections/      # Gestionnaire de collections (Façade Thread-Safe)
├── query/           # Moteur de requêtes (Async)
├── schema/          # Validation, Registre et logique x_compute
├── storage/         # Configuration et I/O bas niveau (Atomic writes)
├── jsonld/          # Support du contexte sémantique
├── transactions/    # (Placeholder) Gestion ACID future
└── indexes/         # (Placeholder) Indexation BTree/Hash
```

### Modèle de Concurrence

Le système utilise un modèle hybride pour garantir la sécurité des threads (requis par Tauri) et la performance :

1.  **`CollectionsManager` (État Partagé)** :
    - C'est le point d'entrée principal.
    - Il détient le `SchemaRegistry` protégé par un **`std::sync::RwLock`**.
    - Permet des lectures concurrentes massives (accès aux schémas).
    - Les écritures sur le registre (chargement lazy) bloquent brièvement les lecteurs.
2.  **`QueryEngine` (Exécution)** :
    - Instancié à la demande pour une requête spécifique.
    - Emprunte une référence au `CollectionsManager` pour lire les données.
    - Exécute le scan, le filtrage et le tri de manière asynchrone (`async/await`).

---

## 📚 Modules Détaillés

### 1. Module `collections` (Manager)

**Responsabilité** : Façade haut niveau pour la manipulation de documents. C'est l'objet que vous manipulez dans les commandes Tauri.

#### API Principale (Synchrone & Atomique)

Les opérations d'écriture sont synchrones pour garantir la persistance immédiate sur le disque.

```rust
// Initialisation
let mgr = CollectionsManager::new(&cfg, "space", "db");

// Création/Suppression de collection
mgr.create_collection("actors")?;
mgr.drop_collection("actors")?;

// Opérations avec schéma (x_compute + validate + persist)
mgr.insert_with_schema("actors/actor.schema.json", doc)?;
mgr.update_with_schema("actors/actor.schema.json", doc)?;
mgr.upsert_with_schema("actors/actor.schema.json", doc)?;

// Lecture directe
let doc = mgr.get("actors", "uuid-123")?;

// Listing
let ids = mgr.list_ids("actors")?;
let docs = mgr.list_all("actors")?; // Attention: charge tout en mémoire
```

### 2. Module `query` (Moteur de Recherche)

**Responsabilité** : Exécuter des recherches complexes (WHERE, ORDER BY, LIMIT) de manière asynchrone.

#### API Unifiée

Le moteur expose une méthode principale : `execute_query`.

```rust
pub struct Query {
    pub collection: String,
    pub filter: Option<QueryFilter>,    // Structure { operator, conditions }
    pub sort: Option<Vec<SortField>>,   // Tri multi-critères
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub projection: Option<Projection>, // Sélection de champs
}

// Exécution Async
let result: QueryResult = engine.execute_query(query).await?;
```

### 3. Module `schema` & `x_compute`

**Responsabilité** : Intelligence des données.

- **`SchemaRegistry`** : Charge, parse et cache les schémas JSON. Chargement "Lazy" (à la demande) thread-safe.
- **`x_compute`** : Extension propriétaire exécutée _avant_ la validation.
  - `uuid()` : Génère un ID unique si absent.
  - `now()` : Met à jour les champs `createdAt` / `updatedAt`.
  - `ptr()` : Récupère des valeurs ailleurs dans le document.
  - `concat()` : Concatène des chaînes.

---

## 💡 Exemples d'Utilisation

### Configuration et Initialisation

```rust
use genaptitude::json_db::storage::JsonDbConfig;
use genaptitude::json_db::collections::manager::CollectionsManager;

// 1. Charger la config (depuis .env ou paramètre)
let cfg = JsonDbConfig::from_env("/path/to/repo")?;

// 2. Créer le manager (Thread-Safe, peut être partagé dans l'AppHandle)
let mgr = CollectionsManager::new(&cfg, "un2", "_system");
```

### Insertion d'un Document (CRUD)

```rust
use serde_json::json;

let doc = json!({
    "name": "Projet Alpha",
    "status": "active"
    // Pas besoin de mettre 'id' ou 'createdAt', x_compute s'en charge
});

// L'insertion est atomique : soit le fichier final existe et est valide, soit rien ne change.
let stored = mgr.insert_with_schema("projects/project.schema.json", doc)?;

println!("ID généré : {}", stored["id"]);
```

### Requête Complexe (Async)

Ceci est typiquement utilisé dans une commande Tauri (`#[tauri::command] async fn`).

```rust
use genaptitude::json_db::query::{Query, QueryEngine, QueryFilter, Condition, ComparisonOperator, FilterOperator};

async fn search_active_projects(mgr: &CollectionsManager<'_>) -> Result<Vec<Value>> {
    // 1. Init Moteur (emprunte le manager)
    let engine = QueryEngine::new(mgr);

    // 2. Construction de la requête
    let query = Query {
        collection: "projects".to_string(),
        filter: Some(QueryFilter {
            operator: FilterOperator::And,
            conditions: vec![
                Condition {
                    field: "status".to_string(),
                    operator: ComparisonOperator::Eq,
                    value: json!("active")
                }
            ]
        }),
        sort: Some(vec![/* ... */]),
        limit: Some(50),
        offset: None,
        projection: None
    };

    // 3. Exécution (await requis)
    let result = engine.execute_query(query).await?;

    Ok(result.documents)
}
```

---

## 🔧 Détails Techniques

### Flux de Données (Pipeline d'Écriture)

1.  **Appel API** : `insert_with_schema(schema_rel, doc)`
2.  **Loading** : Le `CollectionsManager` verrouille le `SchemaRegistry` (RwLock) et charge le schéma si nécessaire.
3.  **Compilation** : Création d'un `SchemaValidator`.
4.  **Compute** : Exécution des fonctions `x_compute` (modification du doc in-place).
5.  **Validation** : Vérification stricte JSON Schema.
6.  **Persistance** :
    - Création fichier `.tmp`
    - Écriture JSON
    - `fs::rename` atomique vers le fichier final.

### Structure sur Disque

```
<domain_root>/          (défini par PATH_GENAPTITUDE_DOMAIN)
  ├── un2/              (space)
  │   ├── _system/      (db)
  │   │   ├── _system.json  (index DB)
  │   │   ├── collections/
  │   │   │   ├── actors/
  │   │   │   │   ├── uuid-1.json
  │   │   │   │   └── uuid-2.json
  │   │   │   └── ...
  │   │   └── schemas/
  │   │       └── v1/       (copie locale des schémas)
```

### Variables d'Environnement

- `PATH_GENAPTITUDE_DOMAIN` : Chemin racine où les données sont stockées.
- `PATH_GENAPTITUDE_DATASET` : (Tests) Chemin vers les jeux de données pour le seeding.
- `RUST_LOG` : Configuration des logs (ex: `info,genaptitude::json_db=debug`).

---

## ⚠️ Limitations Actuelles

- **Indexation** : Les recherches (`QueryEngine`) font actuellement un **scan complet** des fichiers de la collection. Les performances dépendent de la taille de la collection. L'implémentation des B-Trees est prévue.
- **Transactions** : Pas de transactions multi-documents (ACID sur un seul fichier uniquement pour l'instant).

---

**Dernière mise à jour** : Architecture Async/RwLock - Novembre 2025
