#!/usr/bin/env bash
# scripts/dataset_migrate_arcadia_v1.sh
# Objectif: migrer un dataset ARCADAIA "plat" (actors/, activities/, ...) vers la convention:
#   $PATH_GENAPTITUDE_DATASET/arcadia/<version>/data/oa/{actors,activities,exchanges,capabilities,entities}
#   $PATH_GENAPTITUDE_DATASET/arcadia/<version>/data/{diagram,processes}
# et générer un manifest.json.
#
# Usage:
#   scripts/dataset_migrate_arcadia_v1.sh --source=/path/arcadia_src --version=v1 [--mode=copy|move] [--datasets-root=PATH]
#
# Exemples:
#   export PATH_GENAPTITUDE_DATASET="$HOME/genaptitude_dataset"
#   bash scripts/dataset_migrate_arcadia_v1.sh --source="$HOME/genaptitude_dataset/arcadia" --version=v1 --mode=copy
#
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  scripts/dataset_migrate_arcadia_v1.sh --source=PATH --version=v1 [--mode=copy|move] [--datasets-root=PATH]

Options:
  --source=PATH         Chemin du dataset ARCADAIA actuel (qui contient actors/, activities/, ...)
  --version=STR         Version cible (ex: v1, v2025-11-13)
  --mode=copy|move      Copier (par défaut) ou déplacer les fichiers
  --datasets-root=PATH  Racine des datasets (défaut: $PATH_GENAPTITUDE_DATASET)

Remarques:
  - Requiert 'jq' pour générer un manifest propre (sinon on écrit un JSON minimal sans jq).
USAGE
}

SOURCE=""
VERSION=""
MODE="copy"
DATASETS_ROOT="${PATH_GENAPTITUDE_DATASET:-}"

for arg in "$@"; do
  case "$arg" in
    --source=*)
      SOURCE="${arg#*=}"
      ;;
    --version=*)
      VERSION="${arg#*=}"
      ;;
    --mode=*)
      MODE="${arg#*=}"
      ;;
    --datasets-root=*)
      DATASETS_ROOT="${arg#*=}"
      ;;
    -h|--help)
      usage; exit 0
      ;;
    *)
      echo "Unknown option: $arg" >&2; usage; exit 1
      ;;
  esac
done

if [[ -z "$SOURCE" || -z "$VERSION" ]]; then
  usage; exit 1
fi
if [[ -z "$DATASETS_ROOT" ]]; then
  echo "❌ DATASETS_ROOT non défini. Soit export PATH_GENAPTITUDE_DATASET, soit passe --datasets-root" >&2
  exit 1
fi
if [[ ! -d "$SOURCE" ]]; then
  echo "❌ SOURCE introuvable: $SOURCE" >&2
  exit 1
fi
if [[ "$MODE" != "copy" && "$MODE" != "move" ]]; then
  echo "❌ MODE invalide: $MODE (attendu: copy|move)" >&2
  exit 1
fi

# Mappings
OA_SUB=("actors" "activities" "exchanges" "capabilities" "entities")
OTHER_SUB=("diagram" "processes")

TARGET_BASE="$DATASETS_ROOT/arcadia/$VERSION"
TARGET_OA="$TARGET_BASE/data/oa"
TARGET_DIAGRAM="$TARGET_BASE/data/diagram"
TARGET_PROCESSES="$TARGET_BASE/data/processes"

mkdir -p "$TARGET_OA" "$TARGET_DIAGRAM" "$TARGET_PROCESSES"

echo "➡  SOURCE         : $SOURCE"
echo "➡  DATASETS_ROOT  : $DATASETS_ROOT"
echo "➡  VERSION       : $VERSION"
echo "➡  MODE          : $MODE"
echo "➡  CIBLE         : $TARGET_BASE"

copy_or_move() {
  local src="$1"
  local dst="$2"
  local mode="$3"
  mkdir -p "$dst"
  if [[ -d "$src" ]]; then
    if [[ "$mode" == "copy" ]]; then
      cp -a "$src/." "$dst/"
    else
      # move
      shopt -s dotglob
      mv "$src/"* "$dst/" 2>/dev/null || true
      shopt -u dotglob
    fi
  fi
}

# OA
for sub in "${OA_SUB[@]}"; do
  if [[ -d "$SOURCE/$sub" ]]; then
    echo " -> $sub"
    copy_or_move "$SOURCE/$sub" "$TARGET_OA/$sub" "$MODE"
  else
    echo "    (absent) $SOURCE/$sub"
  fi
done

# Diagram / Processes
for sub in "${OTHER_SUB[@]}"; do
  if [[ -d "$SOURCE/$sub" ]]; then
    echo " -> $sub"
    if [[ "$sub" == "diagram" ]]; then
      copy_or_move "$SOURCE/$sub" "$TARGET_DIAGRAM" "$MODE"
    else
      copy_or_move "$SOURCE/$sub" "$TARGET_PROCESSES" "$MODE"
    fi
  else
    echo "    (absent) $SOURCE/$sub"
  fi
done

# manifest.json
MANIFEST_PATH="$TARGET_BASE/manifest.json"
echo "==> Génération du manifest: $MANIFEST_PATH"
TMP_MANIFEST="$(mktemp)"
cat > "$TMP_MANIFEST" <<'JSON'
{
  "name": "arcadia",
  "version": "__VERSION__",
  "license": "CC-BY-4.0",
  "provenance": {
    "author": "GenAptitude",
    "generatedAt": "__NOW__"
  },
  "collections": {
    "arcadia_oa_actors":       { "schema": "arcadia/oa/actor.schema.json",             "dir": "data/oa/actors" },
    "arcadia_oa_activities":   { "schema": "arcadia/oa/activity.schema.json",          "dir": "data/oa/activities" },
    "arcadia_oa_exchanges":    { "schema": "arcadia/oa/activity-exchange.schema.json", "dir": "data/oa/exchanges" },
    "arcadia_oa_capabilities": { "schema": "arcadia/oa/capability.schema.json",        "dir": "data/oa/capabilities" },
    "arcadia_oa_entities":     { "schema": "arcadia/oa/entity.schema.json",            "dir": "data/oa/entities" },
    "arcadia_class_diagrams":  { "schema": "arcadia/metamodel/class-diagram.schema.json", "dir": "data/diagram" },
    "arcadia_oa_processes":    { "schema": "arcadia/oa/process.schema.json",           "dir": "data/processes" }
  },
  "contexts": [
    "schemas/v1/@context/arcadia.jsonld"
  ]
}
JSON

NOW="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
sed -e "s/__VERSION__/$VERSION/g" -e "s/__NOW__/$NOW/g" "$TMP_MANIFEST" > "$MANIFEST_PATH"
rm -f "$TMP_MANIFEST"

echo "✅ Migration terminée."
echo "   → Manifest : $MANIFEST_PATH"
