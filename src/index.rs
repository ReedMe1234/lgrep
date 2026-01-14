//! Vector index using usearch (HNSW algorithm)
//!
//! Provides fast approximate nearest neighbor search for semantic queries.
//! Uses cosine similarity for comparing embeddings.

use crate::chunker::{Chunk, IndexMetadata};
use crate::config::Config;
use crate::error::{LgrepError, Result};
use tracing::{debug, info};
use usearch::{Index, IndexOptions, MetricKind, ScalarKind};

/// Vector index for semantic search
pub struct VectorIndex {
    index: Index,
    metadata: IndexMetadata,
    config: Config,
}

impl VectorIndex {
    /// Create a new empty index
    pub fn new(config: Config) -> Result<Self> {
        let dimension = config.model.dimension();

        let options = IndexOptions {
            dimensions: dimension,
            metric: MetricKind::Cos, // Cosine similarity
            quantization: ScalarKind::F32,
            connectivity: 16,       // M parameter for HNSW
            expansion_add: 128,     // ef_construction
            expansion_search: 64,   // ef
            multi: false,
        };

        let index = Index::new(&options).map_err(|e| LgrepError::Index(e.to_string()))?;

        let metadata = IndexMetadata::new(config.model.model_name().to_string(), dimension);

        Ok(Self {
            index,
            metadata,
            config,
        })
    }

    /// Load existing index from disk
    pub fn load(config: Config) -> Result<Self> {
        let index_path = config.index_path();
        let metadata_path = config.metadata_path();

        if !index_path.exists() || !metadata_path.exists() {
            return Err(LgrepError::NoIndex);
        }

        info!("Loading index from {:?}", index_path);

        // Load metadata first to get dimension
        let metadata_bytes = std::fs::read(&metadata_path)?;
        let metadata: IndexMetadata = bincode::deserialize(&metadata_bytes)?;

        // Create index with correct options
        let options = IndexOptions {
            dimensions: metadata.dimension,
            metric: MetricKind::Cos,
            quantization: ScalarKind::F32,
            connectivity: 16,
            expansion_add: 128,
            expansion_search: 64,
            multi: false,
        };

        let index = Index::new(&options).map_err(|e| LgrepError::Index(e.to_string()))?;

        // Load the index data
        index
            .load(index_path.to_str().unwrap())
            .map_err(|e| LgrepError::Index(e.to_string()))?;

        info!(
            "Loaded {} vectors, {} chunks",
            index.size(),
            metadata.chunks.len()
        );

        Ok(Self {
            index,
            metadata,
            config,
        })
    }

    /// Save index to disk
    pub fn save(&self) -> Result<()> {
        std::fs::create_dir_all(&self.config.index_dir)?;

        let index_path = self.config.index_path();
        let metadata_path = self.config.metadata_path();

        info!("Saving index to {:?}", index_path);

        // Save vector index
        self.index
            .save(index_path.to_str().unwrap())
            .map_err(|e| LgrepError::Index(e.to_string()))?;

        // Save metadata
        let metadata_bytes = bincode::serialize(&self.metadata)?;
        std::fs::write(&metadata_path, metadata_bytes)?;

        // Save config
        self.config.save()?;

        info!(
            "Saved {} vectors, {} chunks",
            self.index.size(),
            self.metadata.chunks.len()
        );

        Ok(())
    }

    /// Add chunks with their embeddings to the index
    pub fn add_chunks(&mut self, chunks: Vec<Chunk>, embeddings: Vec<Vec<f32>>) -> Result<()> {
        if chunks.len() != embeddings.len() {
            return Err(LgrepError::Index(
                "Chunks and embeddings count mismatch".to_string(),
            ));
        }

        // Reserve space
        let current_size = self.index.size();
        let new_size = current_size + chunks.len();
        self.index
            .reserve(new_size)
            .map_err(|e| LgrepError::Index(e.to_string()))?;

        // Add vectors
        for (chunk, embedding) in chunks.iter().zip(embeddings.iter()) {
            self.index
                .add(chunk.id, embedding)
                .map_err(|e| LgrepError::Index(e.to_string()))?;

            debug!("Added chunk {} from {}", chunk.id, chunk.file_path);
        }

        // Update metadata
        for chunk in chunks {
            let file_path = chunk.file_path.clone();
            let file_hash = chunk.file_hash.clone();
            self.metadata.chunks.push(chunk);
            self.metadata.file_hashes.insert(file_path, file_hash);
        }

        // Update next ID
        self.metadata.next_id = self
            .metadata
            .chunks
            .iter()
            .map(|c| c.id)
            .max()
            .unwrap_or(0)
            + 1;

        Ok(())
    }

    /// Remove all chunks from a specific file
    pub fn remove_file(&mut self, file_path: &str) -> Result<Vec<u64>> {
        let removed_ids: Vec<u64> = self
            .metadata
            .chunks
            .iter()
            .filter(|c| c.file_path == file_path)
            .map(|c| c.id)
            .collect();

        // Remove from index (ignore errors for missing keys)
        for id in &removed_ids {
            let _ = self.index.remove(*id);
        }

        // Remove from metadata
        self.metadata.chunks.retain(|c| c.file_path != file_path);
        self.metadata.file_hashes.remove(file_path);

        debug!("Removed {} chunks from {}", removed_ids.len(), file_path);

        Ok(removed_ids)
    }

    /// Search for similar chunks
    pub fn search(&self, query_embedding: &[f32], top_k: usize) -> Result<Vec<SearchResult>> {
        if self.index.size() == 0 {
            return Ok(vec![]);
        }

        let results = self
            .index
            .search(query_embedding, top_k)
            .map_err(|e| LgrepError::Index(e.to_string()))?;

        let mut search_results = Vec::new();

        for (key, distance) in results.keys.iter().zip(results.distances.iter()) {
            // Find the chunk with this ID
            if let Some(chunk) = self.metadata.chunks.iter().find(|c| c.id == *key) {
                // Convert distance to similarity score (cosine distance -> similarity)
                let score = 1.0 - distance;

                search_results.push(SearchResult {
                    chunk: chunk.clone(),
                    score,
                });
            }
        }

        // Sort by score descending
        search_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        Ok(search_results)
    }

    /// Get file hash if file is indexed
    pub fn get_file_hash(&self, file_path: &str) -> Option<&String> {
        self.metadata.file_hashes.get(file_path)
    }

    /// Get all indexed file paths
    pub fn indexed_files(&self) -> Vec<&String> {
        self.metadata.file_hashes.keys().collect()
    }

    /// Get total number of chunks
    pub fn chunk_count(&self) -> usize {
        self.metadata.chunks.len()
    }

    /// Get total number of indexed files
    pub fn file_count(&self) -> usize {
        self.metadata.file_hashes.len()
    }

    /// Get next chunk ID
    pub fn next_id(&self) -> u64 {
        self.metadata.next_id
    }

    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
}

/// Search result with chunk and similarity score
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The matched chunk
    pub chunk: Chunk,
    /// Similarity score (0.0 to 1.0, higher is better)
    pub score: f32,
}
