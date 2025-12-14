# Architecture JSON-DB (GenAptitude)

**JSON-DB** est le moteur de base de données embarqué, orienté document et sémantique, développé spécifiquement pour GenAptitude. Il combine la simplicité du stockage de fichiers JSON plats avec la robustesse d'une base de données transactionnelle (ACID) et la puissance du Web Sémantique (JSON-LD).

## 🌍 Vue d'Ensemble

Le système est conçu en couches modulaires, allant du stockage physique bas niveau jusqu'à l'orchestration transactionnelle de haut niveau.

### Principes Clés

- **Stockage Texte** : Chaque document est un fichier `.json` lisible et éditable par un humain.
- **Architecture Sémantique** : Intégration native de JSON-LD pour lier les données à l'ontologie Arcadia (`oa:`, `sa:`, `la:`, etc.).
- **Intégrité ACID** : Support des transactions multi-collections avec journalisation (WAL) et verrouillage.
- **Réactivité** : Moteur de règles intégré (`GenRules`) calculant automatiquement les champs dérivés (prix, dates, statuts).
- **Requêtes SQL** : Moteur de recherche supportant une syntaxe SQL standard pour filtrer et trier les données JSON.

---

## 📂 Arborescence du Code Source

Voici la structure exhaustive des modules et fichiers composants le moteur :

```text
src-tauri/src/json_db/
├── mod.rs                  // Point d'entrée du module global
├── README.md               // Documentation générale (ce fichier)
├── collections/            // Gestion des collections et cycle de vie
│   ├── mod.rs              // Façade publique
│   ├── manager.rs          // Orchestrateur (Règles + Validation + Indexation)
│   ├── collection.rs       // Opérations I/O bas niveau sur les collections
│   └── README.md
├── indexes/                // Moteur d'indexation
│   ├── mod.rs
│   ├── manager.rs          // Gestionnaire du cycle de vie des index (Create/Drop)
│   ├── driver.rs           // Abstraction I/O et formats binaires (Bincode)
│   ├── hash.rs             // Implémentation Index Hash (HashMap)
│   ├── btree.rs            // Implémentation Index BTree (BTreeMap)
│   ├── text.rs             // Implémentation Index Textuel (Inversé)
│   ├── paths.rs            // Gestion des chemins fichiers index
│   └── README.md
├── jsonld/                 // Moteur sémantique
│   ├── mod.rs
│   ├── processor.rs        // Algorithmes Expansion/Compaction/RDF
│   ├── context.rs          // Gestion des contextes (@context)
│   ├── vocabulary.rs       // Registre statique Arcadia (OA, SA, etc.)
│   └── README.md
├── query/                  // Moteur de recherche
│   ├── mod.rs
│   ├── sql.rs              // Parsing SQL (sqlparser)
│   ├── parser.rs           // Parsing JSON Query & Builder
│   ├── optimizer.rs        // Optimisation des plans d'exécution (Sélectivité)
│   ├── executor.rs         // Exécution (Scan, Filter, Sort, Project)
│   └── README.md
├── schema/                 // Validation structurelle
│   ├── mod.rs
│   ├── registry.rs         // Chargement et cache des schémas JSON
│   ├── validator.rs        // Validation récursive (Draft 2020-12 subset)
│   └── README.md
├── storage/                // Persistance physique
│   ├── mod.rs              // Façade StorageEngine
│   ├── file_storage.rs     // I/O atomique et embedded assets
│   ├── cache.rs            // Cache LRU thread-safe
│   ├── compression.rs      // (Placeholder) Compression future
│   └── README.md
└── transactions/           // Moteur ACID
    ├── mod.rs              // Types de transactions (Request, Operation)
    ├── manager.rs          // Gestionnaire de transactions (Execute, Commit)
    ├── wal.rs              // Write-Ahead Log (Journalisation)
    ├── lock_manager.rs     // Gestion des verrous (Collection-level)
    ├── tests.rs            // Tests d'intégration transactionnels
    └── README.md
```

---

## 🧩 Modules du Système

### 1\. Storage (`src/json_db/storage`)

**La Couche Physique.**
Gère l'interaction avec le système de fichiers.

- **Rôle** : Lecture/Écriture atomique des fichiers, gestion des dossiers (DB/Collection), déploiement automatique des schémas par défaut.
- **Performance** : Intègre un cache LRU thread-safe pour accélérer les lectures fréquentes.
- **Sécurité** : Utilise des écritures atomiques (fichier `.tmp` + rename) pour éviter la corruption.

### 2\. Schema (`src/json_db/schema`)

**La Validation Structurelle.**
Garantit que les documents respectent leur contrat d'interface.

