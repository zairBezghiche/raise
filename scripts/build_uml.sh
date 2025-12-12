#!/bin/bash

# =================================================================
# Script: build_uml.sh
# Description: Crée l'arborescence des répertoires pour les schémas
#              UML (Unified Modeling Language) au sein de GenAptitude.
# =================================================================

BASE_DIR="schemas/v1/uml"

# Vérification : Assurez-vous que le répertoire parent existe.
if [ ! -d "schemas/v1" ]; then
    echo "Le répertoire parent 'schemas/v1' n'existe pas. Veuillez le créer d'abord."
    exit 1
fi

echo "Démarrage de la création de l'arborescence UML sous '$BASE_DIR'..."

# Création des répertoires principaux (structurels)
mkdir -p "$BASE_DIR"/{structure,behavioral,interaction,deployment,common}

# 1. Répertoires pour les diagrammes Structurels
mkdir -p "$BASE_DIR"/structure/{class-diagram,component-diagram,composite-structure-diagram}
echo "  [OK] Structurels (Classes, Composants...)"

# 2. Répertoires pour les diagrammes Comportementaux
mkdir -p "$BASE_DIR"/behavioral/{use-case-diagram,activity-diagram,state-machine-diagram}
echo "  [OK] Comportementaux (Cas d'utilisation, Activités...)"

# 3. Répertoires pour les diagrammes d'Interaction
mkdir -p "$BASE_DIR"/interaction/{sequence-diagram,communication-diagram}
echo "  [OK] Interactions (Séquence, Communication...)"

# 4. Répertoires pour les diagrammes de Déploiement
mkdir -p "$BASE_DIR"/deployment/{deployment-diagram,profile-diagram}
echo "  [OK] Déploiement (Nœuds, Profils...)"

# 5. Répertoires pour les éléments Communs
mkdir -p "$BASE_DIR"/common/data-types
echo "  [OK] Communs (Types de données, Bases...)"

echo ""
echo "Arborescence UML créée avec succès !"
echo "Vous pouvez maintenant commencer à ajouter les schémas JSON-LD (.jsonld) dans ces dossiers."

# Affichage de la structure créée (optionnel, pour vérification)
echo "Structure des répertoires :"
find "$BASE_DIR" -type d