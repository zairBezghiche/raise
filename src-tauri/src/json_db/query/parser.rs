//! Parsing de requêtes pour le moteur JSON
//!
//! Ce module fournit des fonctions pour parser des requêtes depuis :
//! - Spécifications de tri (field:asc, field:desc)
//! - Filtres JSON
//! - Requêtes en pseudo-SQL

use anyhow::{bail, Result};
use serde_json::Value;

use super::{
    ComparisonOperator, Condition, FilterOperator, Projection, Query, QueryFilter, SortField,
    SortOrder,
};

// ============================================================================
// PARSING DES SPÉCIFICATIONS DE TRI
// ============================================================================

/// Parse une liste de specs de tri en Vec<SortField>.
///
/// Formes acceptées :
///   - "field"           → field ASC
///   - "field:asc"       → field ASC
///   - "field:desc"      → field DESC
///   - "+field"          → field ASC
///   - "-field"          → field DESC
///
/// # Exemples
///
/// ```rust
///

pub fn parse_sort_specs(specs: &[String]) -> Result<Vec<SortField>> {
    let mut out = Vec::new();

    for spec in specs {
        let sort_field = parse_single_sort_spec(spec)?;
        out.push(sort_field);
    }

    Ok(out)
}

/// Parse une seule spécification de tri
fn parse_single_sort_spec(spec: &str) -> Result<SortField> {
    let spec = spec.trim();

    // Vérifier les préfixes + et -
    if let Some(field) = spec.strip_prefix('+') {
        return Ok(SortField {
            field: field.trim().to_string(),
            order: SortOrder::Asc,
        });
    }

    if let Some(field) = spec.strip_prefix('-') {
        return Ok(SortField {
            field: field.trim().to_string(),
            order: SortOrder::Desc,
        });
    }

    // Format "field:asc" ou "field:desc"
    let (field, order_str_opt) = if let Some((f, o)) = spec.split_once(':') {
        (f.trim(), Some(o.trim()))
    } else {
        (spec.trim(), None)
    };

    if field.is_empty() {
        bail!("Spécification de tri invalide: '{spec}'");
    }

    let order_str = order_str_opt.unwrap_or("asc").to_lowercase();
    let order = match order_str.as_str() {
        "asc" | "ascending" => SortOrder::Asc,
        "desc" | "descending" => SortOrder::Desc,
        other => {
            bail!("Sens de tri invalide dans '{spec}': utiliser 'asc' ou 'desc', pas '{other}'")
        }
    };

    Ok(SortField {
        field: field.to_string(),
        order,
    })
}

// ============================================================================
// PARSING DES FILTRES
// ============================================================================

/// Parse un filtre depuis un objet JSON
///
/// Format attendu :
/// ```json
/// {
///   "operator": "and",
///   "conditions": [
///     {
///       "field": "age",
///       "operator": "gt",
///       "value": 18
///     }
///   ]
/// }
/// ```
pub fn parse_filter_from_json(value: &Value) -> Result<QueryFilter> {
    let obj = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Filter must be a JSON object"))?;

    let operator_str = obj
        .get("operator")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'operator' field"))?;

    let operator = parse_filter_operator(operator_str)?;

    let conditions_array = obj
        .get("conditions")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Missing 'conditions' array"))?;

    let mut conditions = Vec::new();
    for condition_value in conditions_array {
        conditions.push(parse_condition_from_json(condition_value)?);
    }

    Ok(QueryFilter {
        operator,
        conditions,
    })
}

/// Parse une condition depuis un objet JSON
fn parse_condition_from_json(value: &Value) -> Result<Condition> {
    let obj = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Condition must be a JSON object"))?;

    let field = obj
        .get("field")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'field' in condition"))?
        .to_string();

    let operator_str = obj
        .get("operator")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'operator' in condition"))?;

    let operator = parse_comparison_operator(operator_str)?;

    let condition_value = obj
        .get("value")
        .ok_or_else(|| anyhow::anyhow!("Missing 'value' in condition"))?
        .clone();

    Ok(Condition {
        field,
        operator,
        value: condition_value,
    })
}

