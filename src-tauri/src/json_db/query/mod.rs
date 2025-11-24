//! Moteur de requêtes JSON pour GenAptitude
//!
//! Ce module fournit un système de requêtes similaire à SQL mais optimisé pour JSON.
//! Il supporte :
//! - Filtrage avec opérateurs logiques (AND, OR, NOT)
//! - Tri multi-champs (ASC, DESC)
//! - Pagination (LIMIT, OFFSET)
//! - Opérations CRUD (Create, Read, Update, Delete)
//! - Optimisation automatique des requêtes

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod executor;
pub mod optimizer;
pub mod parser;

// Re-export des types publics
pub use executor::QueryExecutor;
pub use optimizer::QueryOptimizer;
pub type QueryInput = Query;

// 💡 Import de CollectionsManager
use crate::json_db::collections::manager::CollectionsManager;

// ----------------------------------------------------------------------------
// MOTEUR PRINCIPAL
// ----------------------------------------------------------------------------

/// Moteur de requêtes principal (Main Query Engine)
// 💡 Correction de la durée de vie ('a) sur la structure
#[derive(Debug)]
pub struct QueryEngine<'a> {
    executor: QueryExecutor<'a>,
    optimizer: QueryOptimizer,
}

// 💡 Correction de la durée de vie ('a) sur l'implémentation
impl<'a> QueryEngine<'a> {
    /// Crée une nouvelle instance du moteur de requêtes
    // 💡 Prend le CollectionsManager par référence
    pub fn new(manager: &'a CollectionsManager<'a>) -> Self {
        Self {
            // Le manager est passé à l'executor
            executor: QueryExecutor::new(manager),
            optimizer: QueryOptimizer::new(),
        }
    }

    /// Exécute une requête SELECT
    pub async fn execute_query(&self, query: Query) -> Result<QueryResult> {
        // 1. Optimiser la requête
        let optimized = self.optimizer.optimize(query)?;

        // 2. Exécuter la requête optimisée
        self.executor.execute(optimized).await
    }

    /// Insère des documents dans une collection
    pub async fn insert(&self, collection: &str, documents: Vec<Value>) -> Result<InsertResult> {
        self.executor.insert(collection, documents).await
    }

    /// Insert ou update (upsert) des documents
    pub async fn upsert(
        &self,
        collection: &str,
        documents: Vec<Value>,
        match_fields: Vec<String>,
    ) -> Result<UpsertResult> {
        self.executor
            .upsert(collection, documents, match_fields)
            .await
    }

    /// Met à jour des documents correspondant à un filtre
    pub async fn update(
        &self,
        collection: &str,
        filter: QueryFilter,
        updates: Value,
    ) -> Result<UpdateResult> {
        self.executor.update(collection, filter, updates).await
    }

    /// Supprime des documents correspondant à un filtre
    pub async fn delete(&self, collection: &str, filter: QueryFilter) -> Result<DeleteResult> {
        self.executor.delete(collection, filter).await
    }

    /// Liste toutes les collections disponibles
    pub async fn list_collections(&self) -> Result<Vec<String>> {
        self.executor.list_collections().await
    }

    /// Compte le nombre de documents correspondant à un filtre
    pub async fn count(&self, collection: &str, filter: Option<QueryFilter>) -> Result<u64> {
        self.executor.count(collection, filter).await
    }
}

// ============================================================================
// STRUCTURES DE REQUÊTES
// ============================================================================

/// Requête de type SELECT avec filtres, tri et pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    /// Nom de la collection à interroger
    pub collection: String,

    /// Filtre optionnel (WHERE clause)
    pub filter: Option<QueryFilter>,

    /// Tri optionnel (ORDER BY clause)
    pub sort: Option<Vec<SortField>>,

    /// Limite de résultats (LIMIT clause)
    pub limit: Option<usize>,

    /// Offset pour pagination (OFFSET clause)
    pub offset: Option<usize>,

    /// Projection : champs à inclure/exclure
    pub projection: Option<Projection>,
}

impl Query {
    /// Crée une nouvelle requête sur une collection
    pub fn new(collection: impl Into<String>) -> Self {
        Self {
            collection: collection.into(),
            filter: None,
            sort: None,
            limit: None,
            offset: None,
            projection: None,
        }
    }

    /// Ajoute un filtre WHERE
    pub fn filter(mut self, filter: QueryFilter) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Ajoute un tri ORDER BY
    pub fn sort(mut self, sort: Vec<SortField>) -> Self {
        self.sort = Some(sort);
        self
    }

    /// Ajoute une limite LIMIT
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Ajoute un offset OFFSET
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Ajoute une projection (sélection de champs)
    pub fn projection(mut self, projection: Projection) -> Self {
        self.projection = Some(projection);
        self
    }
}

/// Filtre de requête (WHERE clause)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryFilter {
    /// Opérateur logique combinant les conditions
    pub operator: FilterOperator,

    /// Conditions à évaluer
    pub conditions: Vec<Condition>,
}

impl QueryFilter {
    /// Crée un filtre AND
    pub fn and(conditions: Vec<Condition>) -> Self {
        Self {
            operator: FilterOperator::And,
            conditions,
        }
    }

    /// Crée un filtre OR
    pub fn or(conditions: Vec<Condition>) -> Self {
        Self {
            operator: FilterOperator::Or,
            conditions,
        }
    }

    /// Crée un filtre NOT
    pub fn not(conditions: Vec<Condition>) -> Self {
        Self {
            operator: FilterOperator::Not,
            conditions,
        }
    }
}

/// Opérateurs logiques pour combiner les conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FilterOperator {
    /// Toutes les conditions doivent être vraies (AND)
    And,

    /// Au moins une condition doit être vraie (OR)
    Or,

    /// Négation des conditions (NOT)
    Not,
}

