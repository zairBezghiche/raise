# ⚛️ GenAptitude - Architecture Frontend

**Package :** `src/`  
**Stack :** React 18, TypeScript, Vite, Zustand, Tauri IPC (v2)  
**Version :** 2.1 (Full Modular Architecture)

---

## 1\. Vue d'Ensemble

Le frontend de GenAptitude est une **Single Page Application (SPA)** agissant comme une interface native (GUI) pour le noyau Rust. Elle est conçue comme un **Cockpit d'Ingénierie** modulaire, orchestrant des moteurs hétérogènes (IA, WASM, Génétique, Blockchain).

### Responsabilités Clés

- **Visualisation** : Exploration des modèles Arcadia (OA, SA, LA, PA, EPBS).
- **Intelligence** : Chatbot, Diagnostic LLM/RAG, et Orchestration d'Agents.
- **Calcul** : Pilotage des simulations génétiques et des plugins WASM.
- **Infrastructure** : Monitoring Blockchain et Génération de code.

---

## 2\. Structure Exhaustive des Dossiers

L'application suit une structure hybride séparant les composants UI génériques des fonctionnalités métier.

```text
src/
├── assets/             # Ressources statiques (icônes SVG, images, polices)
│
├── components/         # Composants UI réutilisables (Atoms/Molecules)
│   ├── ui/             # Boutons, Inputs, Cards, Modales génériques
│   └── layout/         # Sidebar, Header, AppShell
│
├── features/           # Modules Métier Autonomes (Domain Logic)
│   ├── ai-chat/        # Logique du Chat et du Dashboard IA
│   ├── codegen/        # Logique de l'éditeur de code
│   ├── genetics/       # Logique du dashboard d'optimisation
│   ├── model-viewer/   # Logique d'affichage Arcadia (Dictionnaire)
│   └── blockchain/     # Logique de visualisation réseau
│
├── hooks/              # Custom Hooks React transverses
│   ├── useTauriEvent.ts  # Écoute d'événements Backend
│   ├── useKeyboard.ts    # Raccourcis clavier (ex: 'B' pour Blockchain)
│   └── useDebounce.ts    # Optimisation des inputs
│
├── pages/              # Vues principales (Conteneurs de haut niveau)
│   ├── AssistantPage.tsx
│   ├── CognitivePage.tsx
│   ├── DictionaryPage.tsx
│   ├── GeneticsPage.tsx
│   └── SettingsPage.tsx
│
├── services/           # Couche d'abstraction API (Tauri IPC)
│   ├── ai-service.ts        # Pont vers module AI
│   ├── model-service.ts     # Pont vers module ModelEngine
│   ├── cognitive-service.ts # Pont vers module Plugins
│   ├── genetics-service.ts  # Pont vers module Genetics
│   └── codegen-service.ts   # Pont vers module CodeGen
│
├── store/              # Gestion d'état global (Zustand)
│   ├── model-store.ts  # Le Modèle Projet chargé
│   ├── ai-store.ts     # État de la conversation
│   └── ui-store.ts     # Préférences d'affichage
│
├── types/              # Définitions TypeScript partagées
│   ├── arcadia.types.ts
│   ├── ai.types.ts
│   └── ipc.types.ts
│
├── utils/              # Fonctions utilitaires pures
│   ├── formatters.ts   # Formatage dates/nombres
│   ├── mappers.ts      # Transformation JSON Rust -> UI
│   └── validators.ts   # Validation formulaires
│
├── styles/             # Styles globaux
│   ├── variables.css   # Couleurs, Espacements, Fontes
│   └── globals.css     # Reset CSS et styles de base
│
├── App.tsx             # Router manuel et Layout Principal
└── main.tsx            # Point d'entrée React/Vite
```

---

## 3\. Architecture Modulaire (Vues & Pages)

L'application est structurée autour d'un **Router Manuel** (dans `App.tsx`) qui orchestre 8 vues distinctes stockées dans `src/pages/` ou `src/features/`.

| ID Vue       | Module / Page        | Description                                              | Service Associé         |
| :----------- | :------------------- | :------------------------------------------------------- | :---------------------- |
| `assistant`  | **Assistant IA**     | Interface conversationnelle (Chat) pour l'utilisateur.   | `ai-service.ts`         |
| `ai-studio`  | **AI Studio**        | Console technique : État LLM, NLP, RAG & Agents.         | `ai-service.ts`         |
| `dictionary` | **Modèle & Data**    | Explorateur hiérarchique Arcadia (OA, SA, LA, PA, EPBS). | `model-service.ts`      |
| `cognitive`  | **Blocs Cognitifs**  | Testeur de plugins `.wasm` (Analysers).                  | `cognitive-service.ts`  |
| `codegen`    | **Usine Logicielle** | IDE simplifié pour génération Rust/Python.               | `codegen-service.ts`    |
| `genetics`   | **Génétique**        | Dashboard de simulation d'optimisation.                  | `genetics-service.ts`   |
| `blockchain` | **Blockchain**       | Visualisation VPN et Hyperledger.                        | N/A (Demo)              |
| `admin-db`   | **Base de Données**  | CRUD bas niveau sur la JsonDB.                           | `collection-service.ts` |

---

## 4\. Gestion d'État (Stores)

La gestion d'état est assurée par **Zustand**.

### `useModelStore`

Le store critique. Il contient l'arbre complet du projet chargé.

- **State** : `project: ProjectModel | null`
- **Rôle** : Centralise les données Arcadia pour les redistribuer aux modules (CodeGen pour générer, Cognitive pour analyser, Dictionary pour afficher).

### `useAiStore`

Gère le contexte conversationnel.

- **State** : `messages: Message[]`, `isLoading: boolean`

---

## 5\. Services & Intégration (IPC Layer)

Les services (`src/services/`) convertissent les appels TypeScript en commandes Rust via `invoke`.

- **`ModelService`** : Charge un projet complet (commande `load_project_model`).
- **`AiService`** : Expose les diagnostics internes (`ai_get_system_status`, `ai_test_nlp`).
- **`CognitiveService`** : Envoie le modèle JSON à Wasmtime (`run_consistency_analysis`).
- **`CodegenService`** : Déclenche le moteur de templates Tera (`generate_source_code`).
- **`GeneticsService`** : Lance les threads de calcul CPU (`run_genetic_optimization`).

---

## 6\. Flux de Données Type (Ex: Analyse WASM)

Exemple d'utilisation traversant toute l'architecture :

1.  **Page** : L'utilisateur clique sur "Analyser" dans `CognitivePage`.
2.  **Store** : La page récupère le `project` depuis `useModelStore`.
3.  **Utils** : `mappers.ts` transforme l'objet Arcadia complexe en `CognitiveModel` plat.
4.  **Service** : `cognitiveService` appelle `invoke("run_consistency_analysis")`.
5.  **Backend** : Rust charge le `.wasm`, exécute l'analyse, renvoie le JSON.
6.  **UI** : La page affiche le rapport d'erreurs retourné.

---

## 7\. Guide d'Extension

Pour ajouter un nouveau module :

1.  **Backend** : Créer la commande Rust dans `src-tauri`.
2.  **Types** : Ajouter les interfaces dans `src/types/`.
3.  **Service** : Créer `src/services/mon-service.ts`.
4.  **Feature** : Créer la logique et les composants dans `src/features/mon-module/`.
5.  **Page** : Créer `src/pages/MaPage.tsx` qui assemble la feature.
6.  **Wiring** : Ajouter l'entrée dans le `renderContent` de `App.tsx` et le bouton dans la Sidebar.