/// Parse un opérateur de filtre depuis une chaîne
fn parse_filter_operator(s: &str) -> Result<FilterOperator> {
    match s.to_lowercase().as_str() {
        "and" | "&&" => Ok(FilterOperator::And),
        "or" | "||" => Ok(FilterOperator::Or),
        "not" | "!" => Ok(FilterOperator::Not),
        other => bail!("Unknown filter operator: '{}'", other),
    }
}

/// Parse un opérateur de comparaison depuis une chaîne
fn parse_comparison_operator(s: &str) -> Result<ComparisonOperator> {
    match s.to_lowercase().as_str() {
        "eq" | "=" | "==" => Ok(ComparisonOperator::Eq),
        "ne" | "!=" | "<>" => Ok(ComparisonOperator::Ne),
        "gt" | ">" => Ok(ComparisonOperator::Gt),
        "gte" | ">=" => Ok(ComparisonOperator::Gte),
        "lt" | "<" => Ok(ComparisonOperator::Lt),
        "lte" | "<=" => Ok(ComparisonOperator::Lte),
        "in" => Ok(ComparisonOperator::In),
        "contains" | "like" => Ok(ComparisonOperator::Contains),
        "startswith" | "starts_with" => Ok(ComparisonOperator::StartsWith),
        "endswith" | "ends_with" => Ok(ComparisonOperator::EndsWith),
        "matches" | "regex" => Ok(ComparisonOperator::Matches),
        other => bail!("Unknown comparison operator: '{}'", other),
    }
}

// ============================================================================
// PARSING DES PROJECTIONS
// ============================================================================

/// Parse une projection depuis une liste de champs
///
/// Si la liste commence par "-", c'est une exclusion, sinon c'est une inclusion
///
/// # Exemples
///
/// - `["name", "age"]` → Include name et age
/// - `["-password", "-secret"]` → Exclude password et secret
pub fn parse_projection(fields: &[String]) -> Result<Projection> {
    if fields.is_empty() {
        bail!("Projection fields cannot be empty");
    }

    let is_exclusion = fields[0].starts_with('-');

    let cleaned_fields: Vec<String> = fields
        .iter()
        .map(|f| {
            if let Some(stripped) = f.strip_prefix('-') {
                stripped.to_string()
            } else if let Some(stripped) = f.strip_prefix('+') {
                stripped.to_string()
            } else {
                f.clone()
            }
        })
        .collect();

    if is_exclusion {
        Ok(Projection::Exclude(cleaned_fields))
    } else {
        Ok(Projection::Include(cleaned_fields))
    }
}

// ============================================================================
// CONSTRUCTION DE REQUÊTES
// ============================================================================

/// Builder fluide pour construire des requêtes complexes
pub struct QueryBuilder {
    query: Query,
}

impl QueryBuilder {
    /// Crée un nouveau builder pour une collection
    pub fn new(collection: impl Into<String>) -> Self {
        Self {
            query: Query::new(collection),
        }
    }

    /// Ajoute une condition WHERE avec AND
    pub fn where_eq(mut self, field: impl Into<String>, value: Value) -> Self {
        let condition = Condition::eq(field, value);
        self.add_condition(FilterOperator::And, condition);
        self
    }

    /// Ajoute une condition WHERE avec OR
    pub fn or_where_eq(mut self, field: impl Into<String>, value: Value) -> Self {
        let condition = Condition::eq(field, value);
        self.add_condition(FilterOperator::Or, condition);
        self
    }

    /// Ajoute un ORDER BY
    pub fn order_by(mut self, field: impl Into<String>, order: SortOrder) -> Self {
        let sort_field = SortField {
            field: field.into(),
            order,
        };

        if let Some(ref mut sort) = self.query.sort {
            sort.push(sort_field);
        } else {
            self.query.sort = Some(vec![sort_field]);
        }

        self
    }

