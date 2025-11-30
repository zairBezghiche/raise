# Module json_db

> Cette version reflète les évolutions majeures du code : l'introduction du StorageEngine pour le cache, le support complet des transactions ACID (WAL), le moteur d'indexation (Hash/BTree/Text) et l'optimiseur de requêtes.

---

## 📦 Vue d'Ensemble

Le module **json_db** est une base de données orientée documents JSON souveraine, transactionnelle et optimisée pour l'architecture locale de Tauri. Elle combine la flexibilité du NoSQL avec la rigueur des schémas JSON.

### Caractéristiques Principales

- **Stockage Souverain** : Données stockées en fichiers JSON lisibles, organisés hiérarchiquement (Espace > Base > Collection).

- **Transactions ACID** : Support complet des transactions multi-documents grâce à un Write-Ahead Log (WAL) (`_wal.jsonl`) et un gestionnaire de verrous, garantissant l'atomicité et la durabilité.

- **Moteur de Stockage (StorageEngine)** : Nouvelle couche d'abstraction thread-safe gérant la configuration et le cache en mémoire (Index DB, Schémas) pour des performances élevées en lecture.

- **Indexation Avancée** : Support des index Hash, B-Tree et Text (Full-text simple), persistés au format binaire (bincode) et mis à jour atomiquement avec les données.

- **Moteur de Requêtes Intelligent** : Exécution asynchrone avec optimiseur capable de sélectionner les index appropriés, de simplifier les filtres et de gérer le tri/pagination.

- **Moteur x_compute** : Calcul automatique de champs (UUID, Timestamps, Pointeurs) exécuté avant la validation, permettant des documents auto-suffisants.

- **Contexte Sémantique** : Support natif de JSON-LD pour lier les données aux ontologies métiers (Arcadia/Capella).

---

## 🏗️ Architecture Générale

L'architecture sépare clairement la persistance (synchrone/sécurisée) de l'interrogation (asynchrone/optimisée).

### Arborescence Physique

Structure définie par la variable d'environnement `PATH_GENAPTITUDE_DOMAIN` :

```
<domain_root>/
  ├── <space>/                 # Espace de travail (ex: "un2")
  │   ├── <database>/          # Base de données (ex: "_system")
  │   │   ├── _system.json     # Manifeste DB (Cacheable via StorageEngine)
  │   │   ├── _wal.jsonl       # Journal des transactions (Append-Only)
  │   │   ├── schemas/
  │   │   │   └── v1/          # Registre local des schémas JSON
  │   │   └── collections/
  │   │       └── <collection>/
  │   │           ├── _config.json       # Définition des index & schéma lié
  │   │           ├── _indexes/          # Fichiers d'index binaires (.idx)
  │   │           │   ├── email.hash.idx
  │   │           │   └── title.text.idx
  │   │           ├── <uuid>.json        # Documents unitaires
  │   │           └── ...
```

### Composants Clés

- **StorageEngine** : Le cœur de l'état partagé. Il maintient la configuration et les caches globaux (ex: le contenu de `_system.json`). Il est injecté dans l'état Tauri (`State<StorageEngine>`).

- **CollectionsManager** : Façade principale pour les opérations CRUD. Il utilise le StorageEngine pour accéder aux ressources et coordonne la validation, le calcul (`x_compute`) et la persistance.

- **TransactionManager** : Gère les blocs atomiques `execute(|tx| { ... })`. Il écrit dans le WAL avant d'appliquer les changements sur les fichiers et les index.

- **QueryEngine** : Analyse les requêtes (`Query`), utilise le `QueryOptimizer` pour déterminer la stratégie d'exécution (Index Scan vs Full Scan) et retourne les résultats filtrés.

---

## 📚 Modules Détaillés

### 1. Transactions (`transactions/`)

Le système garantit que toutes les opérations dans un bloc réussissent ou qu'aucune n'est appliquée.

- **WAL (`wal.rs`)** : Toutes les opérations sont sérialisées et écrites dans `_wal.jsonl` avant modification du FS.

- **LockManager** : Gère les verrous pour éviter les conditions de course sur les collections.

```rust
// Exemple d'utilisation interne (via commande Tauri)
let tm = TransactionManager::new(cfg, "space", "db");
tm.execute(|tx| {
    tx.add_insert("users", "u1", json!({...}));
    tx.add_update("accounts", "a1", None, json!({...}));
    // Si une erreur survient ou un panic, rien n'est persisté sur disque (sauf WAL temporaire)
    Ok(())
})?;
```

