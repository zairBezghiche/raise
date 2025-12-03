# Modèle de Domaine : Hardware Engineering

**Domaine :** Ingénierie Matérielle (Electronics & Mechanics)
**Version :** 1.1
**Statut :** Actif (Support Model Engine, HDL Generation & EPBS)

---

## 1\. Vue d'ensemble

Le domaine **Hardware** de GenAptitude couvre la conception physique du système : cartes électroniques, boîtiers mécaniques, câblage et composants FPGA/ASIC.

Il correspond principalement à la couche **PA (Physical Architecture)** et **EPBS (End Product Breakdown Structure)** de la méthode Arcadia.

Grâce au **Model Engine**, les entités matérielles sont chargées sémantiquement, permettant de lier les contraintes physiques (thermique, électrique) aux choix d'architecture logique.

---

## 2\. Ontologie & Sémantique

Le modèle utilise un vocabulaire dédié (`hw`) qui étend les concepts physiques d'Arcadia.

### Namespace

- **URI de base** : `https://genaptitude.io/vocab/hardware#`
- **Préfixe standard** : `hw`

### Types Principaux

| Concept GenAptitude    | Type JSON-LD (`@type`) | Mapping Model Engine           | Description                                        |
| :--------------------- | :--------------------- | :----------------------------- | :------------------------------------------------- |
| **Hardware Component** | `hw:Component`         | `pa.components` (Nature: Node) | Composant physique (IC, Résistance, Connecteur).   |
| **Board / Module**     | `hw:Module`            | `pa.components` (Node)         | Assemblage de composants (PCB, Rack).              |
| **Physical Link**      | `hw:Connection`        | `pa.links`                     | Liaison physique (Piste, Nappe, Câble).            |
| **Pin / Port**         | `hw:Pin`               | `pa.ports`                     | Point de connexion physique.                       |
| **Configuration Item** | `hw:Part`              | `epbs.configurationItems`      | Élément de la nomenclature (BOM) pour fabrication. |

> **Note** : Le `ModelLoader` utilise l'expansion JSON-LD pour projeter ces types métier vers les structures génériques `PhysicalComponent` ou `PhysicalLink` en mémoire.

---

## 3\. Structure des Données (JSON Schema)

Les données sont validées par le schéma `domain-models/hardware/json-schemas/component.schema.json`.

### Exemple de Document (`resistor.json`)

```json
{
  "@context": {
    "hw": "https://genaptitude.io/vocab/hardware#",
    "pa": "https://genaptitude.io/ontology/arcadia/pa#",
    "specifications": "hw:specifications"
  },
  "id": "urn:uuid:res-0402-10k",
  "@type": ["hw:Component", "pa:PhysicalComponent"],
  "name": "R_10k_0402",
  "hw:partNumber": "RC0402JR-0710KL",
  "hw:manufacturer": "Yageo",
  "nature": "Node",
  "specifications": {
    "resistance": 10000,
    "tolerance": 0.05,
    "power_rating": 0.0625,
    "package": "0402"
  },
  "pins": [
    { "number": "1", "type": "passive" },
    { "number": "2", "type": "passive" }
  ],
  "createdAt": "2025-11-29T14:00:00Z"
}
```

### Règles de Validation

1.  **Spécifications** : Les objets `specifications` sont validés pour garantir que les valeurs physiques (Voltage, Courant, Puissance) sont des nombres valides.
2.  **Nomenclature** : Un composant matériel doit idéalement avoir un `partNumber` et un `manufacturer` pour être valide dans la couche EPBS.
3.  **x_compute** : Calcul automatique des IDs et de la version du schéma.

---

## 4\. Intégration Arcadia

Le matériel est la concrétisation physique de la solution.

### Physical Architecture (PA)

Les composants matériels (`Node`) hébergent les composants logiciels (`Behavior`).

- **Relation** : `hosts` / `deployedOn`
- **Exemple** : Le microcontrôleur "STM32" (Node) héberge le firmware "MotorController" (Behavior).

### EPBS (End Product Breakdown Structure)

Le matériel définit la structure du produit final à fabriquer.

- **Composition** : Arborescence Système -\> Sous-système -\> Carte -\> Composant.
- **Livrable** : Génération de la **BOM (Bill of Materials)** à partir des `ConfigurationItems`.

---

## 5\. Génération HDL & Fabrication

Le module `code_generator` dispose de générateurs spécifiques pour le matériel.

| Langage / Format | Templates               | Usage                                                  |
| :--------------- | :---------------------- | :----------------------------------------------------- |
| **VHDL**         | `hdl-templates/vhdl`    | Description comportementale ou structurelle pour FPGA. |
| **Verilog**      | `hdl-templates/verilog` | Alternative pour la conception ASIC/FPGA.              |
| **Netlist**      | (Prévu)                 | Liste des connexions pour le routage PCB.              |

### Flux de Génération

1.  Définition des composants et connexions dans l'éditeur Hardware.
2.  Persistance dans `json_db`.
3.  Transformation par le **Model Engine** en graphe de connectivité.
4.  Génération du code HDL (`entity`/`architecture` en VHDL) mappant les ports et signaux.

---

## 6\. Contraintes & Simulations

Le modèle matériel supporte la définition de contraintes physiques strictes, vérifiées par les **Validators**.

- **Bilan de Puissance** : Somme des `power_rating` ou consommation max des composants d'une carte vs capacité de l'alimentation.
- **Thermique** : Vérification des plages de température (`temperature_range`) vs environnement opérationnel.
- **Intégrité du Signal** : Contraintes sur les `PhysicalLink` (impédance, longueur max).

---

## 7\. Sécurité & Supply Chain

- **Traçabilité des composants** : Chaque `partNumber` critique peut être associé à une preuve d'origine (certificat) ancrée sur la Blockchain Fabric.
- **Obsolescence** : Les métadonnées du modèle peuvent inclure des statuts de cycle de vie (Active, NRND, EOL) pour audit automatique.
