#[cfg(test)]
mod tests {
    use crate::ai::nlp::preprocessing;
    // use crate::ai::nlp::keyword_extractor; // On l'activera à l'étape 3

    #[test]
    fn test_normalization_french() {
        let input = "Hélène veut CRÉER une Activity !";
        // Attendu : minuscule, sans accents, sans ponctuation inutile
        let expected = "helene veut creer une activity";

        let result = preprocessing::normalize(input);
        assert_eq!(
            result, expected,
            "La normalisation doit gérer les accents et la casse."
        );
    }

    #[test]
    fn test_remove_stopwords_french() {
        // Liste de mots vides à filtrer : le, la, de, du, une, pour...
        let input = "La création de la fonction système pour le moteur";
        let expected = "création fonction système moteur"; // On garde les accents ici, le normalize se fait avant ou après selon la stratégie

        let result = preprocessing::remove_stopwords(input);

        // On vérifie que les mots vides ont disparu
        assert!(!result.contains(" la "), "Le mot 'la' doit être retiré");
        assert!(!result.contains(" pour "), "Le mot 'pour' doit être retiré");
        assert!(result.contains("système"), "Le mot important doit rester");
    }

    /* // À activer une fois keyword_extractor implémenté
    #[test]
    fn test_extract_keywords() {
        let input = "Je voudrais ajouter un composant logique";
        let keywords = keyword_extractor::extract(input);

        assert!(keywords.contains(&"composant".to_string()));
        assert!(keywords.contains(&"logique".to_string()));
        assert!(!keywords.contains(&"je".to_string()));
    }
    */
}
