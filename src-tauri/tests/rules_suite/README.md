# Suite de Tests : Rules Engine (GenRules)

Ce dossier contient la suite de tests dédiée au moteur de règles déclaratif **GenRules**. Ces tests vérifient à la fois la logique pure des expressions (AST, Evaluateur) et l'intégration complète dans le cycle de vie de la base de données (Collections, Lookup, Persistance).

## 📂 Structure

La suite est divisée en deux catégories principales :

1.  **Tests Unitaires Logiques** (`logic_scenarios.rs`) : Valident les opérateurs atomiques (Maths, Logique) de manière isolée, sans accès disque ni base de données.
2.  **Tests d'Intégration Système** (`rules_integration.rs`) : Valident le comportement de bout en bout ("End-to-End"), incluant la définition de schémas, l'insertion en base et les effets de bord (Lookup cross-collection).

---

## 🧪 Scénarios de Test

### 1\. Logique Pure (`logic_scenarios.rs`)

Ces tests utilisent un `NoOpDataProvider` pour isoler le moteur de règles.

- **Logique Booléenne Complexe** : Vérifie l'imbrication des opérateurs `AND`, `OR`, `GT`, `EQ`.
  - _Cas testé_ : `(age > 18 AND status == "member") OR role == "admin"`.
- **Précédence Mathématique** : Vérifie que l'ordre des opérations est respecté via la structure de l'AST (les parenthèses implicites de l'arbre).
  - _Cas testé_ : `(price - cost) / price` (Calcul de marge).

### 2\. Intégration Système (`rules_integration.rs`)

Ces tests créent un environnement temporaire complet (`tempdir`) avec une vraie structure de fichiers JSON-DB.

- **Cycle de Vie "End-to-End"** :
  1.  Initialisation d'une DB temporaire.
  2.  Création dynamique d'un schéma JSON contenant des `x_rules` (calcul de total `qty * price`).
  3.  Insertion d'un document brut.
  4.  Vérification que le document persisté contient bien les champs calculés (`total` et `category`).
- **Règles Avancées & Lookup** :
  - Scénario réaliste de facturation.
  - Collection `users` avec un TJM (Taux Journalier).
  - Collection `invoices` qui calcule son total en allant chercher le TJM de l'utilisateur via un **Lookup** (`user_id` -\> `tjm`).
  - Calcul de dates (`date_add` pour l'échéance) et génération de référence (`concat` + `upper`).

## 🚀 Lancer les Tests

Pour exécuter uniquement cette suite de tests :

```bash
cargo test --test rules_suite
```

Pour voir les logs détaillés (valeurs calculées, JSON générés) :

```bash
cargo test --test rules_suite -- --nocapture
```

## ⚠️ Notes Techniques

- **Mocking** : Les tests unitaires utilisent `NoOpDataProvider` pour simuler l'absence de base de données.
- **Chemins Relatifs** : Les tests d'intégration sont sensibles aux chemins des schémas (`v1/orders.json`). Le `CollectionsManager` s'attend à une structure précise (`v1/COLLECTION/schema.json`) pour résoudre les URIs correctement.
- **Atomicité** : Chaque test d'intégration crée son propre répertoire temporaire, garantissant qu'il n'y a pas d'effets de bord entre les tests.
