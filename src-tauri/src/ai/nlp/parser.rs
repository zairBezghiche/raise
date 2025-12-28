use crate::ai::nlp::{preprocessing, tokenizers};

#[derive(Debug, PartialEq)]
pub enum CommandType {
    Create,
    Delete,
    Search,
    Explain,
    Unknown,
}

/// Tente de deviner l'intention brute par mots-clés (Rule-Based).
/// Utile pour un "Fast Path" avant d'appeler le LLM.
pub fn simple_intent_detection(text: &str) -> CommandType {
    let normalized = preprocessing::normalize(text);
    let tokens = tokenizers::tokenize(&normalized);

    // Analyse simple des verbes d'action
    if tokens
        .iter()
        .any(|t| t.contains("creer") || t.contains("ajout") || t.contains("nouv"))
    {
        return CommandType::Create;
    }
    if tokens
        .iter()
        .any(|t| t.contains("supprim") || t.contains("retir") || t.contains("effac"))
    {
        return CommandType::Delete;
    }
    if tokens
        .iter()
        .any(|t| t.contains("cherch") || t.contains("trouv") || t.contains("list"))
    {
        return CommandType::Search;
    }
    if tokens
        .iter()
        .any(|t| t.contains("expliqu") || t.contains("comment") || t.contains("quois"))
    {
        return CommandType::Explain;
    }

    CommandType::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_create() {
        assert_eq!(
            simple_intent_detection("Je veux créer une fonction"),
            CommandType::Create
        );
        assert_eq!(
            simple_intent_detection("Ajoute un composant"),
            CommandType::Create
        );
    }

    #[test]
    fn test_detect_explain() {
        assert_eq!(
            simple_intent_detection("Explique-moi Arcadia"),
            CommandType::Explain
        );
    }
}
