### 1\. Fichier : `src-tauri/README.md`

Copiez ce contenu et remplacez celui de `src-tauri/README.md`.

````markdown
# GenAptitude - Backend JSON-DB

Moteur de base de données NoSQL orienté document, écrit en Rust, avec support des schémas JSON stricts, de l'indexation et des requêtes SQL.

## 🏗 Architecture

- **Storage** : Stockage fichier atomique (`_system.json`, `collections/`, `_meta.json`).
- **Collections** : Gestionnaire CRUD avec validation de schéma (`$schema`).
- **Indexes** : Moteur d'indexation modulaire (Hash, BTree, Text).
- **Query Engine** : Support des filtres complexes et d'un sous-ensemble SQL.

## 🛠 Utilisation du CLI (`jsondb_cli`)

L'outil CLI permet d'administrer la base sans passer par l'interface graphique.

### Commandes de Base

```bash
# 1. Création d'une base (Structure + Schémas standards)
cargo run -p jsondb_cli -- --space un2 --db _system create-db

# 2. Suppression d'une base (Irréversible)
cargo run -p jsondb_cli -- --space un2 --db _system drop-db --force
```
````

### Gestion des Données

```bash
# Insertion (Validation stricte selon le schéma)
cargo run -p jsondb_cli -- --space un2 --db _system insert \
  --collection articles \
  --data '{ "handle": "test-1", "slug": "test-1", "title": "Titre", "displayName": "Display", "status": "draft" }'

# Lecture
cargo run -p jsondb_cli -- --space un2 --db _system list --collection articles
```

### Indexation & Performance

```bash
# Créer un index (Hash) sur un champ
cargo run -p jsondb_cli -- --space un2 --db _system create-index \
  --collection articles --field handle --kind hash

# Supprimer un index
cargo run -p jsondb_cli -- --space un2 --db _system drop-index \
  --collection articles --field handle
```

### Requêtes SQL

```bash
cargo run -p jsondb_cli -- --space un2 --db _system sql \
  --query "SELECT displayName, handle FROM articles WHERE handle = 'test-1'"
```

## ✅ Tests

Le backend est couvert à 100% par des tests unitaires et d'intégration.

```bash
# Lancer toute la suite de tests
cargo test

# Lancer uniquement la suite JSON-DB
cargo test --test json_db_suite
```
