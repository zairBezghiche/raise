use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Expr {
    // --- Primitives ---
    Val(serde_json::Value),
    Var(String),

    // --- Logique & Comparaison ---
    And(Vec<Expr>),
    Or(Vec<Expr>),
    Not(Box<Expr>),
    #[serde(rename = "if")]
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    Eq(Box<Expr>, Box<Expr>),
    Neq(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Gte(Box<Expr>, Box<Expr>),
    Lte(Box<Expr>, Box<Expr>),

    // --- Mathématiques ---
    Add(Vec<Expr>),
    Sub(Vec<Expr>),
    Mul(Vec<Expr>),
    Div(Vec<Expr>),

    // --- 📅 NOUVEAU : DATES ---
    /// Date actuelle ISO8601
    Now,
    /// Différence en jours (end - start)
    DateDiff {
        start: Box<Expr>,
        end: Box<Expr>,
    },
    /// Ajoute X jours
    DateAdd {
        date: Box<Expr>,
        days: Box<Expr>,
    },

    // --- 🔤 NOUVEAU : STRINGS ---
    Concat(Vec<Expr>),
    Upper(Box<Expr>),
    RegexMatch {
        value: Box<Expr>,
        pattern: Box<Expr>,
    },

    // --- 🔍 NOUVEAU : LOOKUPS (Cross-Collection) ---
    Lookup {
        collection: String, // Nom de la collection cible
        id: Box<Expr>,      // ID du document (expression dynamique)
        field: String,      // Champ à lire
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub target: String,
    pub expr: Expr,
}
