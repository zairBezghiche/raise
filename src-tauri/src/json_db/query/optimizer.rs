//! Optimiseur de requêtes pour améliorer les performances
//!
//! Ce module contient la logique d'optimisation des requêtes :
//! - Réorganisation des conditions de filtre
//! - Détection des index applicables
//! - Simplification des filtres redondants
//! - Prédiction de la cardinalité

use anyhow::Result;

use super::{ComparisonOperator, Condition, FilterOperator, Query, QueryFilter};

/// Optimiseur de requêtes
#[derive(Debug, Default)]
pub struct QueryOptimizer {
    // Configuration de l'optimiseur
    config: OptimizerConfig,
}

/// Configuration de l'optimiseur
#[derive(Debug, Clone)]
pub struct OptimizerConfig {
    /// Activer la réorganisation des conditions
    pub reorder_conditions: bool,

    /// Activer la simplification des filtres
    pub simplify_filters: bool,

    /// Activer l'utilisation des index
    pub use_indexes: bool,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            reorder_conditions: true,
            simplify_filters: true,
            use_indexes: true,
        }
    }
}

impl QueryOptimizer {
    /// Crée un nouvel optimiseur avec la configuration par défaut
    pub fn new() -> Self {
        Self {
            config: OptimizerConfig::default(),
        }
    }

    /// Crée un optimiseur avec une configuration personnalisée
    pub fn with_config(config: OptimizerConfig) -> Self {
        Self { config }
    }

    /// Optimise une requête
    pub fn optimize(&self, mut query: Query) -> Result<Query> {
        // 1. Simplifier les filtres si activé
        if self.config.simplify_filters {
            if let Some(ref mut filter) = query.filter {
                *filter = self.simplify_filter(filter.clone())?;
            }
        }

        // 2. Réorganiser les conditions si activé
        if self.config.reorder_conditions {
            if let Some(ref mut filter) = query.filter {
                *filter = self.reorder_conditions(filter.clone())?;
            }
        }

        // 3. Sélectionner les index applicables (future implémentation)
        if self.config.use_indexes {
            // TODO: Implémenter la sélection d'index
        }

        // 4. Optimiser la pagination
        query = self.optimize_pagination(query)?;

        Ok(query)
    }

    /// Simplifie un filtre en éliminant les redondances
    fn simplify_filter(&self, filter: QueryFilter) -> Result<QueryFilter> {
        let mut simplified = filter.clone();

        // Éliminer les conditions dupliquées
        simplified.conditions = self.deduplicate_conditions(&simplified.conditions);

        // Simplifier les AND avec une seule condition
        if simplified.conditions.len() == 1 && matches!(simplified.operator, FilterOperator::And) {
            // Garder tel quel mais noter que c'est déjà simplifié
        }

        // Éliminer les filtres vides
        if simplified.conditions.is_empty() {
            // Retourner un filtre qui matche tout
            simplified.conditions = vec![];
        }

        Ok(simplified)
    }

    /// Réorganise les conditions pour optimiser l'évaluation
    fn reorder_conditions(&self, mut filter: QueryFilter) -> Result<QueryFilter> {
        // Trier les conditions par sélectivité estimée (les plus sélectives d'abord)
        filter
            .conditions
            .sort_by_key(|cond| self.estimate_selectivity(cond));

        Ok(filter)
    }

    /// Estime la sélectivité d'une condition (plus petit = plus sélectif)
    fn estimate_selectivity(&self, condition: &Condition) -> u32 {
        match condition.operator {
            // Opérateurs très sélectifs (matchent peu de documents)
            ComparisonOperator::Eq => 1,
            ComparisonOperator::In => 2,
            ComparisonOperator::Matches => 3,

            // Opérateurs moyennement sélectifs
            ComparisonOperator::StartsWith => 10,
            ComparisonOperator::EndsWith => 11,
            ComparisonOperator::Contains => 12,

            // Opérateurs de comparaison numérique
            ComparisonOperator::Gt => 20,
            ComparisonOperator::Gte => 21,
            ComparisonOperator::Lt => 22,
            ComparisonOperator::Lte => 23,

            // Opérateur peu sélectif
            ComparisonOperator::Ne => 100,
        }
    }

