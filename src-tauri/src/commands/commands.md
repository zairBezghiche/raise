# 🎮 Module `commands`

## Vue d'Ensemble

Le module **`commands`** de GenAptitude expose l'API backend Rust au frontend TypeScript/React via le système de commandes Tauri. Il constitue la couche IPC (Inter-Process Communication) entre l'interface utilisateur et les services backend (blockchain, base de données, agents IA, etc.).

### Caractéristiques Principales

**Commandes Blockchain** (14) :

- ✅ Enregistrement de décisions d'architecture
- ✅ Vérification et récupération de décisions
- ✅ Historique complet des décisions
- ✅ Snapshots de modèles Arcadia/Capella
- ✅ Gestion VPN mesh (connexion, statut, peers)
- ✅ Santé système (Fabric + VPN)
- ✅ Utilitaires (logging, hashing)

**Commandes JSON-DB** (15) :

- ✅ CRUD complet (create, read, update, delete)
- ✅ Opérations avec schéma (validation + x_compute)
- ✅ Opérations brutes (sans validation)
- ✅ Listing de collections et documents
- ✅ Queries complexes avec filtres, tri et pagination (Async)

**Commandes Futures** (placeholders) :

- ⚙️ AI Commands : Interaction avec agents LLM
- ⚙️ Code Commands : Génération et analyse de code
- ⚙️ File Commands : Gestion de fichiers système
- ⚙️ Model Commands : Manipulation de modèles MBSE
- ⚙️ Project Commands : Gestion de projets multi-modèles

---

## 🏗️ Architecture Générale

### Structure Modulaire

```
commands/
├── mod.rs                      # Exports publics
├── blockchain_commands.rs      # Commandes Fabric + VPN (289 lignes)
├── json_db_commands.rs         # Commandes base de données (264 lignes)
├── ai_commands.rs              # ⚙️ Placeholder
├── code_commands.rs            # ⚙️ Placeholder
├── file_commands.rs            # ⚙️ Placeholder
├── model_commands.rs           # ⚙️ Placeholder
└── project_commands.rs         # ⚙️ Placeholder
```

### Flux de Communication Tauri

```
┌─────────────────────────────────────────────────────────────┐
│                   Frontend (TypeScript/React)                │
│  invoke("jsondb_query_collection", { ... })                 │
└─────────────────────────┬─────────────────────────────────────┘
                          │ IPC (JSON Serialization)
                          ▼
┌─────────────────────────────────────────────────────────────┐
│              Rust Backend (Commands Module)                  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  #[tauri::command] async fn jsondb_query(...)         │  │
│  └──────────────────────┬────────────────────────────────┘  │
│                         │ (Async/Tokio)                     │
│                         ▼                                   │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  JsonDb Services (Thread-Safe RwLock/Mutex)           │  │
│  │  • CollectionsManager (CRUD synchrone)                │  │
│  │  • QueryEngine (Recherche asynchrone)                 │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## 📚 Module `json_db_commands`

**Responsabilité** : Expose les opérations CRUD et de requête sur la base de données JSON.

### Commandes de Collections

#### `jsondb_create_collection`

Crée une collection vide.

```rust
#[tauri::command]
pub fn jsondb_create_collection(
    space: String,
    db: String,
    collection: String,
) -> Result<(), String>
```

#### `jsondb_list_collections`

Liste toutes les collections disponibles.

```rust
#[tauri::command]
pub fn jsondb_list_collections(
    space: String,
    db: String
) -> Result<Vec<String>, String>
```

---

### Commandes CRUD avec Schéma

Ces commandes appliquent automatiquement :

1.  **x_compute** : Calcul de champs (UUID, timestamps, etc.)
2.  **Validation** : Vérification stricte contre le schéma JSON

#### `jsondb_insert_with_schema`

Insère un document avec validation.

```rust
#[tauri::command]
pub fn jsondb_insert_with_schema(
    space: String,
    db: String,
    schema_rel: String,
    mut doc: Value,
) -> Result<Value, String>
```

**Exemple Frontend** :

```typescript
const actor = {
  handle: 'devops-engineer',
  displayName: 'Ingénieur DevOps',
  // id et dates sont générés automatiquement
};

const stored = await invoke('jsondb_insert_with_schema', {
  space: 'un2',
  db: '_system',
  schemaRel: 'actors/actor.schema.json',
  doc: actor,
});
console.log('Inserted ID:', stored.id);
```

#### `jsondb_upsert_with_schema`

Met à jour si l'ID existe, insère sinon.

```rust
#[tauri::command]
pub fn jsondb_upsert_with_schema(
    space: String,
    db: String,
    schema_rel: String,
    mut doc: Value,
) -> Result<Value, String>
```

---

### Commandes de Requêtes (Async)

#### `jsondb_query_collection`

Exécute une requête complexe avec filtres, tri, et pagination.

```rust
#[tauri::command]
pub async fn jsondb_query_collection(
    space: String,
    db: String,
    bucket: String, // (Legacy param, unused)
    query_json: String,
) -> Result<QueryResult, String>
```

**Note** : L'argument `query_json` est une chaîne JSON représentant l'objet `Query` complet (incluant filtres, tris, etc.).

**Structure de la Requête (JSON)** :

```json
{
  "collection": "articles",
  "filter": {
    "operator": "and",
    "conditions": [{ "field": "status", "operator": "eq", "value": "published" }]
  },
  "sort": [{ "field": "createdAt", "order": "desc" }],
  "limit": 20,
  "offset": 0
}
```

**Exemple Frontend** :

```typescript
const query = {
  collection: 'articles',
  filter: {
    /* ... */
  },
  limit: 10,
};

