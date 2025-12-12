# 🦀 GenAptitude - Backend Rust (Tauri Core)

Le cœur de GenAptitude est une application **Rust** haute performance utilisant le framework **Tauri v2**.
Il agit comme un serveur local souverain gérant la logique métier lourde, le stockage des données, l'intelligence artificielle et l'orchestration des processus.

## 🏗 Architecture Modulaire

Le backend est structuré en modules découplés (Domain Driven Design) :

```text
src-tauri/src/
├── ai/                 # 🤖 Cerveau Neuro-Symbolique (Agents, RAG, LLM Client)
├── blockchain/         # 🔗 Infrastructure de Confiance (Fabric, Innernet VPN)
├── code_generator/     # ⚡ Usine Logicielle Hybride (Templates Tera + Injection IA)
├── commands/           # 🔌 Interface API (IPC Tauri) exposée au Frontend
├── genetics/           # 🧬 Moteur d'Optimisation (Algorithmes Évolutionnaires)
├── json_db/            # 🗄️ SGBD NoSQL/Sémantique (Stockage, Index, SQL, ACID)
├── model_engine/       # 📚 Moteur Sémantique (Loader Arcadia/Capella)
├── plugins/            # 🧠 Hôte WASM (Exécution de règles dynamiques)
├── traceability/       # 🛡️ Gouvernance (Audit, Conformité DO-178C/AI Act)
├── utils/              # 🛠️ Fondations (Config, Logs, Erreurs)
├── workflow_engine/    # 🔀 Orchestrateur Symbolique (Graphes de tâches, HITL)
├── lib.rs              # Point d'entrée Librairie
└── main.rs             # Point d'entrée Exécutable
```

---

## 🧩 Détail des Modules

### 1\. 🗄️ JSON-DB (Persistance)

Un moteur de base de données transactionnel conçu pour l'ingénierie système.

- **Sémantique** : Validation native **JSON-LD** et conformité aux schémas Arcadia.
- **ACID** : Transactions atomiques avec journalisation (WAL).
- **SQL** : Moteur de requête supportant les projections et filtres complexes.

### 2\. 🤖 AI Kernel (Intelligence)

Le cerveau "neuronal" du système.

- **Dual Mode** : Route les requêtes vers le Local (Docker/Mistral) ou le Cloud (Gemini) selon la complexité.
- **Agents** : `SystemAgent` pour la modélisation, `SoftwareAgent` pour le code.
- **RAG** : Récupération de contexte vectoriel pour ancrer les réponses.

### 3\. 🔀 Workflow Engine (Orchestration)

Le cerveau "symbolique" du système.

- **Déterministe** : Exécute des graphes de tâches définis statiquement.
- **HITL (Human-in-the-loop)** : Gestion native des pauses pour validation humaine.
- **State Machine** : Suivi rigoureux de l'état d'avancement.

### 4\. 🛡️ Traceability (Assurance)

Garantit que le modèle est conforme aux normes critiques.

- **Impact Analysis** : Calcule la propagation d'un changement dans le graphe.
- **Compliance** : Checkers automatiques pour **DO-178C**, **ISO-26262** et **EU AI Act**.

### 5\. 📚 Model Engine

Chargeur haute performance pour les modèles Arcadia.

- Hydrate les données JSON brutes en structures Rust fortement typées (`ProjectModel`).
- Gère les 5 couches d'abstraction : OA, SA, LA, PA, EPBS.

---

## 🛠 Administration & Outils (CLI)

Le projet inclut plusieurs binaires CLI pour l'administration et le débogage sans l'UI.

| Outil            | Commande                     | Usage                                                     |
| :--------------- | :--------------------------- | :-------------------------------------------------------- |
| **JsonDB Admin** | `cargo run -p jsondb_cli`    | Création de bases, requêtes SQL, réparation d'index.      |
| **AI Debugger**  | `cargo run -p ai_cli`        | Test du chat, classification d'intention, ping LLM.       |
| **Validator**    | `cargo run -p validator_cli` | Vérification stricte d'un fichier JSON contre son schéma. |

### Exemples

```bash
# Requête SQL sur la base locale
cargo run -p jsondb_cli -- --space un2 --db _system sql \
  --query "SELECT name, kind FROM actors WHERE kind = 'human'"

# Test de compréhension IA
cargo run -p ai_cli -- classify "Crée une fonction Démarrer"
```

## ✅ Tests et Qualité

La qualité est assurée par une batterie de tests unitaires et d'intégration.

```bash
# 1. Tester les fondations (Utils)
cargo test utils::

# 2. Tester la base de données (Intégration)
cargo test --test json_db_suite

# 3. Tester le moteur de workflow
cargo test workflow_engine::

# 4. Tester tout le projet (Attention : peut être long)
cargo test
```

### Vérification du code

```bash
# Compilation rapide
cargo check

# Linter strict
cargo clippy -- -D warnings
```

```

```

```

```
