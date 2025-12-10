### Fichier : `src-wasm/README.md`

````markdown
# 🧠 GenAptitude - Cognitive Blocks (WASM)

Ce répertoire contient la logique "intelligente" de GenAptitude, découpée en **Blocs Cognitifs**.
Contrairement à une approche monolithique, chaque sous-module ici est compilé en un fichier `.wasm` indépendant, chargé dynamiquement par le backend Tauri (via Wasmtime).

## 🏗 Architecture

L'architecture repose sur un système de plugins strict :

1.  **`core-api` (Le Contrat)** : Une librairie Rust standard qui définit les types de données partagés (`CognitiveModel`, `AnalysisReport`) et les traits. Tous les blocs dépendent de ceci.
2.  **`blocks/*` (Les Plugins)** : Chaque dossier est une crate indépendante qui implémente une logique spécifique (Analyse, Parsing, Optimisation).
3.  **Hébergement** : Ces blocs ne tournent PAS dans le navigateur. Ils tournent dans une sandbox WASM gérée par le processus Rust principal (Tauri).

## 📂 Structure du Dossier

```text
src-wasm/
├── Cargo.toml          # Workspace virtuel (pas de [workspace], gestion via racine)
├── core-api/           # Types partagés et Trait 'CognitiveBlock'
│   ├── src/lib.rs
│   └── Cargo.toml
└── blocks/             # Les implémentations concrètes
    ├── analyzer-consistency/  # Exemple : Vérification de règles
    │   ├── src/lib.rs         # Contient la logique + l'interface FFI
    │   └── Cargo.toml         # Configuré en 'cdylib'
    ├── parser-capella/        # (Futur)
    └── ...
```
````

## 🔌 Le Protocole d'Échange (Memory Model)

Puisque nous utilisons **Wasmtime** (et non un navigateur web), nous ne pouvons pas utiliser les bindings JS automatiques de `wasm-bindgen`.
La communication se fait via la **Mémoire Partagée** et **JSON**.

### Cycle de vie d'un appel :

1.  **Tauri** sérialise les données en JSON (`String`).
2.  **Tauri** appelle `alloc(size)` dans le WASM pour réserver de la mémoire.
3.  **Tauri** écrit les octets du JSON dans cette mémoire.
4.  **Tauri** appelle `run_analysis(ptr, len)`.
5.  **WASM** lit la mémoire, désérialise le JSON, exécute la logique, et sérialise la réponse.
6.  **WASM** retourne un pointeur "packé" vers la réponse.
7.  **Tauri** lit la réponse et la désérialise.

## 🚀 Comment créer un nouveau Bloc Cognitif

### 1\. Créer la crate

Dans `src-wasm/blocks/` :

```bash
cargo new --lib mon-nouveau-bloc
```

### 2\. Configurer `Cargo.toml`

Le bloc doit être une librairie dynamique (`cdylib`) pour générer du WASM.

```toml
[package]
name = "genaptitude-block-nouveau"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"] # INDISPENSABLE

[dependencies]
genaptitude-core-api = { path = "../../core-api" }
serde = { workspace = true }
serde_json = { workspace = true }
```

### 3\. Implémenter le Boilerplate FFI

Dans `lib.rs`, en plus de votre logique, vous devez exposer ces fonctions pour l'hôte :

```rust
use std::mem;

// Logique interne
struct MonBloc;
impl CognitiveBlock for MonBloc { ... }

// Interface Système (Boilerplate obligatoire)
#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut u8 { ... }

#[no_mangle]
pub unsafe extern "C" fn run_analysis(ptr: *mut u8, len: usize) -> u64 { ... }
```

_(Voir `blocks/analyzer-consistency/src/lib.rs` pour l'implémentation de référence)_

## 🛠 Compilation et Déploiement

Ne compilez pas manuellement avec `cargo build` si vous voulez tester l'intégration. Utilisez le script de déploiement qui place les fichiers au bon endroit (`wasm-modules`).

```bash
# Depuis la racine du projet
./scripts/build_plugins.sh
```

Cela génère : `target/wasm32-unknown-unknown/release/xxx.wasm`
Et le copie vers : `wasm-modules/analyzers/xxx.wasm`

## ⚠️ Notes Importantes

- **Pas de `wasm-bindgen`** : Ne l'utilisez pas pour générer du JS. Nous sommes en Rust-to-Wasm pur.
- **Sandboxing** : Le code WASM n'a pas accès au disque, au réseau ou à l'heure système, sauf si nous lui passons des fonctions importées (Host Functions).
- **Panic** : Si le code WASM panic, l'hôte reçoit une erreur `Trapped`, mais l'application GenAptitude ne crashe pas.

<!-- end list -->

```

```