/// Condition de filtrage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// Champ JSON sur lequel appliquer la condition
    pub field: String,

    /// Opérateur de comparaison
    pub operator: ComparisonOperator,

    /// Valeur à comparer
    pub value: Value,
}

impl Condition {
    /// Condition d'égalité (field = value)
    pub fn eq(field: impl Into<String>, value: Value) -> Self {
        Self {
            field: field.into(),
            operator: ComparisonOperator::Eq,
            value,
        }
    }

    /// Condition de non-égalité (field != value)
    pub fn ne(field: impl Into<String>, value: Value) -> Self {
        Self {
            field: field.into(),
            operator: ComparisonOperator::Ne,
            value,
        }
    }

    /// Condition supérieur à (field > value)
    pub fn gt(field: impl Into<String>, value: Value) -> Self {
        Self {
            field: field.into(),
            operator: ComparisonOperator::Gt,
            value,
        }
    }

    /// Condition supérieur ou égal (field >= value)
    pub fn gte(field: impl Into<String>, value: Value) -> Self {
        Self {
            field: field.into(),
            operator: ComparisonOperator::Gte,
            value,
        }
    }

    /// Condition inférieur à (field < value)
    pub fn lt(field: impl Into<String>, value: Value) -> Self {
        Self {
            field: field.into(),
            operator: ComparisonOperator::Lt,
            value,
        }
    }

    /// Condition inférieur ou égal (field <= value)
    pub fn lte(field: impl Into<String>, value: Value) -> Self {
        Self {
            field: field.into(),
            operator: ComparisonOperator::Lte,
            value,
        }
    }

    /// Condition d'appartenance (field IN [values])
    pub fn in_array(field: impl Into<String>, value: Value) -> Self {
        Self {
            field: field.into(),
            operator: ComparisonOperator::In,
            value,
        }
    }

    /// Condition de contenance (field CONTAINS value)
    pub fn contains(field: impl Into<String>, value: Value) -> Self {
        Self {
            field: field.into(),
            operator: ComparisonOperator::Contains,
            value,
        }
    }
}

/// Opérateurs de comparaison
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComparisonOperator {
    /// Égal à (=)
    Eq,

    /// Non égal à (!=)
    Ne,

    /// Supérieur à (>)
    Gt,

    /// Supérieur ou égal à (>=)
    Gte,

    /// Inférieur à (<)
    Lt,

    /// Inférieur ou égal à (<=)
    Lte,

    /// Appartient à un ensemble (IN)
    In,

    /// Contient une valeur (CONTAINS)
    Contains,

    /// Commence par (STARTS WITH)
    StartsWith,

    /// Finit par (ENDS WITH)
    EndsWith,

    /// Correspond à une regex (MATCHES)
    Matches,
}

/// Champ de tri avec ordre
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortField {
    /// Nom du champ sur lequel trier
    pub field: String,

    /// Ordre de tri (ASC ou DESC)
    pub order: SortOrder,
}

impl SortField {
    /// Crée un tri ascendant
    pub fn asc(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            order: SortOrder::Asc,
        }
    }

    /// Crée un tri descendant
    pub fn desc(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            order: SortOrder::Desc,
        }
    }
}

/// Ordre de tri
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    /// Ordre croissant (A → Z, 1 → 9)
    Asc,

    /// Ordre décroissant (Z → A, 9 → 1)
    Desc,
}

/// Projection : sélection de champs à inclure/exclure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Projection {
    /// Liste de champs à inclure
    Include(Vec<String>),

    /// Liste de champs à exclure
    Exclude(Vec<String>),
}

// ============================================================================
// RÉSULTATS DES OPÉRATIONS
// ============================================================================

/// Résultat d'une requête SELECT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// Documents correspondants
    pub documents: Vec<Value>,

    /// Nombre total de résultats (avant pagination)
    pub total_count: u64,

    /// Offset appliqué
    pub offset: usize,

    /// Limite appliquée
    pub limit: Option<usize>,
}

/// Résultat d'une insertion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertResult {
    /// Nombre de documents insérés
    pub inserted_count: u64,

    /// IDs des documents insérés
    pub inserted_ids: Vec<String>,
}

/// Résultat d'un upsert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertResult {
    /// Nombre de documents insérés
    pub inserted_count: u64,

    /// Nombre de documents mis à jour
    pub updated_count: u64,

    /// IDs affectés
    pub affected_ids: Vec<String>,
}

/// Résultat d'une mise à jour
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResult {
    /// Nombre de documents correspondants
    pub matched_count: u64,

    /// Nombre de documents modifiés
    pub modified_count: u64,
}

/// Résultat d'une suppression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteResult {
    /// Nombre de documents supprimés
    pub deleted_count: u64,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_query_builder() {
        let query = Query::new("users")
            .filter(QueryFilter::and(vec![
                Condition::eq("status", json!("active")),
                Condition::gt("age", json!(18)),
            ]))
            .sort(vec![SortField::desc("created_at")])
            .limit(10)
            .offset(0);

        assert_eq!(query.collection, "users");
        assert!(query.filter.is_some());
        assert!(query.sort.is_some());
        assert_eq!(query.limit, Some(10));
    }

    #[test]
    fn test_condition_builders() {
        let eq = Condition::eq("name", json!("Alice"));
        assert!(matches!(eq.operator, ComparisonOperator::Eq));

        let gt = Condition::gt("age", json!(18));
        assert!(matches!(gt.operator, ComparisonOperator::Gt));
    }

    #[test]
    fn test_sort_field_builders() {
        let asc = SortField::asc("name");
        assert!(matches!(asc.order, SortOrder::Asc));

        let desc = SortField::desc("created_at");
        assert!(matches!(desc.order, SortOrder::Desc));
    }
}
