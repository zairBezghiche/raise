//! Gestion des contextes JSON-LD pour les phases Arcadia
//!
//! Ce module permet de charger, fusionner et résoudre les contextes JSON-LD
//! pour chaque phase de la méthode Arcadia (OA, SA, LA, PA, EPBS).

use anyhow::{Context as AnyhowContext, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::ContextValue;

/// Représente un contexte JSON-LD complet avec métadonnées
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcadiaContext {
    /// Version du contexte
    #[serde(rename = "@version", skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Vocabulaire par défaut
    #[serde(rename = "@vocab", skip_serializing_if = "Option::is_none")]
    pub vocab: Option<String>,

    /// Mappings de termes vers des IRIs
    #[serde(flatten)]
    pub mappings: HashMap<String, ContextValue>,
}

impl ArcadiaContext {
    /// Crée un nouveau contexte vide
    pub fn new() -> Self {
        Self {
            version: Some("1.1".to_string()),
            vocab: None,
            mappings: HashMap::new(),
        }
    }

    /// Crée un contexte avec un vocabulaire par défaut
    pub fn with_vocab(vocab: impl Into<String>) -> Self {
        Self {
            version: Some("1.1".to_string()),
            vocab: Some(vocab.into()),
            mappings: HashMap::new(),
        }
    }

    /// Ajoute un mapping simple (terme -> IRI)
    pub fn add_simple_mapping(&mut self, term: impl Into<String>, iri: impl Into<String>) {
        self.mappings
            .insert(term.into(), ContextValue::Simple(iri.into()));
    }

    /// Ajoute un mapping étendu avec type et container
    pub fn add_expanded_mapping(
        &mut self,
        term: impl Into<String>,
        id: impl Into<String>,
        type_: Option<String>,
        container: Option<String>,
    ) {
        self.mappings.insert(
            term.into(),
            ContextValue::Expanded {
                id: id.into(),
                type_,
                container,
            },
        );
    }

    /// Fusionne un autre contexte dans celui-ci
    pub fn merge(&mut self, other: &ArcadiaContext) {
        // Mettre à jour le vocabulaire si présent
        if let Some(vocab) = &other.vocab {
            self.vocab = Some(vocab.clone());
        }

        // Fusionner les mappings
        for (key, value) in &other.mappings {
            self.mappings.insert(key.clone(), value.clone());
        }
    }

    /// Résout un terme en IRI complet
    pub fn resolve_term(&self, term: &str) -> Option<String> {
        // Chercher d'abord dans les mappings
        if let Some(value) = self.mappings.get(term) {
            return match value {
                ContextValue::Simple(iri) => Some(iri.clone()),
                ContextValue::Expanded { id, .. } => Some(id.clone()),
            };
        }

        // Utiliser le vocabulaire par défaut si disponible
        if let Some(vocab) = &self.vocab {
            return Some(format!("{}{}", vocab, term));
        }

        None
    }

    /// Vérifie si un terme est défini dans le contexte
    pub fn has_term(&self, term: &str) -> bool {
        self.mappings.contains_key(term)
    }

    /// Récupère tous les termes définis
    pub fn terms(&self) -> Vec<&String> {
        self.mappings.keys().collect()
    }
}

impl Default for ArcadiaContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Gestionnaire de contextes pour les différentes phases Arcadia
#[derive(Debug, Clone)]
pub struct ContextManager {
    /// Contextes par phase
    contexts: HashMap<ArcadiaLayer, ArcadiaContext>,

    /// Contexte racine (commun à toutes les phases)
    root_context: ArcadiaContext,
}

/// Couches (phases) de la méthode Arcadia
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArcadiaLayer {
    /// Operational Analysis
    #[serde(rename = "oa")]
    OA,
    /// System Analysis
    #[serde(rename = "sa")]
    SA,
    /// Logical Architecture
    #[serde(rename = "la")]
    LA,
    /// Physical Architecture
    #[serde(rename = "pa")]
    PA,
    /// End Product Breakdown Structure
    #[serde(rename = "epbs")]
    EPBS,
}

impl ArcadiaLayer {
    /// Retourne le nom de la couche en minuscules
    pub fn as_str(&self) -> &'static str {
        match self {
            ArcadiaLayer::OA => "oa",
            ArcadiaLayer::SA => "sa",
            ArcadiaLayer::LA => "la",
            ArcadiaLayer::PA => "pa",
            ArcadiaLayer::EPBS => "epbs",
        }
    }

    /// Retourne l'IRI de base pour cette couche
    pub fn base_iri(&self) -> String {
        format!("https://genaptitude.io/ontology/arcadia/{}#", self.as_str())
    }

    /// Retourne toutes les couches dans l'ordre
    pub fn all() -> Vec<ArcadiaLayer> {
        vec![
            ArcadiaLayer::OA,
            ArcadiaLayer::SA,
            ArcadiaLayer::LA,
            ArcadiaLayer::PA,
            ArcadiaLayer::EPBS,
        ]
    }
}

