# Commandes Tauri : json_db

> Cette documentation détaille l'API Tauri exposée pour la base de données, prenant en compte l'intégration du StorageEngine (pour le cache partagé) et l'ajout des Transactions ACID.

---

## 🔌 Vue d'Ensemble

Ce document détaille les commandes exposées par le module **json_db** au frontend. Ces commandes permettent d'interagir avec la base de données locale de manière sécurisée et performante.

> **Note Architecture** : Toutes les commandes utilisent désormais `State<StorageEngine>` pour bénéficier du cache en mémoire des index et des schémas.

---

## 1. Collections

Gestion du cycle de vie des collections.

### `jsondb_create_collection`

Crée une nouvelle collection et initialise son index.

**Signature :**
```rust
fn(space: String, db: String, collection: String, schema: Option<String>) -> Result<(), String>
```

**Usage :**
```typescript
await invoke('jsondb_create_collection', {
  space: 'un2',
  db: '_system',
  collection: 'projects',
  schema: 'projects/project.schema.json' // Optionnel
});
```

### `jsondb_list_collections`

Liste les noms de toutes les collections disponibles dans une base.

**Signature :**
```rust
fn(space: String, db: String) -> Result<Vec<String>, String>
```

**Retour :**
```json
["actors", "projects", "tasks", ...]
```

### `jsondb_drop_collection`

Supprime définitivement une collection et tous ses fichiers sur le disque.

**Signature :**
```rust
fn(space: String, db: String, collection: String) -> Result<(), String>
```

---

## 2. CRUD (Opérations Unitaires)

Ces opérations sont atomiques au niveau du fichier document.

### `jsondb_insert_with_schema`

Insère un document en appliquant le pipeline complet : **x_compute** (calcul automatique) ➜ **Validation** ➜ **Persistance**.

**Signature :**
```rust
fn(space: String, db: String, schema_rel: String, doc: Value) -> Result<Value, String>
```

**Arguments :**
- `schema_rel` : Chemin relatif du schéma (ex: `"actors/actor.schema.json"`). La collection cible est déduite de ce chemin.

**Retour :** Le document complet tel qu'enregistré (avec `id`, `createdAt`, `updatedAt` générés).

**Exemple :**
```typescript
const doc = { name: "Nouveau Projet", status: "active" };
const result = await invoke('jsondb_insert_with_schema', {
  space: 'un2',
  db: '_system',
  schemaRel: 'projects/project.schema.json',
  doc: doc
});
// result: { id: "uuid...", name: "Nouveau Projet", createdAt: "2025-11-27...", ... }
```

### `jsondb_upsert_with_schema`

Si l'ID existe déjà, effectue une mise à jour (remplacement). Sinon, insère le document.

**Signature :**
```rust
fn(space: String, db: String, schema_rel: String, doc: Value) -> Result<Value, String>
```

### `jsondb_update_with_schema`

Met à jour un document existant. Échoue si l'ID n'existe pas.

**Signature :**
```rust
fn(space: String, db: String, schema_rel: String, doc: Value) -> Result<Value, String>
```

### `jsondb_get`

Récupère un document par son ID.

**Signature :**
```rust
fn(space: String, db: String, collection: String, id: String) -> Result<Value, String>
```

**Exemple :**
```typescript
const doc = await invoke('jsondb_get', {
  space: 'un2',
  db: '_system',
  collection: 'projects',
  id: 'urn:uuid:abc-123'
});
```

### `jsondb_delete`

Supprime un document par son ID.

**Signature :**
```rust
fn(space: String, db: String, collection: String, id: String) -> Result<(), String>
```

---

## 3. Moteur de Requêtes (Search)

Recherche avancée avec filtrage, tri et pagination. Utilise le **QueryEngine** pour optimiser l'exécution (utilisation des index Hash/Text/BTree si disponibles).

### `jsondb_query_collection`

**Signature :**
```rust
async fn(space: String, db: String, _bucket: String, query_json: String) -> Result<QueryResult, String>
```

**Arguments :**
- `query_json` : Une chaîne JSON représentant l'objet `Query` complet.

**Retour :**
```rust
QueryResult { 
    documents: Vec<Value>, 
    total: u64, 
    ... 
}
```

### Structure de Query

```typescript
interface Query {
  collection: string;
  filter?: QueryFilter;
  sort?: SortField[];
  limit?: number;
  offset?: number;
}

interface QueryFilter {
  operator: "and" | "or";
  conditions: Condition[];
}

interface Condition {
  field: string;
  operator: "eq" | "ne" | "gt" | "gte" | "lt" | "lte" | "contains";
  value: any;
}

interface SortField {
  field: string;
  order: "asc" | "desc";
}
```

### Exemple d'Utilisation (Frontend)

```typescript
const query = {
  collection: "tasks",
  filter: {
    operator: "and",
    conditions: [
      { field: "status", operator: "eq", value: "pending" },
      { field: "priority", operator: "gte", value: 5 }
    ]
  },
  sort: [{ field: "updatedAt", order: "desc" }],
  limit: 20,
  offset: 0
};

const result = await invoke('jsondb_query_collection', {
  space: 'un2',
  db: '_system',
  _bucket: 'tasks', // Placeholder
  queryJson: JSON.stringify(query)
});

console.log(`Trouvé ${result.total} documents`);
result.documents.forEach(doc => {
  console.log(doc.name, doc.status);
});
```

