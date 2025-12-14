use crate::rules_engine::ast::Expr;
use std::collections::HashSet;

pub struct Analyzer;

impl Analyzer {
    pub fn get_dependencies(expr: &Expr) -> HashSet<String> {
        let mut deps = HashSet::new();
        Self::visit(expr, &mut deps);
        deps
    }

    fn visit(expr: &Expr, deps: &mut HashSet<String>) {
        match expr {
            // Primitives sans dépendances
            Expr::Val(_) | Expr::Now => {}

            // Variable locale
            Expr::Var(name) => {
                deps.insert(name.clone());
            }

            // Listes
            Expr::And(l)
            | Expr::Or(l)
            | Expr::Add(l)
            | Expr::Sub(l)
            | Expr::Mul(l)
            | Expr::Div(l)
            | Expr::Concat(l) => {
                for item in l {
                    Self::visit(item, deps);
                }
            }

            // Unaires
            Expr::Not(e) | Expr::Upper(e) => Self::visit(e, deps),

            // Structures complexes
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                Self::visit(condition, deps);
                Self::visit(then_branch, deps);
                Self::visit(else_branch, deps);
            }

            // Binaires
            Expr::Eq(a, b)
            | Expr::Neq(a, b)
            | Expr::Gt(a, b)
            | Expr::Lt(a, b)
            | Expr::Gte(a, b)
            | Expr::Lte(a, b)
            | Expr::DateDiff { start: a, end: b }
            | Expr::DateAdd { date: a, days: b }
            | Expr::RegexMatch {
                value: a,
                pattern: b,
            } => {
                Self::visit(a, deps);
                Self::visit(b, deps);
            }

            // Lookup : Seul l'ID dépend du contexte courant
            Expr::Lookup { id, .. } => {
                Self::visit(id, deps);
            }
        }
    }
}
