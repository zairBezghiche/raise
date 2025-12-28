# Modèle de Domaine : System Engineering

**Domaine :** Ingénierie Système (System Analysis)
**Version :** 1.1
**Statut :** Actif (Support Model Engine & Standards)

---

## 1\. Vue d'ensemble

Le domaine **System** de GenAptitude couvre la définition du système en tant que "boîte noire". Il se concentre sur ce que le système doit faire pour satisfaire les besoins opérationnels, sans présumer de l'architecture technique interne.

Il correspond à la couche **SA (System Analysis)** de la méthode Arcadia.

Grâce au **Model Engine**, les entités systèmes sont chargées sémantiquement, permettant une traçabilité rigoureuse vers les besoins (OA) et vers la solution logique (LA).

---

## 2\. Ontologie & Sémantique

Le modèle utilise le vocabulaire standard Arcadia SA pour garantir l'interopérabilité.

### Namespace

- **URI de base** : `https://genaptitude.io/ontology/arcadia/sa#`
- **Préfixe standard** : `sa`

### Types Principaux

| Concept GenAptitude     | Type JSON-LD (`@type`)  | Mapping Model Engine (`model.sa.*`) | Description                                                       |
| :---------------------- | :---------------------- | :---------------------------------- | :---------------------------------------------------------------- |
| **System Function**     | `sa:SystemFunction`     | `functions`                         | Fonctionnalité attendue du système (ex: "Détecter Obstacle").     |
| **System Component**    | `sa:SystemComponent`    | `components`                        | Le système lui-même (frontière) ou des sous-systèmes externes.    |
| **System Actor**        | `sa:SystemActor`        | `actors`                            | Acteur externe interagissant avec le système.                     |
| **Functional Exchange** | `sa:FunctionalExchange` | `exchanges`                         | Flux de données ou de contrôle entre fonctions.                   |
| **Capability**          | `sa:SystemCapability`   | `capabilities`                      | Aptitude du système à rendre un service (regroupe des fonctions). |

> **Validation Sémantique** : Le `ModelLoader` vérifie strictement ces URIs lors du chargement. Si un document possède un type inconnu (ex: `sa:UnknownType`), il est ignoré ou logué en avertissement, garantissant la propreté du modèle en mémoire.

---

## 3\. Structure des Données (JSON Schema)

Les documents sont stockés dans `json_db` et validés par des schémas JSON stricts (ex: `system-function.schema.json`).

### Exemple de Document (`function.json`)

```json
{
  "@context": [
    "https://genaptitude.io/ontology/arcadia/sa.jsonld",
    {
      "criticality": "http://example.org/vocab/safety#criticality"
    }
  ],
  "id": "urn:uuid:func-detect-001",
  "@type": "sa:SystemFunction",
  "name": "Détecter Obstacle",
  "description": "Analyse les flux capteurs pour identifier les dangers.",
  "allocatedTo": ["urn:uuid:system-component-001"],
  "realizedActivities": ["urn:uuid:act-surveiller-environnement-001"],
  "inputs": ["urn:uuid:exchange-video-stream-001"],
  "outputs": ["urn:uuid:exchange-alert-data-001"],
  "criticality": "DAL-A",
  "createdAt": "2025-11-29T10:00:00Z"
}
```

### Règles de Validation

1.  **Allocations** : Une `SystemFunction` doit idéalement être allouée à un `SystemComponent` ou un `SystemActor`.
2.  **x_compute** : Les champs techniques (`id`, `createdAt`, `$schema`) sont calculés automatiquement par le moteur avant la validation du schéma.
3.  **Typage Fort** : Les champs comme `criticality` peuvent être définis via des extensions PVMT (Property Values) et validés par le moteur.

---

## 4\. Intégration Arcadia & Traçabilité

[cite_start]Le domaine System est le pivot de la traçabilité verticale[cite: 305].

### Traçabilité Amont (Vers OA)

- Relation : `realizes`
- Mapping : `SystemFunction` -\> `OperationalActivity`
- Objectif : Justifier chaque fonction par un besoin opérationnel.

### Traçabilité Aval (Vers LA)

- Relation : `realizedBy`
- Mapping : `SystemFunction` \<- `LogicalFunction`
- Objectif : Vérifier que toutes les fonctions système sont couvertes par l'architecture logique.

---

## 5\. Gestion des Exigences & Standards

Le domaine Système intègre la gestion des exigences, souvent liées aux normes de sécurité (Safety/Security).

| Standard      | Templates disponibles                      | Usage                                |
| :------------ | :----------------------------------------- | :----------------------------------- |
| **DO-178C**   | `domain-models/system/standards/do-178c`   | Avionique (Design Assurance Levels). |
| **ISO-26262** | `domain-models/system/standards/iso-26262` | Automobile (ASIL Levels).            |
| **IEC-61508** | `domain-models/system/standards/iec-61508` | Industrie générique.                 |

Les exigences sont modélisées comme des entités à part entière (`Requirement`) liées aux fonctions via la relation `satisfies`.

---

## 6\. Ingénierie Avancée

Le **Model Engine** supporte des concepts avancés pour l'analyse système :

### Chaînes Fonctionnelles

Séquences ordonnées de fonctions et d'échanges traversant le système pour répondre à un scénario critique (ex: "Arrêt d'urgence").

- Stockage : `sa:FunctionalChain`
- Validation : Vérification de la continuité du flux.

### Modes & États

Définition du comportement dynamique du système.

- Stockage : `sa:StateMachine`, `sa:Mode`, `sa:State`.
- Usage : Définir quelles fonctions sont actives dans quels modes (ex: "Mode Maintenance" vs "Mode Opérationnel").

---

## 7\. Sécurité & Blockchain

Pour les systèmes critiques, l'intégrité des données est primordiale.

- [cite_start]**Preuves d'Audit** : Les matrices de traçabilité (Fonction \<-\> Exigence) peuvent être hachées et ancrées sur la Blockchain Fabric via la commande `record_model_snapshot`[cite: 19, 653].
- **Validation ACID** : Toute modification de l'architecture système (ajout d'une fonction critique) passe par une transaction atomique via le WAL (`_wal.jsonl`), garantissant qu'aucune corruption de modèle n'est possible en cas de crash.
