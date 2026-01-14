//! Local embedding generation using fastembed (ONNX runtime)
//!
//! Generates embeddings entirely locally - no API calls required.
//! Models are downloaded once and cached in ~/.cache/huggingface/

use crate::config::EmbeddingModel;
use crate::error::{LgrepError, Result};
use fastembed::{EmbeddingModel as FastEmbedModel, InitOptions, TextEmbedding};
use std::sync::Arc;
use tracing::info;

/// Local embedder using fastembed with ONNX runtime
pub struct Embedder {
    model: Arc<TextEmbedding>,
    dimension: usize,
}

impl Embedder {
    /// Create a new embedder with the specified model
    ///
    /// On first use, downloads the model from HuggingFace (~30-470MB).
    /// Subsequent uses load from cache instantly.
    pub fn new(model_config: &EmbeddingModel) -> Result<Self> {
        info!("Loading embedding model: {:?}", model_config);

        let fastembed_model = match model_config {
            EmbeddingModel::AllMiniLmL6V2 => FastEmbedModel::AllMiniLML6V2,
            EmbeddingModel::BgeSmallEnV15 => FastEmbedModel::BGESmallENV15,
            EmbeddingModel::NomicEmbedTextV15 => FastEmbedModel::NomicEmbedTextV15,
            EmbeddingModel::MultilingualE5Small => FastEmbedModel::MultilingualE5Small,
        };

        let model = TextEmbedding::try_new(
            InitOptions::new(fastembed_model).with_show_download_progress(true),
        )
        .map_err(|e| LgrepError::Embedding(e.to_string()))?;

        let dimension = model_config.dimension();

        info!("Model loaded successfully (dimension: {})", dimension);

        Ok(Self {
            model: Arc::new(model),
            dimension,
        })
    }

    /// Get the embedding dimension
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Embed a single text string
    pub fn embed_one(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self
            .model
            .embed(vec![text], None)
            .map_err(|e| LgrepError::Embedding(e.to_string()))?;

        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| LgrepError::Embedding("No embedding returned".to_string()))
    }

    /// Embed multiple texts in a single batch (more efficient)
    pub fn embed_batch(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        self.model
            .embed(texts, None)
            .map_err(|e| LgrepError::Embedding(e.to_string()))
    }

    /// Embed texts with progress callback for large batches
    pub fn embed_batch_with_progress<F>(
        &self,
        texts: Vec<String>,
        batch_size: usize,
        mut progress: F,
    ) -> Result<Vec<Vec<f32>>>
    where
        F: FnMut(usize, usize),
    {
        let total = texts.len();
        let mut all_embeddings = Vec::with_capacity(total);

        for (i, batch) in texts.chunks(batch_size).enumerate() {
            let batch_refs: Vec<&str> = batch.iter().map(|s| s.as_str()).collect();
            let embeddings = self.embed_batch(batch_refs)?;
            all_embeddings.extend(embeddings);

            let done = ((i + 1) * batch_size).min(total);
            progress(done, total);
        }

        Ok(all_embeddings)
    }
}

/// Normalize embedding vector to unit length (for cosine similarity)
#[allow(dead_code)]
pub fn normalize(embedding: &mut [f32]) {
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in embedding.iter_mut() {
            *x /= norm;
        }
    }
}

/// Compute cosine similarity between two normalized vectors
#[allow(dead_code)]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        let mut v = vec![3.0, 4.0];
        normalize(&mut v);
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 0.0];
        let b = vec![1.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let c = vec![0.0, 1.0];
        assert!(cosine_similarity(&a, &c).abs() < 1e-6);
    }
}