const result = await invoke('jsondb_query_collection', {
  space: 'un2',
  db: '_system',
  bucket: 'articles', // Paramètre requis par la signature mais ignoré
  queryJson: JSON.stringify(query),
});

console.log(`Found ${result.documents.length} items`);
```

---

## 📚 Module `blockchain_commands`

**Responsabilité** : Expose les opérations Hyperledger Fabric et Innernet VPN.

### Commandes Fabric

#### `record_decision`

Enregistre une décision d'architecture de manière immuable.

```rust
#[tauri::command]
pub async fn record_decision(
    client: State<'_, FabricClient>,
    decision: ArchitectureDecision,
) -> Result<String, String>
```

**Retour** : ID de transaction Fabric.

#### `verify_decision`

Vérifie l'intégrité d'une décision stockée.

```rust
#[tauri::command]
pub async fn verify_decision(
    client: State<'_, FabricClient>,
    decision_id: String,
) -> Result<ArchitectureDecision, String>
```

### Commandes VPN

#### `vpn_connect` / `vpn_disconnect`

Gère la connexion au réseau mesh privé.

```rust
#[tauri::command]
pub async fn vpn_connect(client: State<'_, InnernetClient>) -> Result<String, String>
```

#### `vpn_get_status`

Retourne l'état de la connexion et la liste des peers.

```rust
#[tauri::command]
pub async fn vpn_get_status(client: State<'_, InnernetClient>) -> Result<NetworkStatus, String>
```

---

## 📊 Tableau Récapitulatif des Commandes

### JSON-DB (15 Commandes)

| Commande                    | Type  | Description                   | Async |
| --------------------------- | ----- | ----------------------------- | ----- |
| `jsondb_create_collection`  | Coll  | Crée collection               | ✗     |
| `jsondb_drop_collection`    | Coll  | Supprime collection           | ✗     |
| `jsondb_list_collections`   | Coll  | Liste noms collections        | ✗     |
| `jsondb_insert_with_schema` | CRUD  | Insert validé                 | ✗     |
| `jsondb_upsert_with_schema` | CRUD  | Upsert validé                 | ✗     |
| `jsondb_update_with_schema` | CRUD  | Update validé                 | ✗     |
| `jsondb_insert`             | CRUD  | Insert simple (schema auto)   | ✗     |
| `jsondb_upsert`             | CRUD  | Upsert simple (schema auto)   | ✗     |
| `jsondb_insert_raw`         | CRUD  | Insert brut (sans validation) | ✗     |
| `jsondb_update_raw`         | CRUD  | Update brut (sans validation) | ✗     |
| `jsondb_get`                | CRUD  | Get par ID                    | ✗     |
| `jsondb_delete`             | CRUD  | Delete par ID                 | ✗     |
| `jsondb_list_ids`           | Read  | Liste tous les IDs            | ✗     |
| `jsondb_list_all`           | Read  | Charge tous les docs          | ✗     |
| `jsondb_query_collection`   | Query | Moteur de recherche complet   | ✓     |

### Blockchain & VPN (14 Commandes)

| Commande                 | Type   | Description         | Async |
| ------------------------ | ------ | ------------------- | ----- |
| `record_decision`        | Fabric | Enregistre décision | ✓     |
| `verify_decision`        | Fabric | Vérifie décision    | ✓     |
| `query_decision_history` | Fabric | Historique          | ✓     |
| `record_model_snapshot`  | Fabric | Snapshot modèle     | ✓     |
| `vpn_connect`            | VPN    | Connexion           | ✓     |
| `vpn_disconnect`         | VPN    | Déconnexion         | ✓     |
| `vpn_get_status`         | VPN    | Statut              | ✓     |
| `vpn_list_peers`         | VPN    | Liste peers         | ✓     |
| `vpn_add_peer`           | VPN    | Ajout peer          | ✓     |
| `vpn_ping_peer`          | VPN    | Ping                | ✓     |
| `get_system_health`      | Global | Santé système       | ✓     |
| `init_logging`           | Util   | Init logs           | ✓     |
| `compute_model_hash`     | Util   | Hash SHA-256        | ✓     |

---

## ⚠️ Notes Techniques

1.  **Asynchronisme** :

    - Les commandes marquées `Async` (`✓`) retournent une `Promise` côté JS et ne bloquent pas l'UI.
    - Les commandes CRUD DB (insert, get) sont synchrones (`✗`) côté Rust pour garantir l'atomicité fichier, mais sont invoquées de manière asynchrone par Tauri (`invoke` est toujours async).

2.  **Gestion des Erreurs** :

    - Toutes les commandes retournent `Result<T, String>`.
    - Les erreurs Rust (`anyhow::Error`) sont converties en chaînes pour être affichées dans le frontend.

3.  **État** :
    - `FabricClient` et `InnernetClient` sont injectés via `State<T>`.
    - `CollectionsManager` est instancié à la volée pour chaque commande DB (léger et stateless).

---

**Dernière mise à jour** : Architecture Async/RwLock - Novembre 2025
