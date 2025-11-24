# 🦀 GenAptitude - Backend Rust

Ce répertoire contient le code source **Rust** de l'application GenAptitude (backend Tauri). Il gère la logique métier, la persistance des données, l'IA et la modélisation système.

## 🏗️ Architecture

Le backend est structuré de manière modulaire pour séparer les responsabilités :

```
src-tauri/
├── src/
│   ├── main.rs           # Point d'entrée Tauri (Setup & Run)
│   ├── lib.rs            # Bibliothèque core (exports des modules)
│   ├── commands/         # Interface API exposée au Frontend (tauri::command)
│   ├── json_db/          # Base de données JSON embarquée (Moteur)
│   ├── ai/               # Orchestration IA Multi-Agents
│   ├── model_engine/     # Moteur de modélisation Arcadia/Capella
│   └── ...
├── tests/                # Suites de tests d'intégration (json_db_suite)
└── tools/                # Outils CLI (jsondb_cli)
```

---

## 📦 Modules Principaux

### 1. Base de Données (`json_db`)

Le cœur du système de persistance. C'est une base de données NoSQL orientée documents, stockée sous forme de fichiers JSON, mais avec des garanties fortes.

- **Architecture** : Asynchrone (`tokio`) et Thread-Safe (`RwLock`).
- **Validation** : Utilise **JSON Schema** pour valider strictement chaque document avant écriture.
- **x_compute** : Système de champs calculés (UUID, timestamps, liens) exécuté côté backend.
- **Stockage** : Hiérarchie `Space` > `Database` > `Collection`. Écritures atomiques (pas de corruption).
- **Requêtes** : Moteur de requêtes (`QueryEngine`) supportant filtres complexes, tris et pagination.

### 2. Interface Frontend (`commands`)

Ce module fait le pont entre l'interface React (TypeScript) et le code Rust.
Toutes les fonctions ici sont asynchrones (`async fn`) et retournent des `Result` gérés par Tauri.

- Les commandes `json_db_commands.rs` exposent le CRUD et le `QueryEngine` au frontend.

### 3. Intelligence Artificielle (`ai`)

- Gestion des **Agents Spécialisés** (System Engineer, Software Architect, etc.).
- Gestion du contexte et des prompts.
- (En cours) Intégration RAG (Retrieval Augmented Generation) avec la `json_db`.

---

## 🛠️ Développement

### Pré-requis

- Rust (édition 2021)
- Node.js / Bun (pour le frontend)
- Variables d'environnement configurées (voir `.env`).

### Tests

Le projet dispose d'une suite de tests rigoureuse, particulièrement pour la base de données.

```bash
# Lancer tous les tests (Unitaires + Intégration)
cargo test

# Lancer uniquement la suite d'intégration de la DB
cargo test --test json_db_suite

# Lancer un test spécifique avec les logs activés
RUST_LOG=debug cargo test --test json_db_suite -- query_find_many --nocapture
```

**Note sur les tests d'intégration :**
Les tests `json_db_suite` créent des environnements temporaires isolés (`/tmp/jsondb_ut_...`) et chargent de vrais datasets (`PATH_GENAPTITUDE_DATASET`) pour valider le comportement réel du moteur.

### CLI (`jsondb_cli`)

Un outil en ligne de commande est disponible dans `tools/jsondb_cli` pour administrer la base de données sans lancer l'interface graphique.

```bash
# Build et utilisation
cd tools/jsondb_cli
cargo run -- query find-many un2 _system my_query.json
```

---

## 🧩 Patterns de Code

### Gestion de la Concurrence (`json_db`)

Si vous devez modifier le cœur de la DB, notez que :

- Le `CollectionsManager` est conçu pour être partagé (`Arc<CollectionsManager>` ou instancié à la volée).
- L'accès au `SchemaRegistry` est protégé par un **`RwLock`**. Utilisez les méthodes internes `get_registry_guard()` pour y accéder.
- Toutes les I/O disques sont (pour l'instant) synchrones pour garantir l'atomicité, mais enveloppées dans des commandes `async` pour ne pas bloquer l'UI Tauri.

### Gestion des Erreurs

Nous utilisons la crate **`anyhow`** pour la propagation des erreurs dans le backend, qui sont ensuite sérialisées en chaînes de caractères pour le frontend via Tauri.

---

**Dernière mise à jour** : Refactoring Async/Thread-Safe - Novembre 2025
