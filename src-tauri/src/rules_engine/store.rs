use crate::rules_engine::ast::Rule;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct RuleStore {
    /// Map: Collection Name -> Liste de règles
    rules_by_collection: HashMap<String, Vec<Rule>>,
    /// Graphe de dépendance inversé : "item.price" -> ["rule_id_1", "rule_id_2"]
    /// (Si "item.price" change, quelles règles relancer ?)
    dependency_graph: HashMap<String, Vec<String>>,
}

// CORRECTION 1 : Implémentation de Default
impl Default for RuleStore {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleStore {
    pub fn new() -> Self {
        Self {
            rules_by_collection: HashMap::new(),
            dependency_graph: HashMap::new(),
        }
    }

    pub fn register_rule(&mut self, collection: &str, rule: Rule) {
        // Indexer la règle par collection
        self.rules_by_collection
            .entry(collection.to_string())
            .or_default() // CORRECTION 2 : or_default() au lieu de or_insert_with(Vec::new)
            .push(rule.clone());

        // Analyser les dépendances pour le graphe
        let deps = crate::rules_engine::Analyzer::get_dependencies(&rule.expr);

        // On indexe par : Collection + Champ (ex: "users.age")
        // Note: Ici on simplifie en indexant juste le champ car le scope est souvent la collection courante
        for dep in deps {
            self.dependency_graph
                .entry(dep)
                .or_default() // CORRECTION 3 : or_default() au lieu de or_insert_with(Vec::new)
                .push(rule.id.clone());
        }
    }

    /// Récupère les règles à exécuter en fonction des champs modifiés
    pub fn get_impacted_rules(
        &self,
        collection: &str,
        changed_fields: &HashSet<String>,
    ) -> Vec<Rule> {
        let mut impacted_rules = Vec::new();

        if let Some(collection_rules) = self.rules_by_collection.get(collection) {
            for rule in collection_rules {
                // Si la règle dépend d'un des champs modifiés, on l'ajoute
                // Optimisation: On pourrait utiliser le dependency_graph pour faire un lookup direct
                // Mais pour l'instant, on itère et on vérifie l'intersection pour être sûr
                let rule_deps = crate::rules_engine::Analyzer::get_dependencies(&rule.expr);
                if !rule_deps.is_disjoint(changed_fields) {
                    impacted_rules.push(rule.clone());
                }
            }
        }

        impacted_rules
    }
}
