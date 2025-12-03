// FICHIER : src-tauri/src/model_engine/tests.rs

#[cfg(test)]
mod integration_tests {
    use crate::json_db::collections::manager::CollectionsManager;
    use crate::json_db::storage::file_storage;
    use crate::json_db::test_utils::init_test_env;
    use crate::model_engine::loader::ModelLoader;
    use serde_json::json;

    #[test]
    fn test_semantic_loading_oa_and_sa() {
        // 1. Initialisation de l'environnement de test isolé
        let env = init_test_env();
        let cfg = &env.cfg;
        let space = "test_space_model";
        let db = "test_db_model";

        // Création physique de la DB
        file_storage::create_db(cfg, space, db).expect("create db");

        let storage = &env.storage;
        let mgr = CollectionsManager::new(storage, space, db);

        // 2. Insertion d'un Acteur OA (avec contexte JSON-LD explicite)
        // Le loader doit reconnaître "oa:OperationalActor" et l'expandre
        let oa_actor = json!({
            "@context": {
                "oa": "https://genaptitude.io/ontology/arcadia/oa#",
                "name": "http://www.w3.org/2004/02/skos/core#prefLabel"
            },
            "@id": "urn:uuid:actor-oa-001",
            "@type": "oa:OperationalActor",
            "name": "Opérateur Humain",
            "description": "Un opérateur dans la boucle"
        });
        // On insère dans une collection arbitraire "actors", le loader scanne tout
        mgr.insert_with_schema("actors", oa_actor)
            .expect("insert oa actor");

        // 3. Insertion d'une Fonction Système SA
        // Ici on teste la capacité à mapper "sa:SystemFunction" vers model.sa.functions
        let sa_func = json!({
            "@context": {
                "sa": "https://genaptitude.io/ontology/arcadia/sa#",
                "name": "http://www.w3.org/2004/02/skos/core#prefLabel"
            },
            "@id": "urn:uuid:func-sa-001",
            "@type": "sa:SystemFunction",
            "name": "Calculer Trajectoire",
            "criticality": "high"
        });
        mgr.insert_with_schema("functions", sa_func)
            .expect("insert sa function");

        // 4. Chargement du modèle via le ModelEngine
        // On utilise le constructeur découplé 'from_engine'
        let loader = ModelLoader::from_engine(storage, space, db);

        // Charge tout le modèle en mémoire (synchrone pour le test)
        let model = loader.load_full_model().expect("load full model");

        // 5. Assertions

        // A. Vérification OA (Operational Analysis)
        assert_eq!(model.oa.actors.len(), 1, "Devrait avoir 1 acteur OA");
        let actor = &model.oa.actors[0];
        assert_eq!(actor.name, "Opérateur Humain");
        // Vérifie que l'URI a été complètement résolue
        assert_eq!(
            actor.kind,
            "https://genaptitude.io/ontology/arcadia/oa#OperationalActor"
        );

        // B. Vérification SA (System Analysis)
        assert_eq!(model.sa.functions.len(), 1, "Devrait avoir 1 fonction SA");
        let func = &model.sa.functions[0];
        assert_eq!(func.name, "Calculer Trajectoire");
        assert_eq!(
            func.kind,
            "https://genaptitude.io/ontology/arcadia/sa#SystemFunction"
        );

        // Vérifie que les propriétés dynamiques sont préservées
        assert_eq!(
            func.properties.get("criticality").and_then(|v| v.as_str()),
            Some("high")
        );

        println!("✅ Test Semantic Loader réussi : OA et SA correctement dispatchés.");
    }
}
