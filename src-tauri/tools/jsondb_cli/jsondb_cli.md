# Guide d'utilisation de la CLI jsondb_cli

> Cette documentation intègre les nouvelles fonctionnalités découvertes dans le code source, notamment le support des Transactions ACID (avec le type d'opération `insertFrom` spécifique à la CLI), les options de Requêtes Ad-Hoc, et précise la configuration de l'environnement.

---

## 📚 Vue d'Ensemble

La CLI **jsondb_cli** est l'outil d'administration en ligne de commande pour la base de données JSON de GenAptitude. Elle permet de gérer l'environnement, les collections, les documents, et d'exécuter des transactions ACID ou des requêtes complexes.

---

## ⚙️ Configuration et Environnement

Avant d'utiliser la CLI, assurez-vous que les variables d'environnement sont définies (via un fichier `.env` ou l'export shell) :

| Variable | Description |
|----------|-------------|
| **`PATH_GENAPTITUDE_DOMAIN`** | **Requis.** Chemin racine où les bases de données sont stockées (ex: `$HOME/genaptitude_domain`). |
| **`PATH_GENAPTITUDE_DATASET`** | *(Optionnel)* Chemin racine pour les datasets utilisés par `seed-dir`. |
| **`RUST_LOG`** | *(Optionnel)* Niveau de log (ex: `info` ou `debug`). |

**Exemple de configuration :**

```bash
export PATH_GENAPTITUDE_DOMAIN="$HOME/genaptitude_domain"
export PATH_GENAPTITUDE_DATASET="$HOME/datasets"
export RUST_LOG="info"
```

---

## 🔧 Structure Générale

```bash
jsondb_cli [OPTIONS] <MODULE> <ACTION> [ARGUMENTS]
```

### Options Globales

Ces options doivent être placées **avant** la sous-commande (`<MODULE>`).

| Option | Description | Exemple |
|--------|-------------|---------|
| **`--repo-root`** | Spécifie explicitement la racine du dépôt (pour localiser `schemas/v1`). Par défaut : dossier courant. | `jsondb_cli --repo-root .. db create ...` |

---

## 1. Gestion des Bases de Données (`db`)

Gestion du cycle de vie physique et interrogation rapide.

| Action | Description | Arguments | Exemple |
|--------|-------------|-----------|---------|
| **`create`** | Crée une DB et initialise sa structure. | `<space> <db>` | `jsondb_cli db create un2 _system` |
| **`open`** | Vérifie l'existence et l'intégrité. | `<space> <db>` | `jsondb_cli db open un2 _system` |
| **`drop`** | Supprime la DB. | `<space> <db> [--hard]` | `jsondb_cli db drop un2 _system --hard` |
| **`query`** | Requête Ad-Hoc sur une collection. | `<space> <db> <coll> [OPTS]` | *(Voir détails ci-dessous)* |

### 🔍 Détail de la commande `db query`

Permet d'interroger une collection sans créer de fichier JSON de requête.

**Options :**

| Option | Description |
|--------|-------------|
| **`--filter-json <JSON>`** | Filtre au format JSON (ex: `{"op":"eq",...}`). |
| **`--sort <field>:<asc\|desc>`** | Tri (répétable). Peut utiliser `+field` ou `-field`. |
| **`--limit <N>`** | Limite de résultats. |
| **`--offset <N>`** | Pagination (décalage). |
| **`--latest`** | Raccourci pour trier par `createdAt:desc`. |

**Exemple :**

```bash
jsondb_cli db query un2 _system articles \
  --filter-json '{"op":"eq","field":"status","value":"published"}' \
  --sort title:asc \
  --limit 5
```

**Exemple avec tri multiple :**

```bash
jsondb_cli db query un2 _system tasks \
  --filter-json '{"op":"eq","field":"status","value":"pending"}' \
  --sort -priority \
  --sort +createdAt \
  --limit 10
```

**Exemple avec `--latest` :**

```bash
jsondb_cli db query un2 _system logs --latest --limit 20
```

---

## 2. Gestion des Collections (`collection`)

| Action | Description | Arguments | Exemple |
|--------|-------------|-----------|---------|
| **`create`** | Crée une collection et lie un schéma. | `<space> <db> <name> --schema <path>` | `jsondb_cli collection create un2 _system articles --schema articles/article.schema.json` |

**Exemple complet :**

```bash
jsondb_cli collection create un2 _system users \
  --schema actors/actor.schema.json
```

---

## 3. Gestion des Documents (`document`)

Opérations unitaires. Le chemin du schéma est relatif à `schemas/v1`.

| Action | Description | Arguments | Exemple |
|--------|-------------|-----------|---------|
| **`insert`** | Insère un document (valide + x_compute). | `<space> <db> --schema <s> --file <f>` | `jsondb_cli document insert un2 _system --schema actors/actor.schema.json --file doc.json` |
| **`upsert`** | Insère ou met à jour si l'ID existe. | `<space> <db> --schema <s> --file <f>` | `jsondb_cli document upsert un2 _system --schema actors/actor.schema.json --file doc.json` |

**Exemple d'utilisation :**

```bash
# Créer un fichier document
cat > new_user.json << EOF
{
  "name": "Alice Dupont",
  "email": "alice@example.com",
  "role": "admin"
}
EOF

# Insérer le document
jsondb_cli document insert un2 _system \
  --schema actors/actor.schema.json \
  --file new_user.json
```

---

## 4. Transactions ACID (`transaction`)

Exécute un lot d'opérations de manière atomique via le **Transaction Manager** et le **WAL**.

| Action | Description | Arguments | Exemple |
|--------|-------------|-----------|---------|
| **`execute`** | Exécute une transaction définie dans un fichier JSON. | `<space> <db> <file>` | `jsondb_cli transaction execute un2 _system ./tx_batch.json` |

### Format du fichier de transaction

Le fichier JSON doit contenir un tableau `operations`. La CLI supporte une opération spéciale **`insertFrom`** pour charger le contenu depuis un fichier externe.

**Variables supportées dans les chemins :** `$HOME`, `$PATH_GENAPTITUDE_DATASET`.

```json
{
  "operations": [
    {
      "type": "insert",
      "collection": "users",
      "doc": { 
        "id": "u1", 
        "name": "Alice", 
        "role": "admin" 
      }
    },
    {
      "type": "insertFrom",
      "collection": "articles",
      "path": "$PATH_GENAPTITUDE_DATASET/articles/intro.json"
    },
    {
      "type": "update",
      "collection": "users",
      "doc": { 
        "id": "u2", 
        "role": "editor" 
      }
    },
    {
      "type": "delete",
      "collection": "logs",
      "id": "log-old-123"
    }
  ]
}
```

### Types d'opérations supportées

| Type | Description | Champs requis |
|------|-------------|---------------|
| **`insert`** | Insère un nouveau document | `collection`, `doc` |
| **`insertFrom`** | Insère un document depuis un fichier | `collection`, `path` |
| **`update`** | Met à jour un document existant | `collection`, `doc` (avec `id`) |
| **`delete`** | Supprime un document | `collection`, `id` |

### Exemple complet

```bash
# Créer le fichier de transaction
cat > batch_operations.json << EOF
{
  "operations": [
    {
      "type": "insert",
      "collection": "logs",
      "doc": {
        "message": "Transaction started",
        "level": "info"
      }
    },
    {
      "type": "insertFrom",
      "collection": "projects",
      "path": "$PATH_GENAPTITUDE_DATASET/projects/project_alpha.json"
    },
    {
      "type": "update",
      "collection": "users",
      "doc": {
        "id": "urn:uuid:user-123",
        "status": "active",
        "lastLogin": "2025-11-27T10:00:00Z"
      }
    }
  ]
}
EOF

# Exécuter la transaction
jsondb_cli transaction execute un2 _system batch_operations.json
```

---

## 5. Moteur de Requêtes Avancé (`query`)

Pour les requêtes complexes définies dans un fichier séparé.

| Action | Description | Arguments | Exemple |
|--------|-------------|-----------|---------|
| **`find-many`** | Exécute une requête définie dans un fichier JSON. | `<space> <db> <file>` | `jsondb_cli query find-many un2 _system ./queries/complex_search.json` |

### Format du fichier de requête

```json
{
  "collection": "articles",
  "filter": {
    "operator": "and",
    "conditions": [
      { 
        "field": "tags", 
        "operator": "contains", 
        "value": "rust" 
      },
      { 
        "field": "status", 
        "operator": "eq", 
        "value": "published" 
      }
    ]
  },
  "sort": [
    { 
      "field": "createdAt", 
      "order": "desc" 
    }
  ],
  "limit": 10,
  "offset": 0,
  "projection": {
    "Include": ["id", "title", "slug"]
  }
}
```

### Structure complète de Query

```json
{
  "collection": "string",           // Nom de la collection
  "filter": {                        // Optionnel
    "operator": "and" | "or",
    "conditions": [
      {
        "field": "string",
        "operator": "eq" | "ne" | "gt" | "gte" | "lt" | "lte" | "contains",
        "value": any
      }
    ]
  },
  "sort": [                          // Optionnel
    {
      "field": "string",
      "order": "asc" | "desc"
    }
  ],
  "limit": number,                   // Optionnel
  "offset": number,                  // Optionnel
  "projection": {                    // Optionnel
    "Include": ["field1", "field2"]  // ou "Exclude": [...]
  }
}
```

### Exemple d'utilisation

```bash
# Créer une requête complexe
cat > search_articles.json << EOF
{
  "collection": "articles",
  "filter": {
    "operator": "and",
    "conditions": [
      { "field": "status", "operator": "eq", "value": "published" },
      { "field": "views", "operator": "gte", "value": 1000 },
      { "field": "tags", "operator": "contains", "value": "technology" }
    ]
  },
  "sort": [
    { "field": "views", "order": "desc" },
    { "field": "createdAt", "order": "desc" }
  ],
  "limit": 20,
  "projection": {
    "Include": ["id", "title", "author", "views", "createdAt"]
  }
}
EOF

# Exécuter la requête
jsondb_cli query find-many un2 _system search_articles.json
```

---

## 6. Utilitaires de Dataset (`dataset`)

Opérations de masse pour l'initialisation (seeding).

| Action | Description | Arguments | Exemple |
|--------|-------------|-----------|---------|
| **`seed-dir`** | Insère tous les `.json` d'un dossier. Le nom du dossier détermine la collection cible. | `<space> <db> <dir_path>` | `jsondb_cli dataset seed-dir un2 _system ./data/actors` |

### Fonctionnement du seed-dir

- Parcourt récursivement le répertoire spécifié
- Pour chaque fichier `.json` trouvé, insère le document dans la collection
- Le nom du dossier parent détermine la collection cible
- Utilise le schéma associé à la collection pour validation

**Exemple de structure de données :**

```
data/
├── actors/
│   ├── actor_001.json
│   ├── actor_002.json
│   └── actor_003.json
├── projects/
│   ├── project_alpha.json
│   └── project_beta.json
└── tasks/
    └── task_001.json
```

**Commandes d'import :**

```bash
# Importer tous les acteurs
jsondb_cli dataset seed-dir un2 _system ./data/actors

# Importer tous les projets
jsondb_cli dataset seed-dir un2 _system ./data/projects

# Importer toutes les tâches
jsondb_cli dataset seed-dir un2 _system ./data/tasks
```

---

## 7. Commandes SQL (`sql`)

> ⚠️ **Statut** : Expérimental / Placeholder

| Action | Description | Arguments |
|--------|-------------|-----------|
| **`exec`** | Exécute une commande SQL (non implémenté). | `<space> <db> <query>` |

Cette fonctionnalité est prévue pour une future version et permettra d'interroger la base avec une syntaxe SQL.

---

## 📊 Récapitulatif des Commandes

| Module | Action | Usage Principal |
|--------|--------|-----------------|
| **db** | `create` | Initialiser une nouvelle base |
| | `open` | Vérifier l'intégrité |
| | `drop` | Supprimer une base |
| | `query` | Requête ad-hoc rapide |
| **collection** | `create` | Créer une collection avec schéma |
| **document** | `insert` | Insérer un document unique |
| | `upsert` | Insérer ou mettre à jour |
| **transaction** | `execute` | Opérations atomiques multiples |
| **query** | `find-many` | Recherche avancée avec fichier |
| **dataset** | `seed-dir` | Import en masse depuis dossier |
| **sql** | `exec` | *(Futur)* Requêtes SQL |

---

## 🔒 Bonnes Pratiques

### 1. Validation et Schémas

Toujours utiliser le flag `--schema` pour garantir l'intégrité des données lors des insertions.

```bash
# ✅ Bon
jsondb_cli document insert un2 _system \
  --schema actors/actor.schema.json \
  --file user.json

# ❌ Éviter (pas de validation)
# Utiliser les commandes avec schéma
```

### 2. Transactions pour les Opérations Critiques

Pour les opérations multi-documents ou critiques, utilisez toujours les transactions ACID.

```bash
# ✅ Atomique et sûr
jsondb_cli transaction execute un2 _system batch_ops.json

# ❌ Éviter pour les opérations liées
# jsondb_cli document insert ... (plusieurs fois)
```

### 3. Variables d'Environnement

Utilisez les variables d'environnement pour les chemins dynamiques dans les transactions.

```json
{
  "type": "insertFrom",
  "collection": "data",
  "path": "$PATH_GENAPTITUDE_DATASET/exports/data.json"
}
```

### 4. Requêtes Ad-Hoc vs Fichiers

- **Ad-hoc** (`db query`) : Pour les tests rapides et l'exploration
- **Fichiers** (`query find-many`) : Pour les requêtes réutilisables et complexes

### 5. Logging

Activez les logs pour le debugging :

```bash
export RUST_LOG=debug
jsondb_cli db query un2 _system articles --latest
```

---

## 🐛 Dépannage

### Problème : "Database not found"

**Solution :** Vérifiez que `PATH_GENAPTITUDE_DOMAIN` est correctement défini et que la base existe.

```bash
echo $PATH_GENAPTITUDE_DOMAIN
jsondb_cli db create un2 _system
```

### Problème : "Schema not found"

**Solution :** Utilisez `--repo-root` pour pointer vers le bon répertoire de schémas.

```bash
jsondb_cli --repo-root /path/to/repo document insert ...
```

### Problème : "Transaction failed"

**Solution :** Vérifiez les logs et le fichier WAL. Les transactions garantissent l'atomicité - si une opération échoue, tout est annulé.

```bash
export RUST_LOG=debug
jsondb_cli transaction execute un2 _system tx.json
```

---

## 📝 Métadonnées

**Version** : 1.0  
**Dernière mise à jour** : Novembre 2025  
**Statut** : Production  
**Dépendances** : Rust, TransactionManager, QueryEngine, StorageEngine