- **Rôle** : Validation JSON Schema (Draft 2020-12) légère.
- **Features** : Résolution des références `$ref` via un registre central (`db://...`), validation des types et des motifs (`patternProperties`).

### 3\. JSON-LD (`src/json_db/jsonld`)

**Le Moteur Sémantique.**
Transforme les objets JSON en graphes de connaissances liés.

- **Rôle** : Expansion/Compaction des clés, gestion des contextes (`@context`) et validation ontologique.
- **Ontologie** : Embarque les définitions Arcadia (OA, SA, LA, PA, EPBS, DATA) dans un registre vocabulaire.

### 4\. Indexes (`src/json_db/indexes`)

**L'Accélération.**
Permet des recherches rapides sans scanner tous les fichiers.

- **Types** : Hash (Egalité), BTree (Plages/Tri), Text (Recherche mots-clés).
- **Maintenance** : Mis à jour atomiquement en temps réel lors des écritures via un driver générique.

### 5\. Query (`src/json_db/query`)

**Le Moteur de Recherche.**
Interroge la base de données.

- **Interface** : Supporte SQL (`SELECT * FROM users WHERE age > 18`) et un QueryBuilder fluide.
- **Optimisation** : Réorganise dynamiquement les filtres par sélectivité (coût) pour accélérer l'exécution.

### 6\. Collections (`src/json_db/collections`)

**L'Orchestrateur.**
La façade principale pour manipuler les données.

- **Rôle** : Coordonne le cycle de vie d'un document. C'est ici que réside le moteur de règles **GenRules**.
- **Pipeline** : Injection ID -\> Règles Métier -\> Validation Schema -\> Enrichissement Sémantique -\> Persistance.

### 7\. Transactions (`src/json_db/transactions`)

**La Sécurité des Données.**
Gère les opérations atomiques complexes.

- **ACID** : Utilise un Write-Ahead Log (WAL) pour garantir la durabilité et un LockManager pour l'isolation.
- **Smart API** : Offre des méthodes de haut niveau pour gérer les insertions, mises à jour et imports en masse de manière transactionnelle.

---

## 🔄 Flux de Données (Pipeline d'Écriture)

Lorsqu'une transaction `Insert` ou `Update` est soumise, le document traverse le pipeline suivant :

1.  **Transaction Manager** : Reçoit la requête, acquiert les verrous sur les collections concernées et écrit l'intention dans le WAL.
2.  **Collections Manager** : Prépare le document (injection ID/Dates).
3.  **GenRules Engine** : Exécute les règles métier (`x_rules`) définies dans le schéma pour calculer les champs dérivés.
4.  **Schema Validator** : Vérifie la structure stricte du document.
5.  **JSON-LD Processor** : Vérifie la cohérence sémantique (`@type` connu).
6.  **Storage Engine** : Écrit le fichier JSON de manière atomique sur le disque.
7.  **Index Manager** : Met à jour les index (Hash, BTree, Text) correspondant aux changements.
8.  **Commit** : Si tout est succès, le WAL est nettoyé et les verrous libérés.

---

## 🛠️ Exemple d'Utilisation Globale

Voici comment les modules interagissent pour insérer un utilisateur et le requêter.

```rust
use crate::json_db::storage::JsonDbConfig;
use crate::json_db::transactions::{TransactionManager, TransactionRequest};
use crate::json_db::query::sql::parse_sql;
use crate::json_db::query::QueryEngine;
use crate::json_db::collections::manager::CollectionsManager;
use crate::json_db::storage::StorageEngine;
use serde_json::json;

async fn demo() -> anyhow::Result<()> {
    let config = JsonDbConfig::new("/tmp/genaptitude_data");
    let space = "demo_space";
    let db = "demo_db";

    // 1. Transaction : Insertion sécurisée
    let tx_mgr = TransactionManager::new(&config, space, db);
    tx_mgr.execute_smart(vec![
        TransactionRequest::Insert {
            collection: "users".to_string(),
            id: None, // Auto-généré
            document: json!({
                "name": "Alice",
                "role": "admin",
                "age": 30
            }),
        }
    ]).await?;

    // 2. Requête : Recherche SQL
    let sql = "SELECT name, age FROM users WHERE role = 'admin' ORDER BY age DESC";
    let query = parse_sql(sql)?;

    // 3. Exécution
    let storage = StorageEngine::new(config.clone());
    let col_mgr = CollectionsManager::new(&storage, space, db);
    let engine = QueryEngine::new(&col_mgr);

    let result = engine.execute_query(query).await?;

    println!("Résultats : {:?}", result.documents);
    Ok(())
}
```
