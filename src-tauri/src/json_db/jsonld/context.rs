// FICHIER : src-tauri/src/json_db/jsonld/context.rs

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use super::{vocabulary::VocabularyRegistry, ContextValue};

/// Enumération des couches Arcadia
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArcadiaLayer {
    OA,
    SA,
    LA,
    PA,
    EPBS,
}

impl ArcadiaLayer {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OA => "oa",
            Self::SA => "sa",
            Self::LA => "la",
            Self::PA => "pa",
            Self::EPBS => "epbs",
        }
    }
}

/// Représente un contexte JSON-LD complet avec métadonnées
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcadiaContext {
    #[serde(rename = "@version", skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    #[serde(rename = "@vocab", skip_serializing_if = "Option::is_none")]
    pub vocab: Option<String>,

    #[serde(flatten)]
    pub mappings: HashMap<String, ContextValue>,
}

impl Default for ArcadiaContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ArcadiaContext {
    pub fn new() -> Self {
        Self {
            version: Some("1.1".to_string()),
            vocab: None,
            mappings: HashMap::new(),
        }
    }

    pub fn add_simple_mapping(&mut self, term: &str, iri: &str) {
        self.mappings
            .insert(term.to_string(), ContextValue::Simple(iri.to_string()));
    }

    pub fn has_term(&self, term: &str) -> bool {
        self.mappings.contains_key(term)
    }
}

/// Gestionnaire principal de contexte pour le processeur
#[derive(Debug, Clone)]
pub struct ContextManager {
    /// Contextes spécifiques par couche (Legacy support)
    pub contexts: HashMap<ArcadiaLayer, ArcadiaContext>,
    /// Table de résolution active (Prefix -> IRI)
    // CORRECTION : Rendu public pour les tests
    pub active_namespaces: HashMap<String, String>,
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
            active_namespaces: VocabularyRegistry::get_default_prefixes(),
        }
    }

    /// Charge un contexte depuis un document JSON-LD (@context)
    pub fn load_from_doc(&mut self, doc: &Value) -> Result<()> {
        if let Some(ctx) = doc.get("@context") {
            self.parse_context_block(ctx)?;
        }
        Ok(())
    }

    fn parse_context_block(&mut self, ctx: &Value) -> Result<()> {
        match ctx {
            Value::Object(map) => {
                for (key, val) in map {
                    if let Value::String(uri) = val {
                        self.active_namespaces.insert(key.clone(), uri.clone());
                    }
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    self.parse_context_block(item)?;
                }
            }
            _ => {} // Ignore string refs
        }
        Ok(())
    }

    /// EXPANSION : Transforme "oa:Actor" en "http://.../oa#Actor"
    pub fn expand_term(&self, term: &str) -> String {
        if VocabularyRegistry::is_iri(term) || term.starts_with('@') {
            return term.to_string();
        }

        if let Some((prefix, suffix)) = term.split_once(':') {
            if let Some(base) = self.active_namespaces.get(prefix) {
                return format!("{}{}", base, suffix);
            }
        }

        if let Some(uri) = self.active_namespaces.get(term) {
            return uri.clone();
        }

        term.to_string()
    }

    /// COMPACTION : Transforme "http://.../oa#Actor" en "oa:Actor"
    pub fn compact_iri(&self, iri: &str) -> String {
        for (prefix, base) in &self.active_namespaces {
            if iri.starts_with(base) {
                let suffix = &iri[base.len()..];
                if suffix.is_empty() {
                    return prefix.clone();
                }
                return format!("{}:{}", prefix, suffix);
            }
        }
        iri.to_string()
    }
}