    /// Déduplique les conditions identiques
    fn deduplicate_conditions(&self, conditions: &[Condition]) -> Vec<Condition> {
        let mut seen = Vec::new();
        let mut unique = Vec::new();

        for condition in conditions {
            let key = format!(
                "{}:{:?}:{}",
                condition.field, condition.operator, condition.value
            );

            if !seen.contains(&key) {
                seen.push(key);
                unique.push(condition.clone());
            }
        }

        unique
    }

    /// Optimise la pagination
    fn optimize_pagination(&self, mut query: Query) -> Result<Query> {
        // Si limit est très grand, le ramener à une valeur raisonnable
        const MAX_REASONABLE_LIMIT: usize = 1000;

        if let Some(limit) = query.limit {
            if limit > MAX_REASONABLE_LIMIT {
                query.limit = Some(MAX_REASONABLE_LIMIT);
            }
        }

        // Si offset est très grand sans limit, ajouter un limit par défaut
        if query.offset.is_some() && query.limit.is_none() {
            query.limit = Some(100);
        }

        Ok(query)
    }

    /// Analyse une requête pour fournir des statistiques d'optimisation
    pub fn analyze_query(&self, query: &Query) -> QueryAnalysis {
        let mut analysis = QueryAnalysis::default();

        // Analyser le filtre
        if let Some(ref filter) = query.filter {
            analysis.filter_complexity = self.calculate_filter_complexity(filter);
            analysis.estimated_selectivity = self.estimate_filter_selectivity(filter);
        }

        // Analyser le tri
        if let Some(ref sort) = query.sort {
            analysis.sort_fields_count = sort.len();
        }

        // Analyser la pagination
        analysis.has_pagination = query.limit.is_some() || query.offset.is_some();

        // Détecter les optimisations possibles
        analysis.optimization_hints = self.detect_optimization_hints(query);

        analysis
    }

    /// Calcule la complexité d'un filtre
    fn calculate_filter_complexity(&self, filter: &QueryFilter) -> usize {
        filter.conditions.len()
            + filter
                .conditions
                .iter()
                .filter(|c| matches!(c.operator, ComparisonOperator::Matches))
                .count()
                * 5 // Les regex sont plus coûteuses
    }

    /// Estime la sélectivité globale d'un filtre
    fn estimate_filter_selectivity(&self, filter: &QueryFilter) -> f64 {
        if filter.conditions.is_empty() {
            return 1.0; // Matche tout
        }

        let avg_selectivity: u32 = filter
            .conditions
            .iter()
            .map(|c| self.estimate_selectivity(c))
            .sum::<u32>()
            / filter.conditions.len() as u32;

        match filter.operator {
            FilterOperator::And => 1.0 / avg_selectivity as f64, // Plus sélectif
            FilterOperator::Or => avg_selectivity as f64 / 100.0, // Moins sélectif
            FilterOperator::Not => 0.5,                          // Moyen
        }
    }

    /// Détecte les hints d'optimisation possibles
    fn detect_optimization_hints(&self, query: &Query) -> Vec<String> {
        let mut hints = Vec::new();

        // Vérifier si un index serait utile
        if let Some(ref filter) = query.filter {
            for condition in &filter.conditions {
                if matches!(condition.operator, ComparisonOperator::Eq) {
                    hints.push(format!(
                        "Index sur '{}' améliorerait les performances",
                        condition.field
                    ));
                }
            }
        }

        // Vérifier le tri
        if let Some(ref sort) = query.sort {
            if sort.len() > 3 {
                hints.push("Tri sur plus de 3 champs peut être coûteux".to_string());
            }
        }

        // Vérifier la pagination
        if let Some(offset) = query.offset {
            if offset > 1000 {
                hints.push(
                    "Offset élevé peut être inefficace, considérer la pagination par curseur"
                        .to_string(),
                );
            }
        }

        // Vérifier l'absence de limit
        if query.limit.is_none() && query.offset.is_none() {
            hints.push("Ajouter une LIMIT pour améliorer les performances".to_string());
        }

        hints
    }
}

