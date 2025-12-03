//! Traitement des données JSON-LD pour Arcadia
//!
//! Ce module fournit des fonctions pour :
//! - Expansion / Compaction
//! - Normalisation RDF
//! - Validation

use anyhow::{anyhow, Result};
use serde_json::{Map, Value};
// use std::collections::HashMap; // SUPPRIMÉ car inutilisé

use super::context::ContextManager;

/// Représentation simple d'un nœud RDF pour l'export
#[derive(Debug, Clone)]
pub enum RdfNode {
    IRI(String),
    Literal(String),
    BlankNode(String),
}

/// Graphe RDF simplifié
#[derive(Debug, Default)]
pub struct RdfGraph {
    triples: Vec<(String, String, RdfNode)>,
}

impl RdfGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_triple(&mut self, subject: String, predicate: String, object: RdfNode) {
        self.triples.push((subject, predicate, object));
    }

    pub fn triples(&self) -> &Vec<(String, String, RdfNode)> {
        &self.triples
    }

    pub fn subjects(&self) -> Vec<String> {
        let mut subs: Vec<String> = self.triples.iter().map(|(s, _, _)| s.clone()).collect();
        subs.sort();
        subs.dedup();
        subs
    }
}

/// Processeur JSON-LD pour les données Arcadia
#[derive(Debug, Clone)]
pub struct JsonLdProcessor {
    context_manager: ContextManager,
}

impl Default for JsonLdProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonLdProcessor {
    pub fn new() -> Self {
        Self {
            context_manager: ContextManager::new(),
        }
    }

    pub fn with_context_manager(context_manager: ContextManager) -> Self {
        Self { context_manager }
    }

    pub fn with_doc_context(mut self, doc: &Value) -> Result<Self> {
        self.context_manager.load_from_doc(doc)?;
        Ok(self)
    }

    pub fn context_manager(&self) -> &ContextManager {
        &self.context_manager
    }

    // --- ALGORITHMES JSON-LD ---

    pub fn expand(&self, doc: &Value) -> Value {
        match doc {
            Value::Object(map) => {
                let mut new_map = Map::new();
                for (k, v) in map {
                    let expanded_key = self.context_manager.expand_term(k);

                    let expanded_val = if k == "@type" {
                        self.expand_value_as_iri(v)
                    } else {
                        self.expand(v)
                    };
                    new_map.insert(expanded_key, expanded_val);
                }
                Value::Object(new_map)
            }
            Value::Array(arr) => Value::Array(arr.iter().map(|v| self.expand(v)).collect()),
            _ => doc.clone(),
        }
    }

    pub fn compact(&self, doc: &Value) -> Value {
        match doc {
            Value::Object(map) => {
                let mut new_map = Map::new();
                for (k, v) in map {
                    if k == "@context" {
                        continue;
                    }

                    let compacted_key = self.context_manager.compact_iri(k);

                    let compacted_val = if k == "@type" {
                        self.compact_value_as_iri(v)
                    } else {
                        self.compact(v)
                    };
                    new_map.insert(compacted_key, compacted_val);
                }
                Value::Object(new_map)
            }
            Value::Array(arr) => Value::Array(arr.iter().map(|v| self.compact(v)).collect()),
            _ => doc.clone(),
        }
    }

    fn expand_value_as_iri(&self, val: &Value) -> Value {
        match val {
            Value::String(s) => Value::String(self.context_manager.expand_term(s)),
            Value::Array(arr) => {
                Value::Array(arr.iter().map(|v| self.expand_value_as_iri(v)).collect())
            }
            _ => val.clone(),
        }
    }

    fn compact_value_as_iri(&self, val: &Value) -> Value {
        match val {
            Value::String(s) => Value::String(self.context_manager.compact_iri(s)),
            Value::Array(arr) => {
                Value::Array(arr.iter().map(|v| self.compact_value_as_iri(v)).collect())
            }
            _ => val.clone(),
        }
    }

    // --- UTILITAIRES RDF / VALIDATION ---

    pub fn get_id(&self, doc: &Value) -> Option<String> {
        doc.get("@id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    pub fn get_type(&self, doc: &Value) -> Option<String> {
        if let Some(t) = doc.get("@type") {
            return t.as_str().map(|s| s.to_string());
        }
        if let Some(t) = doc.get("http://www.w3.org/1999/02/22-rdf-syntax-ns#type") {
            return t.as_str().map(|s| s.to_string());
        }
        None
    }

    pub fn validate_required_fields(&self, doc: &Value, required: &[&str]) -> Result<()> {
        let expanded = self.expand(doc);
        for &field in required {
            let iri = self.context_manager.expand_term(field);
            if expanded.get(&iri).is_none() {
                if doc.get(field).is_none() {
                    return Err(anyhow!("Champ requis manquant : {}", field));
                }
            }
        }
        Ok(())
    }

    pub fn to_ntriples(&self, doc: &Value) -> Result<String> {
        let expanded = self.expand(doc);
        let id = self
            .get_id(&expanded)
            .ok_or_else(|| anyhow!("Document sans @id"))?;

        let mut lines = Vec::new();

        if let Some(obj) = expanded.as_object() {
            for (pred, val) in obj {
                if pred.starts_with('@') {
                    continue;
                }

                let objects = if let Value::Array(arr) = val {
                    arr.iter().collect()
                } else {
                    vec![val]
                };

                for o in objects {
                    let obj_str = match o {
                        Value::String(s) if s.starts_with("http") => format!("<{}>", s),
                        Value::String(s) => format!("{:?}", s),
                        _ => format!("{:?}", o.to_string()),
                    };
                    lines.push(format!("<{}> <{}> {} .", id, pred, obj_str));
                }
            }
        }

        Ok(lines.join("\n"))
    }
}
