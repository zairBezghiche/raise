use crate::rules_engine::ast::Expr;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use regex::Regex;
use serde_json::{json, Value};

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("Champ introuvable : {0}")]
    VarNotFound(String),
    #[error("Type incompatible : attendu nombre")]
    NotANumber,
    #[error("Type incompatible : attendu chaîne de caractères")]
    NotAString,
    #[error("Format de date invalide (attendu ISO8601/RFC3339) : {0}")]
    InvalidDate(String),
    #[error("Erreur Regex : {0}")]
    InvalidRegex(String),
    #[error("Erreur générique : {0}")]
    Generic(String),
}

/// Interface pour permettre à l'évaluateur d'interroger le monde extérieur (DB)
pub trait DataProvider {
    fn get_value(&self, collection: &str, id: &str, field: &str) -> Option<Value>;
}

/// Une implémentation "vide" pour les tests unitaires simples ou quand aucun accès DB n'est requis
pub struct NoOpDataProvider;
impl DataProvider for NoOpDataProvider {
    fn get_value(&self, _c: &str, _id: &str, _f: &str) -> Option<Value> {
        None
    }
}

pub struct Evaluator;

impl Evaluator {
    /// Evalue une expression.
    /// `provider` permet d'accéder aux données cross-collection.
    pub fn evaluate(
        expr: &Expr,
        context: &Value,
        provider: &dyn DataProvider,
    ) -> Result<Value, EvalError> {
        match expr {
            Expr::Val(v) => Ok(v.clone()),

            Expr::Var(path) => {
                let ptr = if path.starts_with('/') {
                    path.clone()
                } else {
                    format!("/{}", path.replace('.', "/"))
                };
                context
                    .pointer(&ptr)
                    .cloned()
                    .ok_or_else(|| EvalError::VarNotFound(path.clone()))
            }

            // --- 📅 DATES ---
            Expr::Now => Ok(json!(Utc::now().to_rfc3339())),

            Expr::DateDiff { start, end } => {
                let s_str = Self::evaluate_as_string(start, context, provider)?;
                let e_str = Self::evaluate_as_string(end, context, provider)?;

                let s_date = parse_date(&s_str)?;
                let e_date = parse_date(&e_str)?;

                let diff = e_date.signed_duration_since(s_date);
                Ok(json!(diff.num_days()))
            }

            Expr::DateAdd { date, days } => {
                let d_str = Self::evaluate_as_string(date, context, provider)?;
                let days_num = Self::evaluate_as_f64(days, context, provider)?;

                let parsed_date = parse_date(&d_str)?;
                let new_date = parsed_date + Duration::days(days_num as i64);

                Ok(json!(new_date.to_rfc3339()))
            }

            // --- 🔤 STRINGS ---
            Expr::Concat(args) => {
                let mut result = String::new();
                for arg in args {
                    let val = Self::evaluate(arg, context, provider)?;
                    if let Some(s) = val.as_str() {
                        result.push_str(s);
                    } else if let Some(n) = val.as_f64() {
                        result.push_str(&n.to_string());
                    } else if let Some(b) = val.as_bool() {
                        result.push_str(&b.to_string());
                    }
                }
                Ok(json!(result))
            }

            Expr::Upper(arg) => {
                let s = Self::evaluate_as_string(arg, context, provider)?;
                Ok(json!(s.to_uppercase()))
            }

            Expr::RegexMatch { value, pattern } => {
                let val_str = Self::evaluate_as_string(value, context, provider)?;
                let pat_str = Self::evaluate_as_string(pattern, context, provider)?;

                let re =
                    Regex::new(&pat_str).map_err(|e| EvalError::InvalidRegex(e.to_string()))?;
                Ok(json!(re.is_match(&val_str)))
            }

            // --- 🔍 LOOKUP ---
            Expr::Lookup {
                collection,
                id,
                field,
            } => {
                // 1. On évalue l'ID (car ça peut être une variable ou un calcul)
                let id_val = Self::evaluate_as_string(id, context, provider)?;

                // 2. On appelle le provider
                match provider.get_value(collection, &id_val, field) {
                    Some(v) => Ok(v),
                    None => Ok(Value::Null),
                }
            }

            // --- MATHS & LOGIQUE (Classique) ---
            Expr::Add(args) => {
                let mut sum = 0.0;
                for arg in args {
                    sum += Self::evaluate_as_f64(arg, context, provider)?;
                }
                Ok(json!(sum))
            }
            Expr::Sub(args) => {
                if args.is_empty() {
                    return Ok(json!(0.0));
                }
                let mut iter = args.iter();
                let first = Self::evaluate_as_f64(iter.next().unwrap(), context, provider)?;
                let mut result = first;
                for arg in iter {
                    result -= Self::evaluate_as_f64(arg, context, provider)?;
                }
                Ok(json!(result))
            }
            Expr::Mul(args) => {
                let mut prod = 1.0;
                for arg in args {
                    prod *= Self::evaluate_as_f64(arg, context, provider)?;
                }
                Ok(json!(prod))
            }
            Expr::Div(args) => {
                if args.is_empty() {
                    return Ok(json!(1.0));
                }
                let mut iter = args.iter();
                let first = Self::evaluate_as_f64(iter.next().unwrap(), context, provider)?;
                let mut result = first;
                for arg in iter {
                    let divisor = Self::evaluate_as_f64(arg, context, provider)?;
                    if divisor == 0.0 {
                        return Ok(Value::Null);
                    }
                    result /= divisor;
                }
                Ok(json!(result))
            }

            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond = Self::evaluate(condition, context, provider)?;
                if is_truthy(&cond) {
                    Self::evaluate(then_branch, context, provider)
                } else {
                    Self::evaluate(else_branch, context, provider)
                }
            }

            Expr::Eq(a, b) => {
                let va = Self::evaluate(a, context, provider)?;
                let vb = Self::evaluate(b, context, provider)?;
                Ok(json!(va == vb))
            }
            Expr::Neq(a, b) => {
                let va = Self::evaluate(a, context, provider)?;
                let vb = Self::evaluate(b, context, provider)?;
                Ok(json!(va != vb))
            }
            Expr::Gt(a, b) => {
                let va = Self::evaluate_as_f64(a, context, provider)?;
                let vb = Self::evaluate_as_f64(b, context, provider)?;
                Ok(json!(va > vb))
            }
            Expr::Gte(a, b) => {
                let va = Self::evaluate_as_f64(a, context, provider)?;
                let vb = Self::evaluate_as_f64(b, context, provider)?;
                Ok(json!(va >= vb))
            }
            Expr::Lt(a, b) => {
                let va = Self::evaluate_as_f64(a, context, provider)?;
                let vb = Self::evaluate_as_f64(b, context, provider)?;
                Ok(json!(va < vb))
            }
            Expr::Lte(a, b) => {
                let va = Self::evaluate_as_f64(a, context, provider)?;
                let vb = Self::evaluate_as_f64(b, context, provider)?;
                Ok(json!(va <= vb))
            }
            Expr::And(args) => {
                for arg in args {
                    let val = Self::evaluate(arg, context, provider)?;
                    if !is_truthy(&val) {
                        return Ok(json!(false));
                    }
                }
                Ok(json!(true))
            }
            Expr::Or(args) => {
                for arg in args {
                    let val = Self::evaluate(arg, context, provider)?;
                    if is_truthy(&val) {
                        return Ok(json!(true));
                    }
                }
                Ok(json!(false))
            }
            Expr::Not(inner) => {
                let val = Self::evaluate(inner, context, provider)?;
                Ok(json!(!is_truthy(&val)))
            }
        }
    }

    // --- Helpers ---

    fn evaluate_as_string(
        expr: &Expr,
        ctx: &Value,
        p: &dyn DataProvider,
    ) -> Result<String, EvalError> {
        let val = Self::evaluate(expr, ctx, p)?;
        val.as_str()
            .map(|s| s.to_string())
            .ok_or(EvalError::NotAString)
    }

    fn evaluate_as_f64(expr: &Expr, ctx: &Value, p: &dyn DataProvider) -> Result<f64, EvalError> {
        let val = Self::evaluate(expr, ctx, p)?;
        val.as_f64().ok_or(EvalError::NotANumber)
    }
}

// Utils
fn parse_date(s: &str) -> Result<DateTime<Utc>, EvalError> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(DateTime::<Utc>::from_naive_utc_and_offset(
            d.and_hms_opt(0, 0, 0).unwrap(),
            Utc,
        ));
    }
    Err(EvalError::InvalidDate(s.to_string()))
}

fn is_truthy(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Null => false,
        _ => true,
    }
}
