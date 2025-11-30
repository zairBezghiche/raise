// FICHIER : src-tauri/src/json_db/query/sql.rs

use anyhow::{bail, Result};
use serde_json::Value;
use sqlparser::ast::{
    BinaryOperator, Expr, OrderByKind, Query as SqlQuery, SetExpr, Statement, TableFactor,
    UnaryOperator, Value as SqlValue,
};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

use super::{
    ComparisonOperator, Condition, FilterOperator, Query, QueryFilter, SortField, SortOrder,
};

pub fn parse_sql(sql: &str) -> Result<Query> {
    let dialect = GenericDialect {};
    let ast = Parser::parse_sql(&dialect, sql)?;

    if ast.len() != 1 {
        bail!("Une seule requête SQL à la fois est supportée");
    }

    match &ast[0] {
        Statement::Query(q) => translate_query(q),
        _ => bail!("Seules les requêtes SELECT sont supportées pour le moment"),
    }
}

fn translate_query(sql_query: &SqlQuery) -> Result<Query> {
    // 1. LIMIT (Désactivé temporairement pour compatibilité version)
    let limit = None;

    // 2. OFFSET (Désactivé temporairement pour compatibilité version)
    let offset = None;

    // 3. ORDER BY
    let sort = if let Some(order_by_struct) = &sql_query.order_by {
        match &order_by_struct.kind {
            OrderByKind::Expressions(exprs) => {
                let mut fields = Vec::new();
                for order_expr in exprs {
                    fields.push(translate_order_by(order_expr)?);
                }
                if fields.is_empty() {
                    None
                } else {
                    Some(fields)
                }
            }
            _ => None,
        }
    } else {
        None
    };

    // 4. SELECT body
    match &*sql_query.body {
        SetExpr::Select(select) => translate_select(select, limit, offset, sort),
        _ => bail!("Syntaxe de requête non supportée (pas de UNION, VALUES, etc.)"),
    }
}

fn translate_select(
    select: &sqlparser::ast::Select,
    limit: Option<usize>,
    offset: Option<usize>,
    sort: Option<Vec<SortField>>,
) -> Result<Query> {
    if select.from.len() != 1 {
        bail!("SELECT doit cibler exactement une collection (pas de JOIN supporté)");
    }

    let collection = match &select.from[0].relation {
        TableFactor::Table { name, .. } => name.to_string(),
        _ => bail!("Clause FROM invalide"),
    };

    let filter = if let Some(selection) = &select.selection {
        Some(translate_expr(selection)?)
    } else {
        None
    };

    Ok(Query {
        collection,
        filter,
        sort,
        limit,
        offset,
        projection: None,
    })
}

