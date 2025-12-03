# Commandes Tauri : model_commands

> **Version API :** 1.1
> **Module :** `model_engine` > **Statut :** Implémenté (Chargement Sémantique)

Ce module expose les fonctionnalités de **Model-Based Systems Engineering (MBSE)** au frontend. Il fait le pont entre le stockage brut (`json_db`) et la logique métier structurée (`Arcadia`).

Contrairement aux commandes `json_db` qui manipulent des documents génériques, ces commandes retournent des objets **fortement typés** et **validés sémantiquement**.

---

## 🔌 Vue d'Ensemble

Les commandes de ce module sont conçues pour :

1.  **Hydrater** le modèle en mémoire depuis le disque.
2.  **Convertir** les documents JSON-LD en structures Rust/TS utilisables.
3.  **Gérer la charge** via des threads dédiés pour ne pas bloquer l'UI.

---

## 1. Chargement du Modèle

### `load_project_model`

Charge l'intégralité du projet (toutes les collections) en mémoire, résout les types sémantiques (JSON-LD) et organise les éléments par couches d'ingénierie (OA, SA, LA, PA, EPBS).

⚠️ **Performance** : Cette opération est coûteuse (I/O + Parsing). Elle est exécutée dans un thread bloquant (`spawn_blocking`) côté Rust, mais reste asynchrone (Promise) côté Frontend.

**Signature Rust :**

```rust
#[tauri::command]
pub async fn load_project_model(
    storage: State<'_, StorageEngine>,
    space: String,
    db: String,
) -> Result<ProjectModel, String>
```

````

**Usage TypeScript :**

```typescript
import { invoke } from '@tauri-apps/api/core';
import type { ProjectModel } from '@/types/model.types';

async function load() {
  try {
    const model = await invoke<ProjectModel>('load_project_model', {
      space: 'un2',
      db: '_system',
    });

    console.log('Acteurs OA chargés:', model.oa.actors.length);
    console.log('Fonctions SA chargées:', model.sa.functions.length);
  } catch (e) {
    console.error('Erreur de chargement', e);
  }
}
```

---

## 2\. Structures de Données Retournées

Le frontend reçoit un objet `ProjectModel` structuré. Voici sa forme JSON typique :

```json
{
  "oa": {
    "actors": [ ... ],
    "activities": [ ... ],
    "capabilities": [ ... ]
  },
  "sa": {
    "components": [ ... ],
    "functions": [ ... ],
    "exchanges": [ ... ]
  },
  "la": { "components": [], ... },
  "pa": { "components": [], ... },
  "epbs": { "configurationItems": [] },
  "meta": {
    "elementCount": 150
  }
}
```

### L'objet `ArcadiaElement`

Chaque élément dans les listes ci-dessus respecte cette structure :

```typescript
interface ArcadiaElement {
  id: string; // UUID v4
  name: string; // Nom ou Libellé
  type: string; // URI Sémantique complète (ex: "https://...#SystemFunction")

  // Propriétés dynamiques (Map)
  [key: string]: any; // ex: "allocatedTo", "criticality", "inputs"...
}
```

---

## 3\. Fonctionnement Interne

### Pipeline de Chargement

1.  **Clonage du Moteur** : Le `StorageEngine` est cloné (opération légère via `Arc`) pour être passé au thread de travail.
2.  **Thread Dédié** : `tauri::async_runtime::spawn_blocking` est utilisé pour sortir de la boucle événementielle de Tauri.
3.  **ModelLoader** :
    - Instancie un `ModelLoader` découplé.
    - Scanne toutes les collections de la DB.
    - Utilise `JsonLdProcessor` pour expandre les types (ex: `"sa:SystemFunction"` devient l'URI canonique).
    - Dispatche chaque élément dans le bon vecteur (`model.sa.functions`, etc.) selon son type exact défini dans `vocabulary.rs`.
4.  **Retour** : Le `ProjectModel` final est sérialisé en JSON et renvoyé au frontend.

### Sémantique JSON-LD

Le backend ne se base pas sur le nom de la collection pour typer les objets, mais sur leur champ `@type` (ou `type`).

- Si le JSON contient `@type: "oa:OperationalActor"`, il sera rangé dans `model.oa.actors`.
- Si le JSON contient `@type: "sa:SystemComponent"`, il sera rangé dans `model.sa.components`.

Cela garantit que le modèle en mémoire est toujours cohérent avec l'ontologie Arcadia, quelle que soit la manière dont les fichiers sont stockés physiquement.

```

```
````
