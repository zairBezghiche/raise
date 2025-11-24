//! Exécuteur de requêtes pour la base de données JSON
//!
//! Ce module contient la logique d'exécution des requêtes :
//! - Évaluation des filtres
//! - Application du tri
//! - Gestion de la pagination
//! - Opérations CRUD

use anyhow::{bail, Result};
use serde_json::Value;
use std::cmp::Ordering;
use uuid;

// Import du CollectionsManager
use crate::json_db::collections::manager::CollectionsManager;

use super::{
    ComparisonOperator, Condition, DeleteResult, FilterOperator, InsertResult, Projection, Query,
    QueryFilter, QueryResult, SortField, SortOrder, UpdateResult, UpsertResult,
};

/// Exécuteur de requêtes
#[derive(Debug)]
pub struct QueryExecutor<'a> {
    // Référence au manager
    manager: &'a CollectionsManager<'a>,
}

impl<'a> QueryExecutor<'a> {
    /// Crée un nouvel exécuteur
    pub fn new(manager: &'a CollectionsManager<'a>) -> Self {
        Self { manager }
    }

    /// Exécute une requête SELECT
    pub async fn execute(&self, query: Query) -> Result<QueryResult> {
        // 1. Récupérer les documents de la collection
        let all_documents = self.get_collection_documents(&query.collection).await?;

        // 2. Appliquer le filtre
        let filtered = if let Some(ref filter) = query.filter {
            all_documents
                .into_iter()
                .filter(|doc| self.evaluate_filter(doc, filter))
                .collect()
        } else {
            all_documents
        };

        let total_count = filtered.len() as u64;

        // 3. Appliquer le tri
        let mut sorted = if let Some(ref sort_fields) = query.sort {
            let mut docs = filtered;
            self.sort_documents(&mut docs, sort_fields)?;
            docs
        } else {
            filtered
        };

        // 4. Appliquer la projection
        if let Some(ref projection) = query.projection {
            sorted = sorted
                .into_iter()
                .map(|doc| self.apply_projection(doc, projection))
                .collect();
        }

        // 5. Appliquer la pagination
        let offset = query.offset.unwrap_or(0);
        let paginated: Vec<Value> = sorted
            .into_iter()
            .skip(offset)
            .take(query.limit.unwrap_or(usize::MAX))
            .collect();

        Ok(QueryResult {
            documents: paginated,
            total_count,
            offset,
            limit: query.limit,
        })
    }

    /// Insère des documents dans une collection
    pub async fn insert(&self, _collection: &str, documents: Vec<Value>) -> Result<InsertResult> {
        // TODO: Intégration avec CollectionsManager (utiliser self.manager)
        // Générer des IDs si nécessaire
        let mut inserted_ids = Vec::new();

        for doc in &documents {
            let id = self.extract_or_generate_id(doc)?;
            inserted_ids.push(id);
        }

        Ok(InsertResult {
            inserted_count: documents.len() as u64,
            inserted_ids,
        })
    }

    /// Insert ou update (upsert) des documents
    pub async fn upsert(
        &self,
        collection: &str,
        documents: Vec<Value>,
        match_fields: Vec<String>,
    ) -> Result<UpsertResult> {
        // TODO: Intégration avec CollectionsManager (utiliser self.manager)
        let mut inserted_count = 0;
        let mut updated_count = 0;
        let mut affected_ids = Vec::new();

        for doc in documents {
            // Créer un filtre basé sur les match_fields
            let filter = self.build_match_filter(&doc, &match_fields)?;

            // Vérifier si un document existe
            let existing = self.find_one(collection, Some(filter.clone())).await?;

            if existing.is_some() {
                // Update
                let id = self.extract_or_generate_id(&doc)?;
                affected_ids.push(id);
                updated_count += 1;
            } else {
                // Insert
                let id = self.extract_or_generate_id(&doc)?;
                affected_ids.push(id);
                inserted_count += 1;
            }
        }

        Ok(UpsertResult {
            inserted_count,
            updated_count,
            affected_ids,
        })
    }