/// Résultat de l'analyse d'une requête
#[derive(Debug, Default, Clone)]
pub struct QueryAnalysis {
    /// Complexité du filtre (nombre de conditions)
    pub filter_complexity: usize,

    /// Sélectivité estimée (0.0 = très sélectif, 1.0 = peu sélectif)
    pub estimated_selectivity: f64,

    /// Nombre de champs de tri
    pub sort_fields_count: usize,

    /// La requête utilise-t-elle la pagination ?
    pub has_pagination: bool,

    /// Hints d'optimisation
    pub optimization_hints: Vec<String>,
}

impl QueryAnalysis {
    /// Génère un rapport d'analyse lisible
    pub fn report(&self) -> String {
        let mut report = String::new();

        report.push_str(&format!(
            "Complexité du filtre: {}\n",
            self.filter_complexity
        ));
        report.push_str(&format!(
            "Sélectivité estimée: {:.2}%\n",
            self.estimated_selectivity * 100.0
        ));
        report.push_str(&format!("Champs de tri: {}\n", self.sort_fields_count));
        report.push_str(&format!("Pagination: {}\n", self.has_pagination));

        if !self.optimization_hints.is_empty() {
            report.push_str("\nHints d'optimisation:\n");
            for hint in &self.optimization_hints {
                report.push_str(&format!("  - {}\n", hint));
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json_db::query::{Condition, FilterOperator, Query, QueryFilter};
    use serde_json::json;

    #[test]
    fn test_optimize_simple_query() {
        let optimizer = QueryOptimizer::new();
        let query = Query::new("users").filter(QueryFilter {
            operator: FilterOperator::And,
            conditions: vec![
                Condition {
                    field: "age".to_string(),
                    operator: ComparisonOperator::Gt,
                    value: json!(18),
                },
                Condition {
                    field: "status".to_string(),
                    operator: ComparisonOperator::Eq,
                    value: json!("active"),
                },
            ],
        });

        let optimized = optimizer.optimize(query).unwrap();

        // Le condition Eq (plus sélective) devrait être première
        assert_eq!(optimized.filter.unwrap().conditions[0].field, "status");
    }

    #[test]
    fn test_deduplicate_conditions() {
        let optimizer = QueryOptimizer::new();
        let conditions = vec![
            Condition {
                field: "name".to_string(),
                operator: ComparisonOperator::Eq,
                value: json!("Alice"),
            },
            Condition {
                field: "name".to_string(),
                operator: ComparisonOperator::Eq,
                value: json!("Alice"),
            },
        ];

        let unique = optimizer.deduplicate_conditions(&conditions);
        assert_eq!(unique.len(), 1);
    }

    #[test]
    fn test_analyze_query() {
        let optimizer = QueryOptimizer::new();
        let query = Query::new("users")
            .filter(QueryFilter {
                operator: FilterOperator::And,
                conditions: vec![Condition {
                    field: "status".to_string(),
                    operator: ComparisonOperator::Eq,
                    value: json!("active"),
                }],
            })
            .limit(10);

        let analysis = optimizer.analyze_query(&query);

        assert_eq!(analysis.filter_complexity, 1);
        assert!(analysis.has_pagination);
    }

    #[test]
    fn test_optimize_pagination() {
        let optimizer = QueryOptimizer::new();
        let query = Query::new("users").limit(10000);

        let optimized = optimizer.optimize(query).unwrap();

        // La limite devrait être ramenée à 1000
        assert_eq!(optimized.limit, Some(1000));
    }
}
