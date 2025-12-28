#!/bin/bash

# --- CONFIGURATION ---
# Définition du dossier cible (utilisation de $HOME pour le chemin absolu)
OUTPUT_DIR="$HOME/genaptitude_zip"
OUTPUT_FILE="$OUTPUT_DIR/genaptitude_context.txt"

# Dossiers à ignorer (pour éviter de scanner target, node_modules, etc.)
IGNORE_PATTERN="target|node_modules|.git|dist|wasm-modules|build"

# --- DÉMARRAGE ---
echo "🚀 Démarrage de la génération du contexte pour LLM..."
echo "📂 Racine du projet analysée : $(pwd)"

# 1. Création du répertoire de destination si nécessaire
if [ ! -d "$OUTPUT_DIR" ]; then
    echo "🔨 Le dossier cible n'existe pas. Création de : $OUTPUT_DIR"
    mkdir -p "$OUTPUT_DIR"
else
    echo "📂 Dossier cible détecté : $OUTPUT_DIR"
fi

# 2. Initialisation du fichier (Écrasement du précédent)
echo "==============================================================================" > "$OUTPUT_FILE"
echo " PROJECT: GenAptitude (Rust/WASM/Tauri)" >> "$OUTPUT_FILE"
echo " GENERATED ON: $(date)" >> "$OUTPUT_FILE"
echo " CONTENT: Project Tree + All Markdown Documentation" >> "$OUTPUT_FILE"
echo "==============================================================================" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# 3. Génération de l'arborescence (Tree)
echo "🌳 Génération de l'arborescence..."
echo "### SECTION 1: PROJECT STRUCTURE ###" >> "$OUTPUT_FILE"
echo '```' >> "$OUTPUT_FILE"
if command -v tree &> /dev/null; then
    # On reste à la racine (.) pour tree, mais on redirige vers le fichier externe
    tree -I "$IGNORE_PATTERN" >> "$OUTPUT_FILE"
else
    find . -maxdepth 4 -not -path '*/.*' >> "$OUTPUT_FILE"
fi
echo '```' >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
echo "==============================================================================" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# 4. Agrégation des fichiers Markdown
echo "📄 Récupération des fichiers Markdown..."
echo "### SECTION 2: MARKDOWN DOCUMENTATION ###" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# On cherche tous les .md en excluant les dossiers indésirables
find . -type f -name "*.md" \
    -not -path "*/target/*" \
    -not -path "*/node_modules/*" \
    -not -path "*/.git/*" \
    -not -path "*/dist/*" \
    -not -path "*/wasm-modules/*" \
    | sort | while read -r file; do
    
    echo "  -> Ajout de : $file"
    
    # En-tête contextuel
    echo "------------------------------------------------------------------------------" >> "$OUTPUT_FILE"
    echo "FILE PATH: $file" >> "$OUTPUT_FILE"
    echo "------------------------------------------------------------------------------" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    
    # Contenu du fichier
    cat "$file" >> "$OUTPUT_FILE"
    
    echo "" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
done

echo "✅ Terminé ! Le fichier est prêt ici :"
echo "👉 $OUTPUT_FILE"