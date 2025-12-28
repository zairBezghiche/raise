use anyhow::Result;
use candle_core::{DType, Device, Tensor}; // <-- CORRECTION 1 : DType vient d'ici
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config}; // On retire Dtype d'ici
use hf_hub::{api::sync::Api, Repo, RepoType};
use tokenizers::Tokenizer;

pub struct EmbeddingModel {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
}

impl EmbeddingModel {
    /// Initialise le modèle 'all-MiniLM-L6-v2'.
    pub fn new() -> Result<Self> {
        // 1. Choix du Device (CPU)
        let device = Device::Cpu;

        // 2. Récupération des fichiers sur HuggingFace
        let api = Api::new()?;
        let repo = api.repo(Repo::new(
            "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            RepoType::Model,
        ));

        let config_filename = repo.get("config.json")?;
        let tokenizer_filename = repo.get("tokenizer.json")?;
        let weights_filename = repo.get("model.safetensors")?;

        // 3. Chargement de la configuration
        let config_content = std::fs::read_to_string(config_filename)?;
        let config: Config = serde_json::from_str(&config_content)?;

        // 4. Chargement du Tokenizer
        let tokenizer = Tokenizer::from_file(tokenizer_filename)
            .map_err(|e| anyhow::anyhow!("Erreur chargement tokenizer: {}", e))?;

        // 5. Chargement des Poids
        // CORRECTION 1 : Utilisation de DType::F32 (importé de candle_core)
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_filename], DType::F32, &device)?
        };
        let model = BertModel::load(vb, &config)?;

        Ok(Self {
            model,
            tokenizer,
            device,
        })
    }

    /// Transforme un texte en vecteur (Embedding).
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // A. Tokenisation
        let tokens = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| anyhow::anyhow!("Erreur encodage: {}", e))?;

        let token_ids = Tensor::new(tokens.get_ids(), &self.device)?.unsqueeze(0)?;
        let token_type_ids = token_ids.zeros_like()?;

        // B. Inférence
        // CORRECTION 2 : Ajout de 'None' pour l'attention_mask (3ème argument)
        let embeddings = self.model.forward(&token_ids, &token_type_ids, None)?;

        // C. Mean Pooling
        let (_n_sentence, n_tokens, _hidden_size) = embeddings.dims3()?;
        let embeddings = (embeddings.sum(1)? / (n_tokens as f64))?;

        // D. Normalisation
        let embeddings = normalize_l2(&embeddings)?;

        // Conversion en Vec<f32>
        let vec = embeddings.squeeze(0)?.to_vec1::<f32>()?;
        Ok(vec)
    }
}

fn normalize_l2(v: &Tensor) -> Result<Tensor> {
    let sum_sq = v.sqr()?.sum_keepdim(1)?;
    let norm = sum_sq.sqrt()?;
    Ok((v.broadcast_div(&norm))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_generation() {
        println!("Téléchargement et chargement du modèle (peut prendre du temps la 1ère fois)...");
        let model = EmbeddingModel::new().expect("Échec chargement modèle Candle");

        let vec1 = model.embed("Le chat mange").unwrap();
        let vec2 = model.embed("Le félin se nourrit").unwrap();

        // Validation dimensionnelle
        assert_eq!(vec1.len(), 384);

        // Validation sémantique (Produit scalaire)
        let similarity: f32 = vec1.iter().zip(&vec2).map(|(a, b)| a * b).sum();
        println!("Similarité ('Chat' vs 'Félin') : {}", similarity);

        // Les phrases sont très proches, le score devrait être élevé (souvent > 0.6)
        assert!(similarity > 0.5, "La similarité sémantique est trop faible");
    }
}
