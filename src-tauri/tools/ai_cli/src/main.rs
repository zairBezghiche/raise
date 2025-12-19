use anyhow::Result;
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

// Imports Métier COMPLETS
use genaptitude::ai::agents::intent_classifier::{EngineeringIntent, IntentClassifier};
use genaptitude::ai::agents::{
    business_agent::BusinessAgent, data_agent::DataAgent, epbs_agent::EpbsAgent,
    hardware_agent::HardwareAgent, software_agent::SoftwareAgent, system_agent::SystemAgent,
    transverse_agent::TransverseAgent, Agent, AgentContext,
};

// Import nécessaire pour le Chat manuel
use genaptitude::ai::llm::client::{LlmBackend, LlmClient};
// Import nécessaire pour la configuration DB
use genaptitude::json_db::storage::{JsonDbConfig, StorageEngine};

/// Outil en ligne de commande (CLI) pour piloter le module IA de GenAptitude.
#[derive(Parser)]
#[command(
    name = "ai_cli",
    author = "GenAptitude Team",
    version,
    about = "Interface CLI pour le cerveau Neuro-Symbolique"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(visible_alias = "c")]
    Chat {
        message: String,
        #[arg(long, short = 'c')]
        cloud: bool,
    },
    #[command(visible_alias = "x")]
    Classify {
        input: String,
        #[arg(long, short = 'x')]
        execute: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Chargement Environnement
    dotenv().ok();

    // 2. Config IA & DB
    let gemini_key = env::var("GENAPTITUDE_GEMINI_KEY").unwrap_or_default();
    let model_name = env::var("GENAPTITUDE_MODEL_NAME").ok();
    let local_url =
        env::var("GENAPTITUDE_LOCAL_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    let domain_path = env::var("PATH_GENAPTITUDE_DOMAIN")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap().join("data"));
    let dataset_path = env::var("PATH_GENAPTITUDE_DATASET")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap().join("dataset"));

    let client = LlmClient::new(&local_url, &gemini_key, model_name.clone());

    // CONFIGURATION DB
    let db_config = JsonDbConfig::new(domain_path.clone());
    let storage = StorageEngine::new(db_config);

    // 3. Initialisation du Contexte Agent
    let ctx = AgentContext::new(
        Arc::new(storage.clone()),
        client.clone(),
        domain_path.clone(),
        dataset_path.clone(),
    );

    let cli = Cli::parse();

    match &cli.command {
        // --- LOGIQUE CHAT ---
        Commands::Chat { message, cloud } => {
            let (backend, backend_name) = if *cloud {
                (LlmBackend::GoogleGemini, "Google Gemini ☁️")
            } else {
                (LlmBackend::LocalLlama, "Local LLM 🏠")
            };

            println!("💬 Discussion avec {}...", backend_name);
            let system_prompt = "Tu es GenAptitude, un assistant expert en ingénierie.";

            match client.ask(backend, system_prompt, message).await {
                Ok(response) => println!("\n🤖 Réponse :\n{}", response),
                Err(e) => eprintln!("❌ Erreur : {}", e),
            }
        }

        Commands::Classify { input, execute } => {
            let classifier = IntentClassifier::new(client.clone());
            println!("🧠 Analyse de l'intention: '{}'", input);
            let intent = classifier.classify(input).await;

            match intent {
                // 1. BUSINESS (OA)
                EngineeringIntent::DefineBusinessUseCase {
                    ref domain,
                    ref process_name,
                    ..
                } => {
                    println!(
                        "🚀 Exécution Business Agent pour : {} ({})",
                        process_name, domain
                    );
                    run_agent(BusinessAgent::new(), &ctx, &intent, *execute).await;
                }

                // 2. SYSTÈME (SA)
                EngineeringIntent::CreateElement { ref layer, .. } if layer == "SA" => {
                    println!("⚙️ Exécution System Agent (SA)...");
                    run_agent(SystemAgent::new(), &ctx, &intent, *execute).await;
                }

                // 3. LOGICIEL (LA) & GÉNÉRATION CODE
                EngineeringIntent::CreateElement {
                    ref layer,
                    ref element_type,
                    ..
                } if layer == "LA" || element_type.contains("Software") => {
                    println!("💻 Exécution Software Agent (LA)...");
                    run_agent(SoftwareAgent::new(), &ctx, &intent, *execute).await;
                }
                EngineeringIntent::GenerateCode {
                    ref language,
                    ref filename,
                    ..
                } => {
                    println!("👨‍💻 Génération de code ({}) -> {}", language, filename);
                    run_agent(SoftwareAgent::new(), &ctx, &intent, *execute).await;
                }

                // 4. MATÉRIEL (PA)
                EngineeringIntent::CreateElement { ref layer, .. } if layer == "PA" => {
                    println!("🔧 Exécution Hardware Agent (PA)...");
                    run_agent(HardwareAgent::new(), &ctx, &intent, *execute).await;
                }

                // 5. CONFIGURATION (EPBS)
                EngineeringIntent::CreateElement { ref layer, .. } if layer == "EPBS" => {
                    println!("📦 Exécution EPBS Agent...");
                    run_agent(EpbsAgent::new(), &ctx, &intent, *execute).await;
                }

                // 6. DONNÉES (DATA)
                EngineeringIntent::CreateElement { ref layer, .. } if layer == "DATA" => {
                    println!("💾 Exécution Data Agent...");
                    run_agent(DataAgent::new(), &ctx, &intent, *execute).await;
                }

                // 7. TRANSVERSE / IVVQ
                EngineeringIntent::CreateElement { ref layer, .. } if layer == "TRANSVERSE" => {
                    println!("✨ Exécution Transverse Agent...");
                    run_agent(TransverseAgent::new(), &ctx, &intent, *execute).await;
                }

                // NON GÉRÉ
                EngineeringIntent::CreateRelationship { .. } => {
                    println!("\n🚧 Intention détectée : Création de Relation (WIP)");
                }
                EngineeringIntent::Chat => {
                    println!("\n💬 Mode DISCUSSION (Pas d'action technique)");
                }
                EngineeringIntent::Unknown => {
                    println!("\n❓ INTENTION INCONNUE");
                }
                _ => {
                    println!("\n⚠️ Cas non géré par le CLI.");
                }
            }
        }
    }

    Ok(())
}

/// Helper pour exécuter un agent de manière uniforme
async fn run_agent<A: Agent>(
    agent: A,
    ctx: &AgentContext,
    intent: &EngineeringIntent,
    execute: bool,
) {
    if execute {
        match agent.process(ctx, intent).await {
            Ok(Some(res)) => println!("\n✅ SUCCÈS :\n{}", res),
            Ok(None) => println!("ℹ️ Ignoré (Pas de résultat)."),
            Err(e) => eprintln!("❌ ÉCHEC : {}", e),
        }
    } else {
        println!("\n(Mode Dry Run - Utilisez -x pour exécuter réellement)");
    }
}