    /// Ajoute un LIMIT
    pub fn limit(mut self, limit: usize) -> Self {
        self.query.limit = Some(limit);
        self
    }

    /// Ajoute un OFFSET
    pub fn offset(mut self, offset: usize) -> Self {
        self.query.offset = Some(offset);
        self
    }

    /// Ajoute une projection
    pub fn select(mut self, fields: Vec<String>) -> Result<Self> {
        self.query.projection = Some(parse_projection(&fields)?);
        Ok(self)
    }

    /// Construit la requête finale
    pub fn build(self) -> Query {
        self.query
    }

    // Helper privé pour ajouter une condition
    fn add_condition(&mut self, operator: FilterOperator, condition: Condition) {
        if let Some(ref mut filter) = self.query.filter {
            // Si l'opérateur est le même, ajouter à la liste existante
            if std::mem::discriminant(&filter.operator) == std::mem::discriminant(&operator) {
                filter.conditions.push(condition);
            } else {
                // Sinon, créer un nouveau filtre avec l'ancien comme condition
                let _old_filter = filter.clone();
                *filter = QueryFilter {
                    operator,
                    conditions: vec![condition],
                };
                // TODO: Gérer le cas où on mélange AND et OR
            }
        } else {
            self.query.filter = Some(QueryFilter {
                operator,
                conditions: vec![condition],
            });
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_sort_specs_basic() {
        let specs = vec!["name".to_string(), "age:desc".to_string()];
        let result = parse_sort_specs(&specs).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].field, "name");
        assert!(matches!(result[0].order, SortOrder::Asc));
        assert_eq!(result[1].field, "age");
        assert!(matches!(result[1].order, SortOrder::Desc));
    }

    #[test]
    fn test_parse_sort_specs_with_prefix() {
        let specs = vec!["+name".to_string(), "-age".to_string()];
        let result = parse_sort_specs(&specs).unwrap();

        assert_eq!(result.len(), 2);
        assert!(matches!(result[0].order, SortOrder::Asc));
        assert!(matches!(result[1].order, SortOrder::Desc));
    }

    #[test]
    fn test_parse_filter_from_json() {
        let json = json!({
            "operator": "and",
            "conditions": [
                {
                    "field": "age",
                    "operator": "gt",
                    "value": 18
                }
            ]
        });

        let filter = parse_filter_from_json(&json).unwrap();
        assert!(matches!(filter.operator, FilterOperator::And));
        assert_eq!(filter.conditions.len(), 1);
    }

    #[test]
    fn test_parse_projection_include() {
        let fields = vec!["name".to_string(), "age".to_string()];
        let projection = parse_projection(&fields).unwrap();

        assert!(matches!(projection, Projection::Include(_)));
    }

    #[test]
    fn test_parse_projection_exclude() {
        let fields = vec!["-password".to_string(), "-secret".to_string()];
        let projection = parse_projection(&fields).unwrap();

        assert!(matches!(projection, Projection::Exclude(_)));
    }

    #[test]
    fn test_query_builder() {
        let query = QueryBuilder::new("users")
            .where_eq("status", json!("active"))
            .where_eq("age", json!(18))
            .order_by("name", SortOrder::Asc)
            .limit(10)
            .build();

        assert_eq!(query.collection, "users");
        assert!(query.filter.is_some());
        assert_eq!(query.limit, Some(10));
    }

    #[test]
    fn test_parse_comparison_operators() {
        assert!(matches!(
            parse_comparison_operator("=").unwrap(),
            ComparisonOperator::Eq
        ));
        assert!(matches!(
            parse_comparison_operator("!=").unwrap(),
            ComparisonOperator::Ne
        ));
        assert!(matches!(
            parse_comparison_operator(">").unwrap(),
            ComparisonOperator::Gt
        ));
    }
}