fn translate_expr(expr: &Expr) -> Result<QueryFilter> {
    match expr {
        Expr::BinaryOp { left, op, right } => match op {
            BinaryOperator::And => {
                let l = translate_expr(left)?;
                let r = translate_expr(right)?;
                if matches!(l.operator, FilterOperator::And)
                    && matches!(r.operator, FilterOperator::And)
                {
                    Ok(QueryFilter {
                        operator: FilterOperator::And,
                        conditions: [l.conditions, r.conditions].concat(),
                    })
                } else {
                    bail!("Les filtres complexes (mix AND/OR) ne sont pas encore supportés");
                }
            }
            BinaryOperator::Or => {
                let l = translate_expr(left)?;
                let r = translate_expr(right)?;
                Ok(QueryFilter {
                    operator: FilterOperator::Or,
                    conditions: [l.conditions, r.conditions].concat(),
                })
            }
            _ => {
                let field = expr_to_field_name(left)?;
                let value = expr_to_value(right)?;
                let operator = match op {
                    BinaryOperator::Eq => ComparisonOperator::Eq,
                    BinaryOperator::NotEq => ComparisonOperator::Ne,
                    BinaryOperator::Gt => ComparisonOperator::Gt,
                    BinaryOperator::GtEq => ComparisonOperator::Gte,
                    BinaryOperator::Lt => ComparisonOperator::Lt,
                    BinaryOperator::LtEq => ComparisonOperator::Lte,
                    BinaryOperator::PGRegexMatch => ComparisonOperator::Matches,
                    _ => bail!("Opérateur binaire non supporté: {:?}", op),
                };
                Ok(QueryFilter {
                    operator: FilterOperator::And,
                    conditions: vec![Condition {
                        field,
                        operator,
                        value,
                    }],
                })
            }
        },
        Expr::UnaryOp {
            op: UnaryOperator::Not,
            expr,
        } => {
            let inner = translate_expr(expr)?;
            Ok(QueryFilter {
                operator: FilterOperator::Not,
                conditions: inner.conditions,
            })
        }
        Expr::Like { expr, pattern, .. } => {
            let field = expr_to_field_name(expr)?;
            let value = expr_to_value(pattern)?;
            Ok(QueryFilter {
                operator: FilterOperator::And,
                conditions: vec![Condition {
                    field,
                    operator: ComparisonOperator::Contains,
                    value,
                }],
            })
        }
        Expr::IsNull(expr) => {
            let field = expr_to_field_name(expr)?;
            Ok(QueryFilter {
                operator: FilterOperator::And,
                conditions: vec![Condition {
                    field,
                    operator: ComparisonOperator::Eq,
                    value: Value::Null,
                }],
            })
        }
        Expr::IsNotNull(expr) => {
            let field = expr_to_field_name(expr)?;
            Ok(QueryFilter {
                operator: FilterOperator::And,
                conditions: vec![Condition {
                    field,
                    operator: ComparisonOperator::Ne,
                    value: Value::Null,
                }],
            })
        }
        Expr::Nested(inner) => translate_expr(inner),
        _ => bail!("Expression non supportée dans WHERE: {:?}", expr),
    }
}

fn expr_to_field_name(expr: &Expr) -> Result<String> {
    match expr {
        Expr::Identifier(ident) => Ok(ident.value.clone()),
        Expr::CompoundIdentifier(idents) => Ok(idents
            .iter()
            .map(|i| i.value.clone())
            .collect::<Vec<_>>()
            .join(".")),
        _ => bail!("L'opérande doit être un champ valide, obtenu: {:?}", expr),
    }
}

fn expr_to_value(expr: &Expr) -> Result<Value> {
    match expr {
        Expr::Value(v_span) => sql_value_to_json(&v_span.value),
        Expr::UnaryOp {
            op: UnaryOperator::Minus,
            expr,
        } => {
            let val = expr_to_value(expr)?;
            if let Some(n) = val.as_f64() {
                Ok(Value::from(-n))
            } else if let Some(n) = val.as_i64() {
                Ok(Value::from(-n))
            } else {
                bail!("Opérateur '-' appliqué sur une valeur non numérique")
            }
        }
        _ => bail!("La valeur de comparaison doit être un littéral"),
    }
}

// Helper inutilisé pour l'instant, suppression du dead_code warning
#[allow(dead_code)]
fn expr_to_usize(expr: &Expr) -> Result<usize> {
    match expr {
        Expr::Value(v) => match &v.value {
            SqlValue::Number(n, _) => n.parse::<usize>().map_err(|e| anyhow::anyhow!(e)),
            _ => bail!("LIMIT/OFFSET doit être un nombre"),
        },
        _ => bail!("Expression LIMIT/OFFSET trop complexe"),
    }
}

fn translate_order_by(order_expr: &sqlparser::ast::OrderByExpr) -> Result<SortField> {
    let field = expr_to_field_name(&order_expr.expr)?;
    let order = match order_expr.options.asc {
        Some(false) => SortOrder::Desc,
        _ => SortOrder::Asc,
    };
    Ok(SortField { field, order })
}

fn sql_value_to_json(val: &SqlValue) -> Result<Value> {
    match val {
        SqlValue::Number(n, _) => {
            if let Ok(i) = n.parse::<i64>() {
                Ok(Value::from(i))
            } else {
                Ok(Value::from(n.parse::<f64>()?))
            }
        }
        SqlValue::SingleQuotedString(s) | SqlValue::DoubleQuotedString(s) => {
            Ok(Value::from(s.clone()))
        }
        SqlValue::Boolean(b) => Ok(Value::from(*b)),
        SqlValue::Null => Ok(Value::Null),
        _ => bail!("Type de valeur SQL non supporté: {:?}", val),
    }
}
