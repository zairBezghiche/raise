pub mod candle;
pub mod fast;

use anyhow::Result;

pub enum EngineType {
    FastEmbed,
    Candle,
}

pub struct EmbeddingEngine {
    inner: EngineImplementation,
}

enum EngineImplementation {
    Fast(fast::FastEmbedEngine),
    Candle(candle::CandleEngine),
}

impl EmbeddingEngine {
    pub fn new() -> Result<Self> {
        Self::new_with_type(EngineType::FastEmbed)
    }

    pub fn new_with_type(engine_type: EngineType) -> Result<Self> {
        let inner = match engine_type {
            EngineType::FastEmbed => {
                println!("🧠 Init NLP Engine: FastEmbed (ONNX)");
                EngineImplementation::Fast(fast::FastEmbedEngine::new()?)
            }
            EngineType::Candle => {
                println!("🕯️ Init NLP Engine: Candle (BERT Pure Rust)");
                EngineImplementation::Candle(candle::CandleEngine::new()?)
            }
        };
        Ok(Self { inner })
    }

    // CORRECTION : Passage en &mut self pour propager la mutabilité
    pub fn embed_batch(&mut self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        match &mut self.inner {
            // Note le &mut ici aussi
            EngineImplementation::Fast(e) => e.embed_batch(texts),
            EngineImplementation::Candle(e) => e.embed_batch(texts),
        }
    }

    // CORRECTION : Passage en &mut self
    pub fn embed_query(&mut self, text: &str) -> Result<Vec<f32>> {
        match &mut self.inner {
            EngineImplementation::Fast(e) => e.embed_query(text),
            EngineImplementation::Candle(e) => e.embed_query(text),
        }
    }
}