impl ContextManager {
    /// Crée un nouveau gestionnaire de contextes
    pub fn new() -> Self {
        let mut root_context = ArcadiaContext::new();

        // Ajouter les préfixes standards
        root_context.add_simple_mapping("arcadia", "https://genaptitude.io/ontology/arcadia#");
        root_context.add_simple_mapping("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#");
        root_context.add_simple_mapping("rdfs", "http://www.w3.org/2000/01/rdf-schema#");
        root_context.add_simple_mapping("owl", "http://www.w3.org/2002/07/owl#");
        root_context.add_simple_mapping("xsd", "http://www.w3.org/2001/XMLSchema#");
        root_context.add_simple_mapping("skos", "http://www.w3.org/2004/02/skos/core#");
        root_context.add_simple_mapping("dct", "http://purl.org/dc/terms/");
        root_context.add_simple_mapping("foaf", "http://xmlns.com/foaf/0.1/");
        root_context.add_simple_mapping("prov", "http://www.w3.org/ns/prov#");
        root_context.add_simple_mapping("mbse", "http://www.omg.org/spec/SysML/");

        // Ajouter les préfixes pour chaque couche
        for layer in ArcadiaLayer::all() {
            root_context.add_simple_mapping(layer.as_str(), layer.base_iri());
        }

        // Ajouter les raccourcis communs
        root_context.add_simple_mapping("id", "@id");
        root_context.add_simple_mapping("type", "@type");

        // Ajouter les relations transversales
        root_context.add_expanded_mapping(
            "realizes",
            "arcadia:realizes",
            Some("@id".to_string()),
            Some("@set".to_string()),
        );
        root_context.add_expanded_mapping(
            "realizedBy",
            "arcadia:realizedBy",
            Some("@id".to_string()),
            Some("@set".to_string()),
        );
        root_context.add_expanded_mapping(
            "belongsToLayer",
            "arcadia:belongsToLayer",
            Some("@id".to_string()),
            None,
        );

        Self {
            contexts: HashMap::new(),
            root_context,
        }
    }

    /// Charge un contexte depuis un fichier JSON-LD
    pub fn load_context_from_file(
        &mut self,
        layer: ArcadiaLayer,
        path: impl AsRef<Path>,
    ) -> Result<()> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read context file: {:?}", path.as_ref()))?;

        let context: serde_json::Value =
            serde_json::from_str(&content).with_context(|| "Failed to parse context JSON")?;

        // Extraire le contexte du champ @context
        if let Some(context_obj) = context.get("@context") {
            let arcadia_context: ArcadiaContext = serde_json::from_value(context_obj.clone())
                .with_context(|| "Failed to deserialize context")?;

            self.contexts.insert(layer, arcadia_context);
        }

