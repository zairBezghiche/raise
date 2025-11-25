# 🦀 GenAptitude - Backend Rust (Tauri)

Ce dossier contient le code source **Rust** de l'application GenAptitude. Il gère la logique métier critique, la persistance des données, l'IA et les communications sécurisées.

## 🏗️ Architecture Modulaire

[cite_start]Le code est organisé en modules distincts exposés via `lib.rs`[cite: 14]:

### 1. `json_db` (Persistance Avancée)

Moteur de base de données NoSQL transactionnel conçu sur mesure pour garantir l'intégrité des données d'ingénierie.

- **`collections/`** : Gestionnaire haut niveau (CRUD). [cite_start]Gère le cycle de vie des fichiers JSON et expose une API thread-safe (`CollectionsManager`)[cite: 13].
- **`transactions/`** : Moteur ACID. [cite_start]Utilise un **Write-Ahead Log (WAL)** (`_wal.jsonl`) pour garantir l'atomicité des opérations multi-documents (`ActiveTransaction`).
- [cite_start]**`indexes/`** : Moteur d'indexation (Hash, BTree, Text) maintenu en mémoire pour accélérer les lectures, avec persistance via `bincode`.
- [cite_start]**`schema/`** : Registre de schémas (`SchemaRegistry`) et moteur `x_compute` pour les champs calculés (UUID, timestamps, pointeurs)[cite: 928].
- [cite_start]**`query/`** : Moteur de requête (`QueryExecutor`) avec optimiseur, supportant les filtres complexes JSON[cite: 699].

### 2. `blockchain` (Souveraineté)

[cite_start]Gestion de la sécurité distribuée et du réseau[cite: 510].

- **`fabric/`** : Client gRPC pour Hyperledger Fabric. [cite_start]Permet de signer et soumettre des transactions (`RecordDecision`) localement via des identités MSP.
- **`vpn/`** : Wrapper pour **Innernet** (WireGuard). [cite_start]Gère la création d'interfaces réseau mesh (`genaptitude0`) pour la communication P2P.

### 3. `ai` (Intelligence Artificielle)

[cite_start]Orchestrateur Neuro-Symbolique[cite: 571].

- [cite_start]**`agents/`** : Implémentation des agents spécialisés (`HardwareAgent`, `SoftwareAgent`, `SystemAgent`) et classificateur d'intentions[cite: 12].
- **`nlp/`** : Pipeline d'extraction d'entités et analyse syntaxique.
- **`llm/`** : Client d'inférence pour modèles locaux.

### 4. `model_engine` (MBSE)

[cite_start]Manipulation des modèles d'ingénierie[cite: 15].

- **`arcadia/`** : Structures de données pour les couches Arcadia (OA, SA, LA, PA, EPBS).
- **`capella/`** : Parsers et générateurs pour l'interopérabilité Capella (XML/XMI).
- **`validators/`** : Vérification de cohérence et compliance (ISO-26262, DO-178C).

---

## 🛠️ Commandes Tauri (`src/commands`)

[cite_start]L'API exposée au frontend est définie dans les modules suivants[cite: 539]:

### [cite_start]Base de données (`json_db_commands.rs`) [cite: 553]

- `jsondb_insert_with_schema` : Création avec validation et calcul automatique.
- `jsondb_execute_transaction` : Exécution atomique d'un lot d'opérations (Insert/Update/Delete).
- `jsondb_query_collection` : Recherche avancée avec filtres et tri.

### [cite_start]Blockchain & Réseau (`blockchain_commands.rs`) [cite: 565]

- `record_decision` : Ancrage d'une décision sur la blockchain.
- `vpn_connect` / `vpn_get_status` : Gestion de la connexion au réseau privé.

---

## 🧪 Tests

[cite_start]Le projet inclut une suite de tests d'intégration complète (`tests/json_db_suite.rs`)[cite: 406]:

```bash
# Lancer tous les tests d'intégration DB (Cycle de vie, ACID, x_compute)
cargo test --test json_db_suite

# Lancer un test spécifique pour le debug
RUST_LOG=debug cargo test --test json_db_suite -- transaction_commit_success
```
