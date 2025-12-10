#!/bin/bash
set -e

echo "========= CONSTRUCTION DES BLOCS COGNITIFS ========="

PROJECT_ROOT=$(pwd)
WASM_DEST="$PROJECT_ROOT/wasm-modules"

# 1. Compilation (depuis la racine workspace)
echo "--> Compilation de 'analyzer-consistency'..."
cargo build -p genaptitude-block-consistency --target wasm32-unknown-unknown --release

# 2. Déploiement
# Attention : Rust transforme les tirets en underscores dans le nom du fichier
SOURCE_WASM="$PROJECT_ROOT/target/wasm32-unknown-unknown/release/genaptitude_block_consistency.wasm"

echo "--> Installation vers wasm-modules/analyzers..."
mkdir -p "$WASM_DEST/analyzers"
cp "$SOURCE_WASM" "$WASM_DEST/analyzers/consistency_basic.wasm"

echo "========= SUCCÈS : Plugin déployé ========="
ls -lh "$WASM_DEST/analyzers/consistency_basic.wasm"