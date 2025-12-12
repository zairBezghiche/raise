# Module de Commandes (Tauri API Layer)

Ce répertoire contient l'ensemble des **Commandes Tauri** qui servent d'interface (API) entre le frontend (React/TypeScript) et le moteur backend (Rust).

Chaque fichier ici expose des fonctions annotées avec `#[tauri::command]`, qui sont enregistrées dans le `main.rs` et appelables depuis l'UI via `invoke()`.

## 📂 Organisation des Modules

| Fichier                        | Domaine                | Description                                                                                                     |
| :----------------------------- | :--------------------- | :-------------------------------------------------------------------------------------------------------------- |
| **`ai_commands.rs`**           | 🧠 IA Générative       | Gestion du chat avec les LLM (Local/Cloud), classification d'intention et RAG (Retrieval Augmented Generation). |
| **`blockchain_commands.rs`**   | 🔗 Blockchain & Réseau | Interactions avec Hyperledger Fabric (transactions) et le VPN Innernet (Mesh networking).                       |
| **`codegen_commands.rs`**      | ⚡ Génération de Code  | Transformation des modèles en code source (Rust, Python) via des templates.                                     |
| **`cognitive_commands.rs`**    | 🤖 Analyse Cognitive   | Exécution de modules WASM (WebAssembly) pour l'analyse structurelle ou sémantique.                              |
| **`genetics_commands.rs`**     | 🧬 Optimisation        | Algorithmes génétiques pour l'optimisation des architectures (simulation de générations).                       |
| **`json_db_commands.rs`**      | 💾 Base de Données     | CRUD complet sur le moteur NoSQL local (Spaces, DBs, Collections, Documents, Index, SQL).                       |
| **`model_commands.rs`**        | 🏗️ Gestion du Modèle   | Chargement et maintien en mémoire du `ProjectModel` (Arcadia) pour les opérations lourdes.                      |
| **`traceability_commands.rs`** | 🔍 Traçabilité & Audit | Moteur d'analyse d'impact, matrices de couverture et vérification de conformité (EU AI Act, DO-178C).           |
| **`utils_commands.rs`**        | 🛠️ Utilitaires         | Informations système, état de santé de l'API et configuration globale.                                          |
| **`workflow_commands.rs`**     | 🔀 Workflow Engine     | Orchestrateur de tâches, machine à états et gestion des validations humaines (HITL).                            |

---

## 🛠 Détails des APIs

### 1. Intelligence Artificielle (`ai_commands.rs`)

Gère l'assistant contextuel.

- `ai_chat(user_input)`: Pipeline complet (Classification -> Recherche Contexte -> Prompting -> LLM). Supporte le mode Dual (Gemini/Local).

### 2. Blockchain & VPN (`blockchain_commands.rs`)

Interface pour la sécurité et la traçabilité distribuée.

- `fabric_submit_transaction(...)`: Soumission de transactions au ledger.
- `vpn_network_status()`: État de la connexion mesh (pairs connectés, IP).

### 3. Base de Données (`json_db_commands.rs`)

Interface directe avec le moteur de stockage JSON.

- **Structure** : Space ➝ DB ➝ Collection ➝ Document.
- **Commandes** : `jsondb_create_db`, `jsondb_insert_document`, `jsondb_execute_query` (recherche complexe), `jsondb_execute_sql`.

### 4. Modèle & Architecture (`model_commands.rs`)

- `load_project_model(space, db)`: Charge l'intégralité du projet depuis la DB vers la RAM (Mutex global) pour permettre les analyses rapides. S'exécute dans un thread bloquant pour ne pas figer l'UI.

### 5. Traçabilité & Conformité (`traceability_commands.rs`)

Nouvelles commandes pour l'assurance qualité.

- `analyze_impact(element_id, depth)`: Calcule la propagation d'un changement dans le graphe.
- `run_compliance_audit()`: Lance les checkers (DO-178C, ISO-26262, EU AI Act) et retourne un rapport JSON.

### 6. Modules Avancés

- **Génétique** (`genetics_commands.rs`): `run_genetic_optimization` prend des paramètres de mutation/génération et simule une convergence.
- **Cognitif** (`cognitive_commands.rs`): `run_consistency_analysis` charge dynamiquement un binaire `.wasm` selon l'environnement (Dev/Prod) pour analyser le modèle.
- **CodeGen** (`codegen_commands.rs`): `generate_source_code` produit du code textuel basé sur les métadonnées du modèle.

### 7. Utilitaires (`utils_commands.rs`)

Commandes systèmes légères.

- `get_app_info()`: Renvoie la version, le mode (Dev/Prod), le chemin de la base de données et l'état de la connexion API.

### 8. Workflow Engine (`workflow_commands.rs`)

Pilotage du moteur d'orchestration symbolique.

- `register_workflow(definition)`: Enregistre un nouveau graphe de tâches.
- `start_workflow(workflow_id)`: Instancie et démarre l'exécution.
- `resume_workflow(instance_id, node_id, approved)`: Débloque une étape en pause (Validation Humaine).
- `get_workflow_state(instance_id)`: Récupère l'avancement et les logs.

---

## 💻 Exemple d'appel (Frontend)

Voici comment appeler ces commandes depuis React/TypeScript :

```typescript
import { invoke } from '@tauri-apps/api/core';

// Exemple 1 : Vérifier la connexion (Utils)
async function checkSystem() {
  const info = await invoke('get_app_info');
  console.log('Système :', info);
}

// Exemple 2 : Lancer un audit de conformité (Traceability)
async function runAudit() {
  try {
    const report = await invoke('run_compliance_audit');
    console.log('Rapport de conformité :', report);
  } catch (error) {
    console.error("Erreur d'audit :", error);
  }
}

// Exemple 3 : Démarrer un Workflow (Engine)
async function runDeploymentPipeline() {
  const view = await invoke('start_workflow', { workflowId: 'deploy-prod-v1' });
  console.log('Workflow démarré, statut :', view.status);
}
```

## ⚠️ Notes Techniques

- **État Partagé (`AppState`)** : Les commandes `model_commands` et `traceability_commands` partagent le même `Mutex<ProjectModel>`.
- **Workflow Store** : Le moteur de workflow utilise un `tokio::sync::Mutex` (asynchrone) pour permettre l'exécution concurrente des tâches sans bloquer l'interface.
- **Async/Sync** : Les opérations lourdes (IA, Chargement Modèle, Génétique) sont `async` et déléguées à des threads dédiés.

<!-- end list -->
