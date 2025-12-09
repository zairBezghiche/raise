### 2\. Fichier : `src/services/json-db/README.md`

Ce fichier explique aux développeurs Frontend comment utiliser les services pour interagir avec la base de données.

**Action :** Créez le fichier **`src/services/json-db/README.md`** (ou remplacez son contenu) avec ceci :

````markdown
# Services Frontend - JSON-DB

Cette couche fait le pont entre l'UI (React) et le Backend (Rust/Tauri).

## 📂 Architecture

- **`collection-service.ts`** : Point d'entrée principal. Gère le cycle de vie de la DB (Create/Drop), des Collections, des Index et le CRUD.
- **`query-service.ts`** : Constructeur de requêtes (QueryBuilder) et exécution SQL.
- **`transaction-service.ts`** : Gestion des opérations atomiques par lots.
- **`jsonld-service.ts`** : Utilitaires pour le format JSON-LD (Web Sémantique).

## 🚀 Utilisation (Exemples)

### Initialisation et Administration

```typescript
import { collectionService } from '@/services/json-db/collection-service';

// Créer la structure physique sur le disque
await collectionService.createDb();

// Créer un index pour accélérer les recherches
await collectionService.createIndex('actors', 'name', 'hash');
```
````

### CRUD & Recherche

```typescript
// Insérer un document
await collectionService.insertDocument('actors', {
  name: 'Robot',
  description: 'Unité autonome',
});

// Rechercher via QueryBuilder
import { createQuery } from '@/services/json-db/query-service';

const query = createQuery('actors').where('name', 'Contains', 'Robot').limit(10).build();

const results = await collectionService.queryDocuments('actors', query);
```
