//! Gestion des contextes JSON-LD pour données liées

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

pub mod context;
pub mod processor;
pub mod vocabulary;

///! Traitement des données JSON-LD pour Arcadia
///!
///! Ce module fournit des fonctions pour :
///! - Expansion : convertir JSON-LD compact en forme étendue
///! - Compaction : convertir forme étendue en JSON-LD compact
///! - Normalisation : produire des graphes RDF canoniques
///! - Validation : vérifier la conformité avec les schémas
// Import depuis le module parent (super = jsonld)
use self::context::{ArcadiaContext, ArcadiaLayer, ContextManager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonLdContext {
    #[serde(rename = "@context")]
    pub context: ContextDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContextDefinition {
    Simple(String),
    Object(HashMap<String, ContextValue>),
    Array(Vec<ContextDefinition>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContextValue {
    Simple(String),
    Expanded {
        #[serde(rename = "@id")]
        id: String,
        #[serde(rename = "@type")]
        type_: Option<String>,
        #[serde(rename = "@container")]
        container: Option<String>,
    },
}
use anyhow::{anyhow, Result};

/// Processeur JSON-LD pour les données Arcadia
#[derive(Debug, Clone)]
pub struct JsonLdProcessor {
    /// Gestionnaire de contextes
    context_manager: ContextManager,
}

impl JsonLdProcessor {
    /// Crée un nouveau processeur avec les contextes par défaut
    pub fn new() -> Self {
        Self {
            context_manager: ContextManager::new(),
        }
    }

    /// Crée un processeur avec un gestionnaire de contextes personnalisé
    pub fn with_context_manager(context_manager: ContextManager) -> Self {
        Self { context_manager }
    }

    /// Récupère le gestionnaire de contextes
    pub fn context_manager(&self) -> &ContextManager {
        &self.context_manager
    }

    /// Récupère le gestionnaire de contextes (mutable)
    pub fn context_manager_mut(&mut self) -> &mut ContextManager {
        &mut self.context_manager
    }

    /// Expanse un document JSON-LD compact en forme étendue
    pub fn expand(&self, document: &Value, layer: ArcadiaLayer) -> Result<Value> {
        let context = self.context_manager.get_merged_context(layer);

        match document {
            Value::Object(obj) => self.expand_object(obj, &context),
            Value::Array(arr) => {
                let expanded: Result<Vec<Value>> =
                    arr.iter().map(|item| self.expand(item, layer)).collect();
                Ok(Value::Array(expanded?))
            }
            _ => Ok(document.clone()),
        }
    }

    /// Expanse un objet JSON
    fn expand_object(&self, obj: &Map<String, Value>, context: &ArcadiaContext) -> Result<Value> {
        let mut expanded = Map::new();

        for (key, value) in obj {
            // Ignorer les mots-clés de contexte
            if key == "@context" {
                continue;
            }

            // Traiter les mots-clés JSON-LD
            if key.starts_with('@') {
                expanded.insert(key.clone(), value.clone());
                continue;
            }

            // Résoudre le terme dans le contexte
            let expanded_key = context.resolve_term(key).unwrap_or_else(|| key.clone());

            // Expanser récursivement la valeur
            let expanded_value = match value {
                Value::Object(inner) => self.expand_object(inner, context)?,
                Value::Array(arr) => {
                    let expanded_arr: Result<Vec<Value>> = arr
                        .iter()
                        .map(|item| match item {
                            Value::Object(inner) => self.expand_object(inner, context),
                            _ => Ok(item.clone()),
                        })
                        .collect();
                    Value::Array(expanded_arr?)
                }
                _ => value.clone(),
            };

            expanded.insert(expanded_key, expanded_value);
        }

        Ok(Value::Object(expanded))
    }

    /// Compacte un document JSON-LD expansé
    pub fn compact(&self, document: &Value, layer: ArcadiaLayer) -> Result<Value> {
        let context = self.context_manager.get_merged_context(layer);

        match document {
            Value::Object(obj) => self.compact_object(obj, &context),
            Value::Array(arr) => {
                let compacted: Result<Vec<Value>> =
                    arr.iter().map(|item| self.compact(item, layer)).collect();
                Ok(Value::Array(compacted?))
            }
            _ => Ok(document.clone()),
        }
    }

    /// Compacte un objet JSON
    fn compact_object(&self, obj: &Map<String, Value>, context: &ArcadiaContext) -> Result<Value> {
        let mut compacted = Map::new();

        // Créer un mapping inversé IRI -> terme
        let inverse_context: HashMap<String, String> = context
            .mappings
            .iter()
            .filter_map(|(term, ctx_value)| match ctx_value {
                ContextValue::Simple(iri) => Some((iri.clone(), term.clone())),
                ContextValue::Expanded { id, .. } => Some((id.clone(), term.clone())),
            })
            .collect();

        for (key, value) in obj {
            // Préserver les mots-clés JSON-LD
            if key.starts_with('@') {
                compacted.insert(key.clone(), value.clone());
                continue;
            }

            // Trouver le terme compact
            let compact_key = inverse_context
                .get(key)
                .cloned()
                .unwrap_or_else(|| key.clone());

            // Compacter récursivement la valeur
            let compacted_value = match value {
                Value::Object(inner) => self.compact_object(inner, context)?,
                Value::Array(arr) => {
                    let compacted_arr: Result<Vec<Value>> = arr
                        .iter()
                        .map(|item| match item {
                            Value::Object(inner) => self.compact_object(inner, context),
                            _ => Ok(item.clone()),
                        })
                        .collect();
                    Value::Array(compacted_arr?)
                }
                _ => value.clone(),
            };

            compacted.insert(compact_key, compacted_value);
        }

        Ok(Value::Object(compacted))
    }

    /// Ajoute un contexte à un document JSON-LD
    pub fn add_context(&self, document: &mut Value, layer: ArcadiaLayer) -> Result<()> {
        if let Value::Object(obj) = document {
            let context_iri = format!(
                "https://genaptitude.io/ontology/arcadia/{}.jsonld",
                layer.as_str()
            );
            obj.insert("@context".to_string(), Value::String(context_iri));
            Ok(())
        } else {
            Err(anyhow!("Document must be a JSON object"))
        }
    }

    /// Valide qu'un document contient les champs requis pour un type donné
    pub fn validate_required_fields(
        &self,
        document: &Value,
        required_fields: &[&str],
    ) -> Result<()> {
        let obj = document
            .as_object()
            .ok_or_else(|| anyhow!("Document must be a JSON object"))?;

        for field in required_fields {
            if !obj.contains_key(*field) {
                return Err(anyhow!("Missing required field: {}", field));
            }
        }

        Ok(())
    }

    /// Extrait l'ID d'un document JSON-LD
    pub fn get_id(&self, document: &Value) -> Option<String> {
        document
            .get("@id")
            .or_else(|| document.get("id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Extrait le type d'un document JSON-LD
    pub fn get_type(&self, document: &Value) -> Option<String> {
        document
            .get("@type")
            .or_else(|| document.get("type"))
            .and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                Value::Array(arr) => arr.first().and_then(|v| v.as_str()).map(|s| s.to_string()),
                _ => None,
            })
    }

    /// Crée un graphe RDF à partir d'un document JSON-LD
    pub fn to_rdf_graph(&self, document: &Value, layer: ArcadiaLayer) -> Result<RdfGraph> {
        let expanded = self.expand(document, layer)?;
        let mut graph = RdfGraph::new();

        self.extract_triples(&expanded, &mut graph)?;

        Ok(graph)
    }

    /// Extrait les triplets RDF d'un document expansé
    fn extract_triples(&self, document: &Value, graph: &mut RdfGraph) -> Result<()> {
        match document {
            Value::Object(obj) => {
                let subject = self
                    .get_id(document)
                    .unwrap_or_else(|| format!("_:b{}", graph.blank_node_counter()));

                for (predicate, value) in obj {
                    if predicate.starts_with('@') {
                        continue;
                    }

                    match value {
                        Value::Object(_) => {
                            let object = self
                                .get_id(value)
                                .unwrap_or_else(|| format!("_:b{}", graph.blank_node_counter()));
                            graph.add_triple(
                                subject.clone(),
                                predicate.clone(),
                                RdfNode::IRI(object),
                            );
                            self.extract_triples(value, graph)?;
                        }
                        Value::Array(arr) => {
                            for item in arr {
                                if let Value::Object(_) = item {
                                    let object = self.get_id(item).unwrap_or_else(|| {
                                        format!("_:b{}", graph.blank_node_counter())
                                    });
                                    graph.add_triple(
                                        subject.clone(),
                                        predicate.clone(),
                                        RdfNode::IRI(object),
                                    );
                                    self.extract_triples(item, graph)?;
                                } else {
                                    graph.add_triple(
                                        subject.clone(),
                                        predicate.clone(),
                                        RdfNode::Literal(item.to_string()),
                                    );
                                }
                            }
                        }
                        Value::String(s) => {
                            graph.add_triple(
                                subject.clone(),
                                predicate.clone(),
                                RdfNode::Literal(s.clone()),
                            );
                        }
                        Value::Number(n) => {
                            graph.add_triple(
                                subject.clone(),
                                predicate.clone(),
                                RdfNode::Literal(n.to_string()),
                            );
                        }
                        Value::Bool(b) => {
                            graph.add_triple(
                                subject.clone(),
                                predicate.clone(),
                                RdfNode::Literal(b.to_string()),
                            );
                        }
                        _ => {}
                    }
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    self.extract_triples(item, graph)?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl Default for JsonLdProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Représente un nœud RDF
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum RdfNode {
    /// IRI (Internationalized Resource Identifier)
    IRI(String),
    /// Literal (valeur textuelle)
    Literal(String),
    /// Blank node (nœud anonyme)
    BlankNode(String),
}

/// Représente un triplet RDF (sujet, prédicat, objet)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RdfTriple {
    pub subject: String,
    pub predicate: String,
    pub object: RdfNode,
}

/// Représente un graphe RDF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdfGraph {
    triples: Vec<RdfTriple>,
    blank_counter: usize,
}

impl RdfGraph {
    /// Crée un nouveau graphe vide
    pub fn new() -> Self {
        Self {
            triples: Vec::new(),
            blank_counter: 0,
        }
    }

    /// Ajoute un triplet au graphe
    pub fn add_triple(&mut self, subject: String, predicate: String, object: RdfNode) {
        self.triples.push(RdfTriple {
            subject,
            predicate,
            object,
        });
    }

    /// Récupère tous les triplets
    pub fn triples(&self) -> &[RdfTriple] {
        &self.triples
    }

    /// Génère un nouveau compteur de blank node
    pub fn blank_node_counter(&mut self) -> usize {
        let counter = self.blank_counter;
        self.blank_counter += 1;
        counter
    }

    /// Récupère tous les sujets uniques
    pub fn subjects(&self) -> Vec<&String> {
        let mut subjects: Vec<&String> = self.triples.iter().map(|t| &t.subject).collect();
        subjects.sort();
        subjects.dedup();
        subjects
    }

    /// Récupère tous les prédicats uniques
    pub fn predicates(&self) -> Vec<&String> {
        let mut predicates: Vec<&String> = self.triples.iter().map(|t| &t.predicate).collect();
        predicates.sort();
        predicates.dedup();
        predicates
    }

    /// Filtre les triplets par sujet
    pub fn triples_with_subject(&self, subject: &str) -> Vec<&RdfTriple> {
        self.triples
            .iter()
            .filter(|t| t.subject == subject)
            .collect()
    }

    /// Filtre les triplets par prédicat
    pub fn triples_with_predicate(&self, predicate: &str) -> Vec<&RdfTriple> {
        self.triples
            .iter()
            .filter(|t| t.predicate == predicate)
            .collect()
    }

    /// Exporte le graphe au format N-Triples
    pub fn to_ntriples(&self) -> String {
        self.triples
            .iter()
            .map(|triple| {
                let object = match &triple.object {
                    RdfNode::IRI(iri) => format!("<{}>", iri),
                    RdfNode::Literal(lit) => format!("\"{}\"", lit),
                    RdfNode::BlankNode(id) => id.clone(),
                };
                format!("<{}> <{}> {} .", triple.subject, triple.predicate, object)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for RdfGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_processor_creation() {
        let processor = JsonLdProcessor::new();
        assert!(processor
            .context_manager()
            .root_context()
            .has_term("arcadia"));
    }

    #[test]
    fn test_get_id() {
        let processor = JsonLdProcessor::new();

        let doc = json!({
            "@id": "http://example.org/activity-1",
            "name": "Test"
        });

        assert_eq!(
            processor.get_id(&doc),
            Some("http://example.org/activity-1".to_string())
        );
    }

    #[test]
    fn test_get_type() {
        let processor = JsonLdProcessor::new();

        let doc = json!({
            "@type": "OperationalActivity",
            "name": "Test"
        });

        assert_eq!(
            processor.get_type(&doc),
            Some("OperationalActivity".to_string())
        );
    }

    #[test]
    fn test_validate_required_fields() {
        let processor = JsonLdProcessor::new();

        let doc = json!({
            "@id": "test",
            "name": "Test Activity"
        });

        assert!(processor
            .validate_required_fields(&doc, &["@id", "name"])
            .is_ok());
        assert!(processor
            .validate_required_fields(&doc, &["@id", "name", "description"])
            .is_err());
    }

    #[test]
    fn test_rdf_graph() {
        let mut graph = RdfGraph::new();

        graph.add_triple(
            "http://example.org/activity-1".to_string(),
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),
            RdfNode::IRI("http://example.org/OperationalActivity".to_string()),
        );

        graph.add_triple(
            "http://example.org/activity-1".to_string(),
            "http://www.w3.org/2004/02/skos/core#prefLabel".to_string(),
            RdfNode::Literal("Test Activity".to_string()),
        );

        assert_eq!(graph.triples().len(), 2);
        assert_eq!(graph.subjects().len(), 1);
    }

    #[test]
    fn test_ntriples_export() {
        let mut graph = RdfGraph::new();

        graph.add_triple(
            "http://example.org/s".to_string(),
            "http://example.org/p".to_string(),
            RdfNode::Literal("Test".to_string()),
        );

        let ntriples = graph.to_ntriples();
        assert!(ntriples.contains("<http://example.org/s>"));
        assert!(ntriples.contains("<http://example.org/p>"));
        assert!(ntriples.contains("\"Test\""));
    }
}