        Ok(())
    }

    /// Charge un contexte depuis une chaîne JSON-LD
    pub fn load_context_from_str(&mut self, layer: ArcadiaLayer, json: &str) -> Result<()> {
        let context: serde_json::Value =
            serde_json::from_str(json).with_context(|| "Failed to parse context JSON")?;

        if let Some(context_obj) = context.get("@context") {
            let arcadia_context: ArcadiaContext = serde_json::from_value(context_obj.clone())
                .with_context(|| "Failed to deserialize context")?;

            self.contexts.insert(layer, arcadia_context);
        }

        Ok(())
    }

    /// Récupère le contexte pour une couche spécifique
    pub fn get_context(&self, layer: ArcadiaLayer) -> Option<&ArcadiaContext> {
        self.contexts.get(&layer)
    }

    /// Récupère un contexte fusionné (root + couche)
    pub fn get_merged_context(&self, layer: ArcadiaLayer) -> ArcadiaContext {
        let mut merged = self.root_context.clone();

        if let Some(layer_context) = self.contexts.get(&layer) {
            merged.merge(layer_context);
        }

        merged
    }

    /// Récupère le contexte racine
    pub fn root_context(&self) -> &ArcadiaContext {
        &self.root_context
    }

    /// Résout un terme dans le contexte d'une couche
    pub fn resolve_term(&self, layer: ArcadiaLayer, term: &str) -> Option<String> {
        let merged = self.get_merged_context(layer);
        merged.resolve_term(term)
    }

    /// Vérifie si un terme existe dans une couche
    pub fn has_term(&self, layer: ArcadiaLayer, term: &str) -> bool {
        let merged = self.get_merged_context(layer);
        merged.has_term(term)
    }

    /// Charge tous les contextes depuis un répertoire
    pub fn load_all_contexts(&mut self, base_path: impl AsRef<Path>) -> Result<()> {
        let base = base_path.as_ref();

        for layer in ArcadiaLayer::all() {
            let context_file = base.join(format!("{}.jsonld", layer.as_str()));

            if context_file.exists() {
                self.load_context_from_file(layer, &context_file)
                    .with_context(|| format!("Failed to load context for layer: {:?}", layer))?;
            }
        }

        Ok(())
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = ArcadiaContext::new();
        assert_eq!(ctx.version, Some("1.1".to_string()));
        assert!(ctx.vocab.is_none());
    }

    #[test]
    fn test_simple_mapping() {
        let mut ctx = ArcadiaContext::new();
        ctx.add_simple_mapping("name", "http://example.org/name");

        let resolved = ctx.resolve_term("name");
        assert_eq!(resolved, Some("http://example.org/name".to_string()));
    }

    #[test]
    fn test_vocab_resolution() {
        let ctx = ArcadiaContext::with_vocab("http://example.org/vocab#");

        let resolved = ctx.resolve_term("Activity");
        assert_eq!(
            resolved,
            Some("http://example.org/vocab#Activity".to_string())
        );
    }

    #[test]
    fn test_context_merge() {
        let mut ctx1 = ArcadiaContext::new();
        ctx1.add_simple_mapping("name", "http://ex1.org/name");

        let mut ctx2 = ArcadiaContext::new();
        ctx2.add_simple_mapping("age", "http://ex2.org/age");

        ctx1.merge(&ctx2);

        assert!(ctx1.has_term("name"));
        assert!(ctx1.has_term("age"));
    }

    #[test]
    fn test_layer_enum() {
        assert_eq!(ArcadiaLayer::OA.as_str(), "oa");
        assert_eq!(ArcadiaLayer::SA.as_str(), "sa");
        assert_eq!(ArcadiaLayer::LA.as_str(), "la");
        assert_eq!(ArcadiaLayer::PA.as_str(), "pa");
        assert_eq!(ArcadiaLayer::EPBS.as_str(), "epbs");
    }

    #[test]
    fn test_context_manager() {
        let manager = ContextManager::new();

        // Vérifier que le contexte racine contient les préfixes standards
        let root = manager.root_context();
        assert!(root.has_term("arcadia"));
        assert!(root.has_term("rdf"));
        assert!(root.has_term("rdfs"));
    }

    #[test]
    fn test_merged_context() {
        let mut manager = ContextManager::new();

        let mut oa_context = ArcadiaContext::with_vocab(ArcadiaLayer::OA.base_iri());
        oa_context.add_simple_mapping("OperationalActivity", "oa:OperationalActivity");

        manager.contexts.insert(ArcadiaLayer::OA, oa_context);

        let merged = manager.get_merged_context(ArcadiaLayer::OA);

        // Doit contenir les termes du contexte racine et du contexte OA
        assert!(merged.has_term("arcadia"));
        assert!(merged.has_term("OperationalActivity"));
    }
}
