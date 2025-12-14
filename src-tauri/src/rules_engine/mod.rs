pub mod analyzer;
pub mod ast;
pub mod evaluator;
pub mod store;

pub use analyzer::Analyzer;
pub use ast::{Expr, Rule};
// EXPORT CRUCIAL : On rend public DataProvider et NoOpDataProvider
pub use evaluator::{DataProvider, EvalError, Evaluator, NoOpDataProvider};
pub use store::RuleStore;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules_engine::ast::Expr;
    use serde_json::json;

    #[test]
    fn test_rete_light_workflow() {
        // ... (Code existant inchangé, sauf l'appel evaluate)
        let rule_expr = Expr::Mul(vec![
            Expr::Var("item.qty".to_string()),
            Expr::Var("item.price".to_string()),
        ]);

        let dependencies = Analyzer::get_dependencies(&rule_expr);
        assert!(dependencies.contains("item.qty"));
        assert!(dependencies.contains("item.price"));

        let context = json!({
            "item": { "qty": 5, "price": 10.5 }
        });

        // CORRECTION : On passe NoOpDataProvider (le 3ème argument)
        let provider = NoOpDataProvider;
        let result = Evaluator::evaluate(&rule_expr, &context, &provider);

        match result {
            Ok(val) => assert_eq!(val, 52.5),
            Err(e) => panic!("Erreur : {}", e),
        }
    }

    #[test]
    fn test_rule_store_indexing() {
        use std::collections::HashSet;
        let mut store = RuleStore::new();

        let r1 = Rule {
            id: "calc_total".into(),
            target: "total".into(),
            expr: Expr::Mul(vec![Expr::Var("qty".into()), Expr::Var("price".into())]),
        };

        store.register_rule("users", r1);

        let mut changes = HashSet::new();
        changes.insert("qty".to_string());

        let impacted = store.get_impacted_rules("users", &changes);
        assert_eq!(impacted.len(), 1);
        assert_eq!(impacted[0].id, "calc_total");
    }

    #[test]
    fn test_logic_and_comparison() {
        let rule = Expr::If {
            condition: Box::new(Expr::Gte(
                Box::new(Expr::Var("age".to_string())),
                Box::new(Expr::Val(json!(18))),
            )),
            then_branch: Box::new(Expr::Val(json!("Majeur"))),
            else_branch: Box::new(Expr::Val(json!("Mineur"))),
        };

        let ctx_kid = json!({ "age": 12 });
        let ctx_adult = json!({ "age": 25 });
        let provider = NoOpDataProvider; // Dummy provider

        assert_eq!(
            Evaluator::evaluate(&rule, &ctx_kid, &provider).unwrap(),
            json!("Mineur")
        );
        assert_eq!(
            Evaluator::evaluate(&rule, &ctx_adult, &provider).unwrap(),
            json!("Majeur")
        );
    }
}
