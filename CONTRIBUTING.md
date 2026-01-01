# Contributing to RAISE

<p align="center">
  <img src="src/assets/images/logo-white.svg" alt="RAISE Logo" width="150">
</p>

Thank you for your interest in **RAISE** (Rationalized Advanced Intelligence System Engine).

By contributing to this project, you are helping build a **Digital Commons Infrastructure** for European critical engineering. We value sovereignty, rigor, and linguistic precision.

---

## 🇪🇺 Contribution Channels by Language

RAISE enforces a specific **Linguistic Strategy**:

- **Code (Rust/JS/WASM)**: Strict English.
- **Semantics (Models/Docs/Rules)**: Local language (French, German, etc.) is encouraged for precision in engineering domains.

Please select your area of contribution:

| Scope                | Language        | Guidelines & Location                                              |
| :------------------- | :-------------- | :----------------------------------------------------------------- |
| **💻 Core Code**     | 🇺🇸 **English**  | See [General Workflow](#-development-workflow) below.              |
| **📄 Documentation** | 🇫🇷 **Français** | Go to [`docs/fr/`](docs/fr/README.md) to improve French semantics. |
| **📄 Documentation** | 🇩🇪 **Deutsch**  | Go to [`docs/de/`](docs/de/README.md) (Help needed!).              |
| **📄 Documentation** | 🇪🇸 **Español**  | Go to [`docs/es/`](docs/es/README.md) (Help needed!).              |

---

## 🏛️ Contribution Philosophy

Every contribution must respect the project's four pillars:

1.  **Sovereignty (Local-First)**: We refuse mandatory cloud dependencies. Features must work offline or on Air-Gapped networks.
2.  **Rationalization (Explainability)**: AI must not be a black box. Agents must be auditable via the Traceability module (DO-178C compliant logs).
3.  **Rigor (Validation)**: Targeted for critical industries (Aerospace, Defense). Tests are mandatory.
4.  **Digital Commons**: We prioritize open standards (JSON-LD, W3C, ONNX) over proprietary formats.

---

## 🛠️ Development Workflow (Core Code)

### 1. Setup

Ensure you have read the "Installation" section in the main `README.md`.
Required: **Rust 1.88+**, **Node.js 20+**, and **WASM targets**.

### 2. Pull Request (PR) Process

1.  **Fork** the repository.
2.  Create a branch: `git checkout -b feat/my-feature`.
3.  **Compile & Test** locally:
    - Backend: `cargo test` (Runs Rust units & integration tests)
    - Wasm: `cd src-wasm && ./build.sh` (Compiles Cognitive Blocks)
    - Frontend: `npm run build`
4.  Submit PR to `main` branch.

### 3. Commit Convention

We use **Conventional Commits** in English.
Format: `type(scope): short description`

- `feat`: New feature (e.g., `feat(genetics): add optimization solver`)
- `fix`: Bug fix (e.g., `fix(json_db): fix WAL corruption`)
- `docs`: Documentation changes
- `style`: Formatting, missing semi-colons, etc.
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `test`: Adding missing tests

---

## 🧩 Architecture & Code Standards

### Backend (Rust - `src-tauri`)

- **Architecture Layers**: Respect the clean separation (API -> Service -> Core Domain).
- **Idiomatic Rust**: Use `clippy` before committing: `cargo clippy`.
- **Error Handling**: No `unwrap()` in production code. Use Pattern Matching or `?`.

### Cognitive Blocks (WASM - `src-wasm`)

- **Performance**: Minimize memory allocation loops. These modules run in the UI thread context.

### Frontend (React/TypeScript)

- **Strict Typing**: No `any`.
- **I18n**: All UI strings must be extracted to `locales/en/translation.json`.

---

## ⚖️ License & DCO

By contributing, you agree that your contributions will be licensed under the **Apache License 2.0**.

### Developer Certificate of Origin (DCO)

To ensure the legal traceability of this Common Good, please sign your commits (`git commit -s`).
This certifies that you have the right to submit this code.

---

**Together, let's build the future of Sovereign Engineering.**
