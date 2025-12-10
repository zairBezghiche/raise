use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::AppHandle;
use tokio::time::{sleep, Duration}; // Pour simuler le temps de calcul

// Paramètres envoyés par le Frontend
#[derive(Debug, Deserialize)]
pub struct GeneticsParams {
    pub population_size: u32,
    pub generations: u32,
    pub mutation_rate: f32,
}

// Résultat renvoyé au Frontend
#[derive(Debug, Serialize)]
pub struct OptimizationResult {
    pub best_score: f32,
    pub duration_ms: u64,
    pub improvement_log: Vec<f32>, // Pour tracer le graphique de convergence
    pub best_candidate_id: String,
}

#[tauri::command]
pub async fn run_genetic_optimization(
    _app: AppHandle,
    params: GeneticsParams,
    _model: Value,
) -> Result<OptimizationResult, String> {
    println!("🧬 Démarrage Optimisation Génétique : {:?}", params);

    // --- SIMULATION DU MOTEUR GÉNÉTIQUE ---
    // Ici, vous connecterez plus tard votre vrai module 'genetics::engine'
    // Pour l'instant, on simule une courbe de convergence logarithmique.

    let start_time = std::time::Instant::now();
    let mut log = Vec::new();
    let mut current_score = 50.0; // Score initial bas

    // On simule le passage des générations
    for i in 0..params.generations {
        // Amélioration progressive (simulation)
        let improvement = 100.0 / (i as f32 + 5.0);
        current_score += improvement * params.mutation_rate;

        if current_score > 99.9 {
            current_score = 99.9;
        }

        // On ne log que tous les 10% pour ne pas surcharger
        if i % (params.generations / 10).max(1) == 0 {
            log.push(current_score);
            // Petit délai pour simuler le calcul CPU sans bloquer l'async
            sleep(Duration::from_millis(10)).await;
        }
    }

    // Ajout du score final
    log.push(current_score);

    Ok(OptimizationResult {
        best_score: (current_score * 100.0).round() / 100.0,
        duration_ms: start_time.elapsed().as_millis() as u64,
        improvement_log: log,
        best_candidate_id: format!("ARCH-OPT-{:04}", rand::random::<u16>()),
    })
}