    /// Met à jour des documents correspondant à un filtre
    pub async fn update(
        &self,
        collection: &str,
        filter: QueryFilter,
        _updates: Value,
    ) -> Result<UpdateResult> {
        // TODO: Intégration avec CollectionsManager (utiliser self.manager)
        let documents = self.get_collection_documents(collection).await?;

        let matched: Vec<&Value> = documents
            .iter()
            .filter(|doc| self.evaluate_filter(doc, &filter))
            .collect();

        let matched_count = matched.len() as u64;

        // Simuler la modification (dans la vraie implémentation, on modifierait réellement)
        let modified_count = matched_count;

        Ok(UpdateResult {
            matched_count,
            modified_count,
        })
    }

    /// Supprime des documents correspondant à un filtre
    pub async fn delete(&self, collection: &str, filter: QueryFilter) -> Result<DeleteResult> {
        // TODO: Intégration avec CollectionsManager (utiliser self.manager)
        let documents = self.get_collection_documents(collection).await?;

        let to_delete: Vec<&Value> = documents
            .iter()
            .filter(|doc| self.evaluate_filter(doc, &filter))
            .collect();

        Ok(DeleteResult {
            deleted_count: to_delete.len() as u64,
        })
    }

    /// Liste toutes les collections disponibles
    pub async fn list_collections(&self) -> Result<Vec<String>> {
        // Utilisation réelle du CollectionsManager
        self.manager.list_collection_names().map_err(|e| e.into())
    }

    /// Compte le nombre de documents correspondant à un filtre
    pub async fn count(&self, collection: &str, filter: Option<QueryFilter>) -> Result<u64> {
        let documents = self.get_collection_documents(collection).await?;

        if let Some(filter) = filter {
            Ok(documents
                .iter()
                .filter(|doc| self.evaluate_filter(doc, &filter))
                .count() as u64)
        } else {
            Ok(documents.len() as u64)
        }
    }

    // ========================================================================
    // MÉTHODES PRIVÉES - ÉVALUATION ET MANIPULATION
    // ========================================================================

    /// Évalue un filtre sur un document
    fn evaluate_filter(&self, document: &Value, filter: &QueryFilter) -> bool {
        match filter.operator {
            FilterOperator::And => filter
                .conditions
                .iter()
                .all(|cond| self.evaluate_condition(document, cond)),

            FilterOperator::Or => filter
                .conditions
                .iter()
                .any(|cond| self.evaluate_condition(document, cond)),

            FilterOperator::Not => !filter
                .conditions
                .iter()
                .all(|cond| self.evaluate_condition(document, cond)),
        }
    }

    /// Évalue une condition sur un document
    fn evaluate_condition(&self, document: &Value, condition: &Condition) -> bool {
        let field_value = self.get_field_value(document, &condition.field);

        match &condition.operator {
            ComparisonOperator::Eq => field_value == Some(&condition.value),

            ComparisonOperator::Ne => field_value != Some(&condition.value),

            ComparisonOperator::Gt => {
                if let (Some(field_val), Some(cond_val)) = (field_value, condition.value.as_f64()) {
                    if let Some(field_num) = field_val.as_f64() {
                        return field_num > cond_val;
                    }
                }
                false
            }

            ComparisonOperator::Gte => {
                if let (Some(field_val), Some(cond_val)) = (field_value, condition.value.as_f64()) {
                    if let Some(field_num) = field_val.as_f64() {
                        return field_num >= cond_val;
                    }
                }
                false
            }

            ComparisonOperator::Lt => {
                if let (Some(field_val), Some(cond_val)) = (field_value, condition.value.as_f64()) {
                    if let Some(field_num) = field_val.as_f64() {
                        return field_num < cond_val;
                    }
                }
                false
            }

            ComparisonOperator::Lte => {
                if let (Some(field_val), Some(cond_val)) = (field_value, condition.value.as_f64()) {
                    if let Some(field_num) = field_val.as_f64() {
                        return field_num <= cond_val;
                    }
                }
                false
            }

            ComparisonOperator::In => {
                if let (Some(field_val), Some(array)) = (field_value, condition.value.as_array()) {
                    return array.contains(field_val);
                }
                false
            }

            ComparisonOperator::Contains => {
                if let Some(field_val) = field_value {
                    if let Some(field_str) = field_val.as_str() {
                        if let Some(search_str) = condition.value.as_str() {
                            return field_str.contains(search_str);
                        }
                    }
                }
                false
            }

            ComparisonOperator::StartsWith => {
                if let Some(field_val) = field_value {
                    if let Some(field_str) = field_val.as_str() {
                        if let Some(prefix) = condition.value.as_str() {
                            return field_str.starts_with(prefix);
                        }
                    }
                }
                false
            }

            ComparisonOperator::EndsWith => {
                if let Some(field_val) = field_value {
                    if let Some(field_str) = field_val.as_str() {
                        if let Some(suffix) = condition.value.as_str() {
                            return field_str.ends_with(suffix);
                        }
                    }
                }
                false
            }

            ComparisonOperator::Matches => {
                // TODO: Implémenter le support des regex
                false
            }
        }
    }

