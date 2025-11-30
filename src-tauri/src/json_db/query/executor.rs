use anyhow::Result; // CORRECTION : suppression de anyhow inutile
use serde_json::Value;
use std::cmp::Ordering;

use crate::json_db::collections::manager::CollectionsManager;
use crate::json_db::query::{
    ComparisonOperator, Condition, FilterOperator, Query, QueryFilter, QueryResult, SortField,
    SortOrder,
};

pub struct QueryEngine<'a> {
    manager: &'a CollectionsManager<'a>,
}

impl<'a> QueryEngine<'a> {
    pub fn new(manager: &'a CollectionsManager<'a>) -> Self {
        Self { manager }
    }

    pub async fn execute_query(&self, query: Query) -> Result<QueryResult> {
        let mut documents = self.manager.list_all(&query.collection)?;

        if let Some(filter) = &query.filter {
            documents.retain(|doc| self.evaluate_filter(doc, filter));
        }

        if let Some(sort_fields) = &query.sort {
            documents.sort_by(|a, b| self.compare_docs(a, b, sort_fields));
        }

        let total_count = documents.len() as u64;
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(documents.len());

        let paged_docs = documents.into_iter().skip(offset).take(limit).collect();

        Ok(QueryResult {
            documents: paged_docs,
            total_count,
            offset: Some(offset), // CORRECTION: Option<usize>
            limit: Some(limit),   // CORRECTION: Option<usize>
        })
    }

    // --- Filtres ---

    fn evaluate_filter(&self, document: &Value, filter: &QueryFilter) -> bool {
        match filter.operator {
            FilterOperator::And => filter
                .conditions
                .iter()
                .all(|c| self.evaluate_condition(document, c)),
            FilterOperator::Or => filter
                .conditions
                .iter()
                .any(|c| self.evaluate_condition(document, c)),
            FilterOperator::Not => !filter
                .conditions
                .iter()
                .any(|c| self.evaluate_condition(document, c)),
        }
    }

    fn evaluate_condition(&self, document: &Value, condition: &Condition) -> bool {
        let field_value = self.get_field_value(document, &condition.field);
        match &condition.operator {
            ComparisonOperator::Eq => field_value == Some(&condition.value),
            ComparisonOperator::Ne => field_value != Some(&condition.value),
            // ... (Comparaisons numriques ok) ...
            ComparisonOperator::Gt => {
                self.compare_values(field_value, &condition.value) == Some(Ordering::Greater)
            }
            ComparisonOperator::Gte => {
                let ord = self.compare_values(field_value, &condition.value);
                ord == Some(Ordering::Greater) || ord == Some(Ordering::Equal)
            }
            ComparisonOperator::Lt => {
                self.compare_values(field_value, &condition.value) == Some(Ordering::Less)
            }
            ComparisonOperator::Lte => {
                let ord = self.compare_values(field_value, &condition.value);
                ord == Some(Ordering::Less) || ord == Some(Ordering::Equal)
            }
            ComparisonOperator::In => {
                if let (Some(val), Some(arr)) = (field_value, condition.value.as_array()) {
                    return arr.contains(val);
                }
                false
            }
            ComparisonOperator::Contains => {
                if let Some(val) = field_value {
                    if let Some(arr) = val.as_array() {
                        return arr.contains(&condition.value);
                    }
                    if let Some(s) = val.as_str() {
                        if let Some(sub) = condition.value.as_str() {
                            return s.contains(sub);
                        }
                    }
                }
                false
            }
            // ... (Autres opérateurs)
            _ => false,
        }
    }

    // --- Tri ---

    fn compare_docs(&self, a: &Value, b: &Value, sort_fields: &[SortField]) -> Ordering {
        for sort in sort_fields {
            let va = self.get_field_value(a, &sort.field);
            let vb = self.get_field_value(b, &sort.field);
            let ord = match (va, vb) {
                (Some(a), Some(b)) => self.compare_json_values(a, b),
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
                (None, None) => Ordering::Equal,
            };
            if ord != Ordering::Equal {
                return match sort.order {
                    SortOrder::Asc => ord,
                    SortOrder::Desc => ord.reverse(),
                };
            }
        }
        Ordering::Equal
    }

    fn get_field_value<'b>(&self, doc: &'b Value, path: &str) -> Option<&'b Value> {
        if !path.contains('.') {
            return doc.get(path);
        }
        doc.pointer(&format!("/{}", path.replace('.', "/")))
    }

    fn compare_values(&self, a: Option<&Value>, b: &Value) -> Option<Ordering> {
        match a {
            Some(val_a) => {
                if let (Some(na), Some(nb)) = (val_a.as_f64(), b.as_f64()) {
                    return na.partial_cmp(&nb);
                }
                if let (Some(sa), Some(sb)) = (val_a.as_str(), b.as_str()) {
                    return Some(sa.cmp(sb));
                }
                // CORRECTION: Comparaison bool
                if let (Some(ba), Some(bb)) = (val_a.as_bool(), b.as_bool()) {
                    return Some(ba.cmp(&bb)); // &bb car cmp attend une référence
                }
                None
            }
            None => None,
        }
    }

    fn compare_json_values(&self, a: &Value, b: &Value) -> Ordering {
        if let (Some(na), Some(nb)) = (a.as_f64(), b.as_f64()) {
            return na.partial_cmp(&nb).unwrap_or(Ordering::Equal);
        }
        if let (Some(sa), Some(sb)) = (a.as_str(), b.as_str()) {
            return sa.cmp(sb);
        }
        a.to_string().cmp(&b.to_string())
    }
}