---

## 4. Transactions (ACID)

Permet d'exécuter un lot d'opérations de manière atomique : tout réussit ou rien n'est appliqué.

### `jsondb_execute_transaction`

Exécute une liste d'opérations (Insert, Update, Delete) séquentiellement.

**Signature :**
```rust
fn(space: String, db: String, request: TransactionRequest) -> Result<(), String>
```

### Structure TransactionRequest

```rust
pub struct TransactionRequest {
    pub operations: Vec<OperationRequest>,
}

pub enum OperationRequest {
    Insert { 
        collection: String, 
        doc: Value 
    }, // ID généré si absent
    Update { 
        collection: String, 
        doc: Value 
    }, // ID requis
    Delete { 
        collection: String, 
        id: String 
    },
}
```

### Exemple d'Utilisation (Frontend)

```typescript
const transaction = {
  operations: [
    { 
      type: "insert", 
      collection: "logs", 
      doc: { 
        message: "Début traitement", 
        level: "info" 
      } 
    },
    { 
      type: "update", 
      collection: "users", 
      doc: { 
        id: "user-123", 
        status: "active" 
      } 
    },
    {
      type: "delete",
      collection: "temp_data",
      id: "temp-456"
    }
  ]
};

try {
  await invoke('jsondb_execute_transaction', {
    space: 'un2',
    db: '_system',
    request: transaction
  });
  console.log("✅ Transaction committed!");
} catch (e) {
  console.error("❌ Transaction failed (Rollback effectué):", e);
}
```

### Garanties ACID

- **Atomicité** : Toutes les opérations réussissent ou aucune n'est appliquée
- **Cohérence** : La validation des schémas est maintenue
- **Isolation** : Les verrous empêchent les accès concurrents
- **Durabilité** : Le WAL garantit la récupération après crash

---

## 5. Utilitaires

### `jsondb_list_ids`

Retourne uniquement les IDs (noms de fichiers sans extension) d'une collection. Très rapide (scan répertoire uniquement).

**Signature :**
```rust
fn(space: String, db: String, collection: String) -> Result<Vec<String>, String>
```

**Exemple :**
```typescript
const ids = await invoke('jsondb_list_ids', {
  space: 'un2',
  db: '_system',
  collection: 'projects'
});
// ["urn:uuid:abc-123", "urn:uuid:def-456", ...]
```

### `jsondb_list_all`

Charge tous les documents d'une collection.

⚠️ **Performance** : À utiliser uniquement pour les petites collections (< 1000 éléments) ou pour l'export.

**Signature :**
```rust
fn(space: String, db: String, collection: String) -> Result<Vec<Value>, String>
```

**Exemple :**
```typescript
const allDocs = await invoke('jsondb_list_all', {
  space: 'un2',
  db: '_system',
  collection: 'settings'
});
```

### `jsondb_refresh_registry`

Force le rechargement du registre de schémas depuis le disque (invalidation du cache interne). Utile lors du développement de schémas.

**Signature :**
```rust
fn(space: String, db: String) -> Result<(), String>
```

**Exemple :**
```typescript
// Après avoir modifié un schéma sur le disque
await invoke('jsondb_refresh_registry', {
  space: 'un2',
  db: '_system'
});
console.log("✅ Registre de schémas rechargé");
```

---

## 📊 Récapitulatif des Commandes

| Catégorie | Commande | Description |
|-----------|----------|-------------|
| **Collections** | `jsondb_create_collection` | Crée une nouvelle collection |
| | `jsondb_list_collections` | Liste toutes les collections |
| | `jsondb_drop_collection` | Supprime une collection |
| **CRUD** | `jsondb_insert_with_schema` | Insère un nouveau document |
| | `jsondb_upsert_with_schema` | Insère ou met à jour |
| | `jsondb_update_with_schema` | Met à jour un document existant |
| | `jsondb_get` | Récupère un document par ID |
| | `jsondb_delete` | Supprime un document |
| **Requêtes** | `jsondb_query_collection` | Recherche avancée avec filtres |
| **Transactions** | `jsondb_execute_transaction` | Opérations atomiques multiples |
| **Utilitaires** | `jsondb_list_ids` | Liste rapide des IDs |
| | `jsondb_list_all` | Charge tous les documents |
| | `jsondb_refresh_registry` | Recharge les schémas |

---

## 🔒 Bonnes Pratiques

1. **Validation** : Toujours utiliser les commandes `*_with_schema` pour garantir l'intégrité des données.

2. **Transactions** : Utiliser `jsondb_execute_transaction` pour les opérations multi-documents critiques.

3. **Performance** : 
   - Préférer `jsondb_list_ids` à `jsondb_list_all` quand seuls les IDs sont nécessaires
   - Utiliser les index (Hash/BTree/Text) pour optimiser les requêtes fréquentes

4. **Cache** : Le `StorageEngine` met automatiquement en cache les schémas et configurations. Utiliser `jsondb_refresh_registry` uniquement en développement.

5. **Gestion d'erreurs** : Toutes les commandes retournent `Result<T, String>`. Toujours gérer les erreurs côté frontend.

---

## 📝 Métadonnées

**Version** : 1.0  
**Dernière mise à jour** : Novembre 2025  
**Statut** : Production  
**Dépendances** : Tauri v2, StorageEngine, QueryEngine, TransactionManager