    /// Récupère la valeur d'un champ dans un document (supporte les chemins imbriqués)
    fn get_field_value(&self, document: &'a Value, field_path: &str) -> Option<&'a Value> {
        let parts: Vec<&str> = field_path.split('.').collect();
        let mut current = document;

        for part in parts {
            current = current.get(part)?;
        }

        Some(current)
    }

    /// Trie une liste de documents selon les champs de tri spécifiés
    fn sort_documents(&self, documents: &mut [Value], sort_fields: &[SortField]) -> Result<()> {
        documents.sort_by(|a, b| {
            for sort_field in sort_fields {
                let val_a = self.get_field_value(a, &sort_field.field);
                let val_b = self.get_field_value(b, &sort_field.field);

                let cmp = match (val_a, val_b) {
                    (Some(a), Some(b)) => self.compare_values(a, b),
                    (Some(_), None) => Ordering::Greater,
                    (None, Some(_)) => Ordering::Less,
                    (None, None) => Ordering::Equal,
                };

                let ordered_cmp = match sort_field.order {
                    SortOrder::Asc => cmp,
                    SortOrder::Desc => cmp.reverse(),
                };

                if ordered_cmp != Ordering::Equal {
                    return ordered_cmp;
                }
            }
            Ordering::Equal
        });

        Ok(())
    }

    /// Compare deux valeurs JSON
    fn compare_values(&self, a: &Value, b: &Value) -> Ordering {
        match (a, b) {
            // Nombres
            (Value::Number(a), Value::Number(b)) => {
                let a_f64 = a.as_f64().unwrap_or(0.0);
                let b_f64 = b.as_f64().unwrap_or(0.0);
                a_f64.partial_cmp(&b_f64).unwrap_or(Ordering::Equal)
            }
            // Chaînes
            (Value::String(a), Value::String(b)) => a.cmp(b),
            // Booléens
            (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
            // Null
            (Value::Null, Value::Null) => Ordering::Equal,
            // Types différents : ordre arbitraire mais cohérent
            (Value::Null, _) => Ordering::Less,
            (_, Value::Null) => Ordering::Greater,
            (Value::Bool(_), _) => Ordering::Less,
            (_, Value::Bool(_)) => Ordering::Greater,
            (Value::Number(_), _) => Ordering::Less,
            (_, Value::Number(_)) => Ordering::Greater,
            (Value::String(_), _) => Ordering::Less,
            (_, Value::String(_)) => Ordering::Greater,
            _ => Ordering::Equal,
        }
    }

    /// Applique une projection sur un document
    fn apply_projection(&self, mut document: Value, projection: &Projection) -> Value {
        if let Value::Object(ref mut map) = document {
            match projection {
                Projection::Include(fields) => {
                    // Garder seulement les champs spécifiés
                    map.retain(|key, _| fields.contains(key));
                }
                Projection::Exclude(fields) => {
                    // Retirer les champs spécifiés
                    for field in fields {
                        map.remove(field);
                    }
                }
            }
        }
        document
    }

    /// Construit un filtre basé sur des champs de correspondance
    fn build_match_filter(&self, document: &Value, match_fields: &[String]) -> Result<QueryFilter> {
        let mut conditions = Vec::new();

        for field in match_fields {
            if let Some(value) = self.get_field_value(document, field) {
                conditions.push(Condition {
                    field: field.clone(),
                    operator: ComparisonOperator::Eq,
                    value: value.clone(),
                });
            } else {
                bail!("Match field '{}' not found in document", field);
            }
        }

        Ok(QueryFilter {
            operator: FilterOperator::And,
            conditions,
        })
    }

    /// Extrait l'ID d'un document ou en génère un nouveau
    fn extract_or_generate_id(&self, document: &Value) -> Result<String> {
        if let Some(id) = document.get("id") {
            if let Some(id_str) = id.as_str() {
                return Ok(id_str.to_string());
            }
        }

        // Générer un nouvel ID (UUID)
        Ok(uuid::Uuid::new_v4().to_string())
    }

    /// Trouve un seul document correspondant à un filtre
    async fn find_one(
        &self,
        collection: &str,
        filter: Option<QueryFilter>,
    ) -> Result<Option<Value>> {
        let documents = self.get_collection_documents(collection).await?;

        if let Some(filter) = filter {
            Ok(documents
                .into_iter()
                .find(|doc| self.evaluate_filter(doc, &filter)))
        } else {
            Ok(documents.into_iter().next())
        }
    }

    /// Récupère tous les documents d'une collection
    async fn get_collection_documents(&self, collection: &str) -> Result<Vec<Value>> {
        self.manager.list_all(collection).map_err(|e| e.into())
    }
}

#[cfg(test)]
// Suppression de l'avertissement "invalid value" pour permettre le mock avec transmute
#[allow(invalid_value)]
mod tests {
    use super::*;
    use serde_json::json;

    // 💡 CORRECTION DU MOCK
    // Crée un exécuteur avec un manager "faux" mais non-nul (pointeur 0x1)
    // pour satisfaire les exigences de Rust sur les références non-nulles.
    // C'est "safe" TANT QUE les tests n'appellent pas de méthode utilisant self.manager.
    fn get_executor_stub() -> QueryExecutor<'static> {
        let ptr = std::ptr::NonNull::<CollectionsManager<'static>>::dangling().as_ptr();
        let fake_manager_ref = unsafe { &*ptr };
        QueryExecutor {
            manager: fake_manager_ref,
        }
    }

    #[test]
    fn test_evaluate_condition_eq() {
        let executor = get_executor_stub();
        let doc = json!({"name": "Alice", "age": 30});
        let condition = Condition {
            field: "name".to_string(),
            operator: ComparisonOperator::Eq,
            value: json!("Alice"),
        };

        assert!(executor.evaluate_condition(&doc, &condition));
    }

    #[test]
    fn test_evaluate_condition_gt() {
        let executor = get_executor_stub();
        let doc = json!({"age": 30});
        let condition = Condition {
            field: "age".to_string(),
            operator: ComparisonOperator::Gt,
            value: json!(18),
        };

        assert!(executor.evaluate_condition(&doc, &condition));
    }

    #[test]
    fn test_get_field_value_nested() {
        let executor = get_executor_stub();
        let doc = json!({
            "user": {
                "profile": {
                    "name": "Alice"
                }
            }
        });

        let value = executor.get_field_value(&doc, "user.profile.name");
        assert_eq!(value, Some(&json!("Alice")));
    }

    #[test]
    fn test_sort_documents() {
        let executor = get_executor_stub();
        let mut docs = vec![
            json!({"name": "Charlie", "age": 30}),
            json!({"name": "Alice", "age": 25}),
            json!({"name": "Bob", "age": 35}),
        ];

        let sort_fields = vec![SortField {
            field: "age".to_string(),
            order: SortOrder::Asc,
        }];

        executor.sort_documents(&mut docs, &sort_fields).unwrap();

        assert_eq!(docs[0].get("name").unwrap(), "Alice");
        assert_eq!(docs[2].get("name").unwrap(), "Bob");
    }

    #[test]
    fn test_apply_projection_include() {
        let executor = get_executor_stub();
        let doc = json!({
            "name": "Alice",
            "age": 30,
            "email": "alice@example.com"
        });

        let projected = executor.apply_projection(
            doc,
            &Projection::Include(vec!["name".to_string(), "age".to_string()]),
        );

        assert!(projected.get("name").is_some());
        assert!(projected.get("age").is_some());
        assert!(projected.get("email").is_none());
    }
}
