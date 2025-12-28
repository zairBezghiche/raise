use anyhow::{Context, Result};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

pub struct FastEmbedEngine {
    model: TextEmbedding,
}

impl FastEmbedEngine {
    pub fn new() -> Result<Self> {
        // CORRECTION : Utilisation de .with_show_download_progress(true)
        let options =
            InitOptions::new(EmbeddingModel::BGESmallENV15).with_show_download_progress(true);

        let model = TextEmbedding::try_new(options).context("❌ FastEmbed Init Failed")?;

        Ok(Self { model })
    }

    pub fn embed_batch(&mut self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        self.model.embed(texts, None)
    }

    pub fn embed_query(&mut self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.model.embed(vec![text.to_string()], None)?;
        embeddings
            .into_iter()
            .next()
            .context("No embedding generated")
    }
}
