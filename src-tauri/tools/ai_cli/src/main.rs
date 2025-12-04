use anyhow::Result;
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use std::env;
use std::path::PathBuf;

use genaptitude::ai::agents::intent_classifier::{EngineeringIntent, IntentClassifier};
use genaptitude::ai::agents::{system_agent::SystemAgent, Agent};
use genaptitude::ai::llm::client::{LlmBackend, LlmClient};
use genaptitude::json_db::storage::{JsonDbConfig, StorageEngine};

#[derive(Parser)]
#[command(
    name = "ai_cli",
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
    dotenv().ok();

    let gemini_key = env::var("GENAPTITUDE_GEMINI_KEY").unwrap_or_default();
    let model_name = env::var("GENAPTITUDE_MODEL_NAME").ok();
    let local_url =
        env::var("GENAPTITUDE_LOCAL_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let db_path_str =
        env::var("PATH_GENAPTITUDE_DOMAIN").unwrap_or_else(|_| "./genaptitude_db".to_string());

    // Initialisation Moteur
    let config = JsonDbConfig::new(PathBuf::from(db_path_str));
    let storage = StorageEngine::new(config);

    // PLUS AUCUNE LOGIQUE DE SEEDING ICI !
    // C'est StorageEngine::new (ou plutôt les appels sous-jacents à create_db) qui s'en chargera.

    let client = LlmClient::new(&local_url, &gemini_key, model_name);
    let args = Cli::parse();

    match args.command {
        Commands::Chat { message, cloud } => {
            let backend = if cloud {
                LlmBackend::GoogleGemini
            } else {
                LlmBackend::LocalLlama
            };
            let mode = if cloud { "CLOUD" } else { "LOCAL" };
            println!("🤖 [{}] : \"{}\"", mode, message);

            match client.ask(backend, "Assistant CLI.", &message).await {
                Ok(res) => println!("\n✅ :\n{}", res),
                Err(e) => eprintln!("❌ : {}", e),
            }
        }

        Commands::Classify { input, execute } => {
            println!("🧠 Analyse : \"{}\"", input);
            let classifier = IntentClassifier::new(client.clone());
            let intent = classifier.classify(&input).await;

            match intent {
                EngineeringIntent::CreateElement {
                    ref layer,
                    ref element_type,
                    ref name,
                } => {
                    println!("🔧 CRÉATION : {} {} ({})", layer, element_type, name);

                    if execute {
                        println!("⚡ Exécution...");
                        let agent = SystemAgent::new(client.clone(), storage);
                        match agent.process(&intent).await {
                            Ok(Some(res)) => println!("\n✅ SUCCÈS :\n{}", res),
                            Ok(None) => println!("ℹ️ Ignoré."),
                            Err(e) => eprintln!("❌ ÉCHEC : {}", e),
                        }
                    } else {
                        println!("(Dry Run -> -x pour exécuter)");
                    }
                }
                _ => println!("ℹ️ Autre intention"),
            }
        }
    }

    Ok(())
}
