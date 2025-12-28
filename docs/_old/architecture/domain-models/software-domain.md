# Modèle de Domaine : Software Engineering

**Domaine :** Ingénierie Logicielle
**Version :** 1.1
**Statut :** Actif (Support Model Engine & Code Generator)

---

## 1\. Vue d'ensemble

Le domaine **Software** de GenAptitude est responsable de la conception, de la modélisation et de la génération des artefacts logiciels. Il fait le pont entre l'architecture système abstraite (Arcadia SA/LA) et l'implémentation concrète (Code).

Il repose sur une **sémantique forte (JSON-LD)** pour garantir que les composants logiciels, leurs interfaces et leurs dépendances sont correctement interprétés par le **Model Engine** et les agents IA.

---

## 2\. Ontologie & Sémantique

Le modèle logiciel utilise un vocabulaire dédié qui étend ou précise les concepts Arcadia.

### Namespace

- **URI de base** : `https://genaptitude.io/vocab/software#`
- **Préfixe standard** : `sw`

### Types Principaux

| Concept GenAptitude | Type JSON-LD (`@type`) | Mapping Arcadia (Model Engine)            | Description                                                    |
| :------------------ | :--------------------- | :---------------------------------------- | :------------------------------------------------------------- |
| **Component**       | `sw:Component`         | `pa:PhysicalComponent` (Nature: Behavior) | Unité de déploiement logiciel (Service, Bibliothèque, Module). |
| **Interface**       | `sw:Interface`         | `pa:Interface`                            | Contrat d'interaction (API, Protocole).                        |
| **Dependency**      | `sw:dependsOn`         | `pa:ComponentExchange`                    | Lien de dépendance technique.                                  |
| **Service**         | `sw:Service`           | `pa:PhysicalFunction`                     | Fonctionnalité exposée par un composant.                       |

> **Note d'Architecture** : Bien que le domaine Software ait son propre vocabulaire, le **ModelLoader** projette ces entités dans les structures `PhysicalArchitecture` du `ProjectModel` en mémoire pour assurer la cohérence globale.

---

## 3\. Structure des Données (JSON Schema)

Les documents stockés dans la `json_db` respectent le schéma strict `component.schema.json`.

### Exemple de Document (`component.json`)

```json
{
  "@context": {
    "sw": "https://genaptitude.io/vocab/software#",
    "id": "@id",
    "type": "@type",
    "name": "http://schema.org/name",
    "dependencies": { "@id": "sw:dependsOn", "@type": "@id" }
  },
  "id": "urn:uuid:auth-service-001",
  "type": "sw:Component",
  "name": "Authentication Service",
  "sw:componentType": "microservice",
  "version": "1.0.0",
  "interfaces": [
    {
      "name": "ILogin",
      "type": "input",
      "protocol": "gRPC"
    }
  ],
  "dependencies": ["urn:uuid:user-db-connector-002"],
  "metadata": {
    "language": "rust",
    "framework": "actix-web"
  }
}
```

### Règles de Validation (Model Engine)

Lors de l'insertion via `jsondb_insert_with_schema` :

1.  **x_compute** : Génère l'ID (UUID v4) et les timestamps (`createdAt`, `updatedAt`) s'ils sont absents.
2.  **Validation Schéma** : Vérifie que `sw:componentType` appartient à l'énumération autorisée (`service`, `library`, `module`, `function`).
3.  **Intégrité** : Le `TransactionManager` assure que la création du composant et la mise à jour de ses index (ex: recherche par `framework`) sont atomiques.

---

## 4\. Intégration Arcadia

Le domaine Software s'inscrit principalement dans les couches basses d'Arcadia :

### Logical Architecture (LA)

- Les composants logiciels réalisent des **Logical Components**.
- Exemple : Le composant logique "Gestionnaire Identité" devient le composant logiciel "Auth Service".

### Physical Architecture (PA)

- C'est le lieu de résidence principal du modèle logiciel.
- **Node** (Matériel/Conteneur) : Héberge les composants logiciels.
- **Behavior** (Logiciel) : Représente le code lui-même.

Le **Model Loader** utilise l'expansion JSON-LD pour classer automatiquement les entités `sw:Component` dans la couche `PhysicalArchitecture` (`model.pa.components`) lors du chargement du projet.

---

## 5\. Génération de Code

Le modèle logiciel est la source primaire pour le module `code_generator`.

| Langage Cible  | Templates Utilisés            | Stratégie                                          |
| :------------- | :---------------------------- | :------------------------------------------------- |
| **Rust**       | `microservices-template.json` | Génération de Structs, Traits et `Cargo.toml`.     |
| **TypeScript** | `web-app-example`             | Génération d'interfaces et de clients API.         |
| **C++**        | `embedded-board`              | Génération de headers `.hpp` et squelettes `.cpp`. |

### Flux de Génération

1.  L'agent IA (`SoftwareAgent`) ou l'utilisateur définit l'architecture.
2.  Les données sont persistées dans `json_db` (Collection `software/`).
3.  La commande `generate_code` charge le modèle via `ModelLoader`.
4.  Le générateur applique les patrons de conception (Patterns) définis dans `domain-models/software/patterns/`.

---

## 6\. Patterns & Templates

Le domaine supporte nativement plusieurs styles architecturaux stockés dans `domain-models/software/templates/` :

- **Monolithique** : Un seul composant `sw:Component` regroupant toutes les fonctions.
- **Microservices** : Grappe de composants communiquant via `sw:Interface` (REST/gRPC).
- **Serverless** : Fonctions unitaires (`sw:componentType` = `function`) déclenchées par événements.

---

## 7\. Sécurité & Traçabilité

- **Preuves (Evidence)** : Chaque modification de l'architecture logicielle (ajout de dépendance, changement d'interface) peut être ancrée sur la Blockchain (Fabric) via la commande `record_decision`.
- **Souveraineté** : Le code et les modèles ne quittent jamais le poste de travail (Local First).
