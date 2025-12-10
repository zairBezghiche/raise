### Fichier : `src-tauri/README.md`

````markdown
# 🦀 GenAptitude - Backend Rust (Tauri Core)

Le cœur de GenAptitude est une application **Rust** haute performance utilisant le framework **Tauri v2**.
Il agit comme un serveur local sécurisé gérant la logique métier lourde, le stockage des données, l'intelligence artificielle et l'exécution de plugins.

## 🏗 Architecture Modulaire

Le backend est découpé en modules distincts (Crates internes ou modules) pour une séparation stricte des responsabilités.

```text
src-tauri/src/
├── ai/                 # 🤖 Noyau IA (LLM, NLP, RAG, Agents)
├── blockchain/         # 🔗 Connecteurs Infrastructure (Hyperledger Fabric, WireGuard)
├── code_generator/     # ⚡ Usine Logicielle (Moteur de templates Tera)
├── commands/           # 🔌 Couche d'exposition IPC (Commandes Tauri)
├── genetics/           # 🧬 Moteur d'Optimisation (Algorithmes Évolutionnaires)
├── json_db/            # 🗄️ Base de Données NoSQL Native (Storage & Query)
├── model_engine/       # 📚 Moteur Sémantique (Chargement Arcadia/Capella)
├── plugins/            # 🧠 Hôte WASM (Wasmtime Runtime)
├── lib.rs              # Point d'entrée de la librairie
└── main.rs             # Point d'entrée de l'exécutable (Setup Tauri)
```
````

---

## 🧩 Détail des Modules

### 1\. 🗄️ JSON-DB (Base de Données)

Moteur NoSQL orienté document, écrit en Rust pur.

- **Storage** : Fichiers atomiques (`_system.json`, `collections/`).
- **Fonctions** : Indexation (Hash/BTree), Schémas JSON stricts, Requêtes SQL (Subset).
- **CLI** : Outil d'administration en ligne de commande (voir section dédiée plus bas).

### 2\. 🤖 AI Kernel

Un orchestrateur d'intelligence artificielle local et cloud.

- **LLM** : Client abstrait pour OpenAI ou Ollama local.
- **RAG** : Système de contexte vectoriel pour injecter la documentation technique.
- **Agents** : Système multi-agents pour la spécialisation des tâches (Architecte, Reviewer).

### 3\. 🧠 Cognitive Host (WASM)

Un environnement "Sandbox" sécurisé utilisant **Wasmtime**.

- **Rôle** : Charge dynamiquement des fichiers `.wasm` (situés dans `wasm-modules/`) pour exécuter des règles de validation métier sans recompiler le backend.
- **Performance** : Exécution native proche du C/Rust.

### 4\. 📚 Model Engine

Le chargeur sémantique pour les modèles d'ingénierie (Arcadia).

- **Fonction** : Lit les données brutes de la DB et construit un graphe d'objets typés (OA, SA, LA, PA, EPBS).
- **Usage** : Sert de source de vérité pour le Frontend et les Générateurs.

### 5\. 🧬 Genetics Engine

Module de calcul intensif (CPU Bound).

- **Fonction** : Exécute des algorithmes évolutionnaires pour explorer l'espace de conception.
- **Processus** : Simulation de générations, mutations et sélections pour optimiser des critères (coût, performance).

### 6\. ⚡ Code Generator

Moteur de génération de code source.

- **Techno** : Utilise le moteur de templates **Tera** (similaire à Jinja2).
- **Sortie** : Génère du code Rust, Python ou C++ à partir du Modèle Système.

### 7\. 🔗 Blockchain & Network

- **WireGuard** : Monitoring de l'état du VPN et des pairs.
- **Hyperledger** : Soumission et requête de transactions de traçabilité.

---

## 🛠 Administration JSON-DB (CLI)

L'outil `jsondb_cli` permet d'administrer la base sans passer par l'interface graphique.

### Commandes de Base

```bash
# 1. Création d'une base (Structure + Schémas standards)
cargo run -p jsondb_cli -- --space un2 --db _system create-db

# 2. Suppression d'une base (Irréversible)
cargo run -p jsondb_cli -- --space un2 --db _system drop-db --force
```

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

Le moteur supporte un sous-ensemble du SQL pour le requêtage.

```bash
cargo run -p jsondb_cli -- --space un2 --db _system sql \
  --query "SELECT displayName, handle FROM articles WHERE handle = 'test-1'"
```

---

## ✅ Tests et Qualité

Le backend est couvert par des tests unitaires et des suites d'intégration.

### Lancer les tests

```bash
# 1. Lancer tous les tests (Unitaires + Intégration)
cargo test

# 2. Lancer uniquement la suite JSON-DB
cargo test --test json_db_suite

# 3. Lancer uniquement les tests du moteur IA
cargo test ai::
```

### Vérification du code

```bash
# Vérification rapide de compilation
cargo check

# Analyse statique (Linter)
cargo clippy
```

---

## 🚀 Développement

Pour ajouter une nouvelle fonctionnalité :

1.  Créer la logique métier dans son module dédié (ex: `src/mon_module/mod.rs`).
2.  Créer une commande Tauri asynchrone dans `src/commands/mon_module_commands.rs`.
3.  Enregistrer la commande dans `src/commands/mod.rs`.
4.  Exposer la commande dans `src/main.rs` via `.invoke_handler()`.

<!-- end list -->

```

```