### 2. Indexation (`indexes/`)

Les index sont vitaux pour les performances de lecture. Ils sont gérés via un driver générique.

**Types supportés :**

- **Hash** : Pour les recherches exactes (`eq`).
- **BTree** : Pour les tris et recherches par plage (`gt`, `lt`, `sort`).
- **Text** : Index inversé pour la recherche plein texte (`contains`).

**Mise à jour :** Le `CollectionsManager` et le `TransactionManager` mettent à jour les fichiers `.idx` de manière synchrone après l'écriture du document JSON.

### 3. Moteur de Requêtes (`query/`)

Le `QueryEngine` exécute les recherches complexes définies par la structure `Query`.

- **Optimiseur (`optimizer.rs`)** : Analyse la requête pour réorganiser les conditions (les plus sélectives d'abord) et détecter les index utilisables.

- **Exécuteur (`executor.rs`)** :
  - Si un index couvre le filtre (ex: `where name = "X"` avec index Hash sur `name`), il récupère directement les IDs concernés (**Index Scan**).
  - Sinon, il itère sur tous les documents de la collection (**Full Scan**).

### 4. Schémas & Compute (`schema/`)

- **SchemaRegistry** : Charge et cache les schémas JSON. Gère la résolution des `$ref` internes (`db://...`).

- **x_compute** : Moteur de règles exécuté avant validation. Il gère :
  - `uuid_v4` : Génération d'ID.
  - `now_rfc3339` : Timestamps (`createdAt`, `updatedAt`).
  - `ptr` : Copie de valeurs intra-document ou depuis le contexte.

---

## 💡 Guide d'Utilisation (Rust Backend)

### Initialisation

```rust
use genaptitude::json_db::storage::{JsonDbConfig, StorageEngine};
use genaptitude::json_db::collections::manager::CollectionsManager;

// 1. Configuration (automatique via .env)
let config = JsonDbConfig::from_env("/path/to/repo")?;

// 2. Création du moteur (State global)
let storage = StorageEngine::new(config);

// 3. Instanciation d'un manager pour une requête spécifique
let mgr = CollectionsManager::new(&storage, "un2", "_system");
```

### Écriture (CRUD)

```rust
// Insertion avec validation et calcul automatique
// Le schéma détermine la collection cible (ex: "actors/actor.schema.json" -> "actors")
let doc = json!({ "name": "New Project" });
let result = mgr.insert_with_schema("projects/project.schema.json", doc)?;
// result contient maintenant "id", "createdAt", etc.
```

### Recherche (Query)

```rust
use genaptitude::json_db::query::{Query, QueryEngine, QueryFilter, Condition, ComparisonOperator};

// 1. Créer l'engin
let engine = QueryEngine::new(&mgr);

// 2. Construire la requête
let query = Query::new("projects")
    .filter(QueryFilter::and(vec![
        Condition::eq("status", json!("active")),
        Condition::contains("tags", json!("urgent")) // Utilise l'index TEXT si présent
    ]))
    .sort(vec![/* ... */])
    .limit(20);

// 3. Exécuter (Async)
let results = engine.execute_query(query).await?;
```

---

## 🔧 Pipeline d'Écriture (Détail Technique)

Lors d'un `insert_with_schema` ou d'un commit de transaction :

1. **Chargement** : Le schéma est récupéré depuis le `SchemaRegistry` (mémoire).

2. **Calcul (x_compute)** : Le document est enrichi (ID, dates).

3. **Validation** : Vérification stricte JSON Schema.

4. **WAL** : L'opération est ajoutée au journal des transactions (si mode transactionnel).

5. **Persistance** : Écriture atomique du fichier JSON (`.tmp` → `rename`).

6. **Indexation** : Mise à jour des fichiers `.idx` (Hash/BTree/Text).

7. **Cache Update** : Le cache du `StorageEngine` est invalidé ou mis à jour pour refléter le nouveau fichier dans `_system.json`.

---

## ⚠️ Limitations Actuelles

- **Jointures** : Pas de support natif des jointures (JOIN) dans les requêtes. Les relations sont gérées applicativement ou via des agrégations ultérieures.

- **Migrations** : Le système de migration de schéma est basique (ajout de champs/index), pas de transformations de données complexes en masse intégrées au moteur pour l'instant.

---

## 📝 Métadonnées

**Dernière mise à jour** : Architecture StorageEngine & ACID - Novembre 2025

**Version** : 1.0

**Statut** : Production
