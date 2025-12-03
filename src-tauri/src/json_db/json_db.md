# Module json_db

> **Version :** 1.2
> **Mise à jour :** Novembre 2025
> **Nouveautés :** StorageEngine, Transactions ACID, Indexation Binaire, **Couche Sémantique JSON-LD**.

---

## 📦 Vue d'Ensemble

Le module **json_db** est une base de données orientée documents JSON souveraine, transactionnelle et sémantique. Elle constitue le socle de persistance de la plateforme GenAptitude.

Elle ne se contente pas de stocker des données : elle structure la connaissance via des **schémas JSON** stricts et assure l'interopérabilité métier via **JSON-LD**.

### Caractéristiques Principales

- **Stockage Souverain** : Données stockées en fichiers JSON lisibles, organisés hiérarchiquement.
- **Transactions ACID** : Atomicité et durabilité garanties par un **Write-Ahead Log (WAL)** (`_wal.jsonl`).
- **Performance (StorageEngine)** : Cache en mémoire thread-safe (`Arc<RwLock>`) pour les lectures haute performance.
- **Indexation Hybride** : Index Hash, B-Tree et Textuels maintenus en temps réel.
- **Moteur x_compute** : Calcul automatique de champs (UUID, dates) avant validation.
- **Sémantique Forte (Nouveau)** : Adhésion stricte au méta-modèle Arcadia via l'expansion JSON-LD, permettant au `ModelLoader` de typer fortement les données.

---

## 🏗️ Architecture Générale

### Arborescence Physique

Structure définie par `PATH_GENAPTITUDE_DOMAIN` :

```

\<domain_root\>/
├── \<space\>/ \# Espace (ex: "un2")
│ ├── \<database\>/ \# Base (ex: "\_system")
│ │ ├── \_system.json \# Manifeste DB
│ │ ├── \_wal.jsonl \# Journal des transactions
│ │ ├── schemas/v1/ \# Registre des schémas JSON
│ │ └── collections/
│ │ └── \<collection\>/
│ │ ├── \_config.json
│ │ ├── \_indexes/ \# Index binaires (.idx)
│ │ ├── \<uuid\>.json \# Documents (JSON-LD compact)
│ │ └── ...

```

### Composants Clés

1.  **StorageEngine** : Cœur de l'état partagé. Gère la configuration et le cache.
2.  **CollectionsManager** : Façade CRUD. Coordonne `x_compute`, validation de schéma et persistance.
3.  **TransactionManager** : Gère les blocs atomiques et le WAL.
4.  **QueryEngine** : Moteur de recherche asynchrone avec optimiseur.
5.  **JsonLdProcessor** : Moteur sémantique gérant l'expansion et la compaction des types (`oa:Actor` ↔ URI canonique).

---

## 🔗 Intégration Sémantique & Model Engine

C'est l'évolution majeure de la version 1.2. La base de données ne stocke pas seulement des objets JSON, mais des **Concepts Métier**.

### 1. Le Vocabulaire Centralisé

Pour éviter les "chaînes magiques", tous les types Arcadia sont définis dans `vocabulary.rs`.
Exemple : `arcadia_types::OA_ACTOR` = `"OperationalActor"`.

### 2. Le Flux Sémantique

Lorsqu'un document est chargé par le `ModelLoader` :

1.  **Lecture Brute** : Le JSON stocké est lu (souvent sous forme compacte avec préfixes).
    ```json
    { "@type": "oa:OperationalActor", "name": "User" }
    ```
2.  **Expansion JSON-LD** : Le `JsonLdProcessor` utilise les contextes pour résoudre les URIs complètes.
    ```json
    { "@type": ["[https://genaptitude.io/ontology/arcadia/oa#OperationalActor](https://genaptitude.io/ontology/arcadia/oa#OperationalActor)"], ... }
    ```
3.  **Dispatch Typé** : Le `ModelLoader` compare l'URI obtenue avec le vocabulaire officiel pour instancier la bonne structure Rust (`OperationalAnalysis`, `SystemAnalysis`, etc.).

### 3. Structure en Mémoire (`ProjectModel`)

Les données de la DB sont projetées en mémoire dans une structure fortement typée :

```rust
pub struct ProjectModel {
    pub oa: OperationalAnalysis, // Contient Vec<ArcadiaElement> pour OA
    pub sa: SystemAnalysis,      // Contient Vec<ArcadiaElement> pour SA
    pub la: LogicalArchitecture,
    pub pa: PhysicalArchitecture,
    pub epbs: EPBS,
    pub meta: ProjectMeta,
}
```

---

## 📚 Modules Détaillés

### 1\. Transactions (`transactions/`)

Assure que toutes les modifications d'un bloc sont appliquées ou aucune.

- **WAL** : Écriture séquentielle avant modification disque.
- **Recovery** : Rejoue les transactions non committées au démarrage.

### 2\. Indexation (`indexes/`)

- **Hash** : Recherche exacte (`eq`).
- **BTree** : Recherche par plage (`gt`, `lt`) et tri.
- **Text** : Recherche plein texte basique (`contains`).
- **Persistance** : Format binaire `bincode` pour rapidité de chargement.

### 3\. Requêtes (`query/`)

- **Optimiseur** : Réorganise les filtres par sélectivité.
- **Exécuteur** : Choisit entre Index Scan et Full Scan.

### 4\. Schémas (`schema/`)

- **SchemaRegistry** : Cache les fichiers de schéma.
- **Validator** : Validation stricte JSON Schema (Draft 2020-12).
- **Compute** : Moteur de règles pour générer les métadonnées techniques (`id`, `createdAt`) avant insertion.

---

## 💡 Guide d'Utilisation (Rust)

### Insertion (Avec validation sémantique)

```rust
// Le document utilise un contexte JSON-LD pour abréger les types
let doc = json!({
    "@context": { "oa": "[https://genaptitude.io/ontology/arcadia/oa#](https://genaptitude.io/ontology/arcadia/oa#)" },
    "@type": "oa:OperationalActor",
    "name": "Opérateur"
});

// insert_with_schema va :
// 1. Calculer l'ID et les dates
// 2. Valider contre le schéma "actor.schema.json"
// 3. Persister le JSON
mgr.insert_with_schema("actors", doc)?;
```

### Chargement du Modèle Complet

Pour travailler sur le projet, on charge tout en mémoire via le `ModelLoader` qui fait le lien sémantique.

```rust
// Utilisation du constructeur découplé (recommandé)
let loader = ModelLoader::from_engine(&storage, "space", "db");

// Charge et dispatch sémantiquement tous les éléments dans les bonnes couches
let project = loader.load_full_model()?;

println!("Acteurs OA : {}", project.oa.actors.len());
println!("Fonctions SA : {}", project.sa.functions.len());
```

---

## ⚠️ Limitations et Bonnes Pratiques

1.  **Contextes JSON-LD** : Assurez-vous que vos documents (ou schémas) définissent correctement `@context` pour que l'expansion fonctionne. Le système fournit des contextes par défaut.
2.  **Performance** : Le chargement complet (`load_full_model`) est une opération coûteuse (I/O). Elle doit être exécutée dans un thread bloquant (`spawn_blocking`) pour ne pas figer l'interface Tauri.
3.  **Migration** : En cas de changement de modèle de données (nouveaux champs), utilisez les migrations intégrées plutôt que de modifier les fichiers JSON à la main.

---

**Statut :** Production  
**Intégration :** Prêt pour le module IA (Agents)

```

```
