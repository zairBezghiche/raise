#!/bin/bash

# Se placer automatiquement à la racine du projet (le dossier parent de "scripts/")
cd "$(dirname "$0")/.." || exit
echo "📂 Dossier de travail : $(pwd)"

# 1. Mise à jour de json_db.md avec la section Cache & StorageEngine
echo "📝 Mise à jour de src-tauri/src/json_db/json_db.md..."
mkdir -p src-tauri/src/json_db
cat <<'EOF' > src-tauri/src/json_db/json_db.md
# 📦 Module `json_db`

## Vue d'Ensemble

Le module **`json_db`** est un moteur de base de données NoSQL orienté documents, conçu spécifiquement pour l'architecture locale de GenAptitude. Il combine la simplicité du stockage fichier JSON avec des garanties transactionnelles fortes (ACID) et des performances de lecture optimisées par un système de cache intelligent.

### Caractéristiques Principales

- **Stockage Souverain** : Données stockées sous forme de fichiers JSON lisibles.
- **Transactions ACID** : Garantie d'intégrité via WAL (`_wal.jsonl`) et commit atomique.
- **Moteur de Stockage (StorageEngine)** : Couche d'abstraction gérant la configuration et le cache en mémoire.
- **Cache Thread-Safe** : Mise en cache des index et manifestes avec gestion de TTL (Time To Live) et capacité maximale.
- **Indexation Hybride** : Hash, B-Tree et Text (Full-Text) pour des recherches rapides.
- **Moteur `x_compute`** : Calcul automatique de champs (UUID, dates) avant validation.

---

## 🏗️ Architecture

### Arborescence Physique

```text
<domain_root>/
  ├── <space>/                  # Espace de travail (ex: "un2")
  │   ├── <database>/           # Base de données (ex: "_system")
  │   │   ├── _system.json      # Manifeste (Mis en cache par StorageEngine)
  │   │   ├── _wal.jsonl        # Journal des transactions
  │   │   ├── collections/
  │   │   │   └── <collection>/
  │   │   │       ├── _config.json # Config index
  │   │   │       ├── _indexes/    # Index binaires
  │   │   │       └── <uuid>.json  # Documents