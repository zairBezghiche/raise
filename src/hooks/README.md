# 🪝 Custom Hooks Library

Ce répertoire regroupe les hooks React personnalisés qui encapsulent la logique métier, la gestion d'état complexe et les interactions avec le backend Tauri (Rust). Ils permettent de garder les composants UI propres et focalisés sur le rendu.

## 📂 Liste des Hooks

### 🧠 Intégration IA & Modèle

#### `useRulesEngine` (`useRulesEngine.ts`)

Gère l'interaction réactive avec le moteur de règles **GenRules**.

- **Rôle** : Synchronise un document "brouillon" (Draft) avec le backend pour recalculer les champs dérivés en temps réel.
- **Fonctionnalités** :
  - **Debounce** : Attend que l'utilisateur arrête de taper (défaut 500ms) avant d'appeler le backend.
  - **Calcul** : Appelle la commande `jsondb_evaluate_draft`.
  - **Protection** : Évite les boucles infinies de mise à jour grâce à une référence (`useRef`) du dernier état validé.
- **Usage** : Utilisé par les formulaires de démo (`InvoiceDemo`, `ModelRulesDemo`).

#### `useAIChat` (`useAIChat.ts`)

Encapsule la logique conversationnelle avec les LLMs.

- **Rôle** : Gère l'historique des messages, l'état "Thinking" et l'envoi vers Rust.
- **Backend** : Bascule dynamiquement entre un mode `mock` (simulation JS) et `tauri-local` (appel réel `invoke('ai_chat')`) selon la configuration globale.
- **Store** : Connecté au `useAiStore` pour persister la session.

#### `useCodeGeneration` (`useCodeGeneration.ts`)

Gère le processus de génération de code source à partir du modèle.

- **Rôle** : Coordonne la demande de génération vers le `codegenService`.
- **Contexte** : Utilise automatiquement le `currentProject` chargé dans le `ModelStore`.
- **Utilitaire** : Fournit une méthode `copyToClipboard` pour copier le résultat.

#### `useModelState` (`useModelState.ts`)

Façade simplifiée pour accéder au `ModelStore` (Arcadia/Capella).

- **Rôle** : Fournit des accesseurs dérivés pratiques (ex: `selectedElement` objet complet au lieu de juste l'ID) et les actions de mutation.
- **Avantage** : Abstrait la complexité de `Zustand` pour les composants simples.

### 🛠️ Utilitaires Système

#### `useFileSystem` (`useFileSystem.ts`)

Wrapper autour de l'API Fichiers de Tauri v2 (`@tauri-apps/plugin-fs`).

- **Rôle** : Simplifie la lecture/écriture de fichiers JSON typés.
- **Sécurité** : Configure par défaut le `BaseDirectory.AppLocalData` pour isoler les données de l'application.

#### `useTauriEvent` (`useTauriEvent.ts`)

Abonnement déclaratif aux événements globaux Tauri.

- **Rôle** : Attache un écouteur d'événement (`listen`) au montage du composant et le nettoie (`unlisten`) automatiquement au démontage.
- **Usage** : Idéal pour écouter les logs backend ou les notifications asynchrones.

## 📦 Exemple d'Utilisation

```typescript
import { useRulesEngine } from '@/hooks/useRulesEngine';

function MyForm() {
  // Le hook gère tout le cycle de vie : saisie -> debounce -> calcul -> mise à jour
  const { doc, handleChange, isCalculating } = useRulesEngine({
    space: 'demo',
    db: 'test',
    collection: 'invoices',
    initialDoc: { total: 0 },
  });

  return (
    <div>
      <input onChange={(e) => handleChange('qty', e.target.value)} />
      {isCalculating && <span>Calcul en cours...</span>}
      <div>Total: {doc.total}</div>
    </div>
  );
}
```
