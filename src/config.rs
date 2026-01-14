//! Configuration types and constants for lgrep
//!
//! Defines embedding models, index configuration, and file filtering rules.

use crate::error::{LgrepError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Supported embedding models (all run locally via ONNX)
///
/// These models are downloaded on first use and cached locally.
/// No API keys or network access required after initial download.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum EmbeddingModel {
    /// Fast, small model (384 dims, ~30MB) - good for most use cases
    #[default]
    AllMiniLmL6V2,
    /// Higher quality (384 dims, ~90MB) - better semantic understanding
    BgeSmallEnV15,
    /// Best quality for code (768 dims, ~90MB)
    NomicEmbedTextV15,
    /// Multilingual support (384 dims, ~470MB)
    MultilingualE5Small,
}

impl EmbeddingModel {
    /// Get the HuggingFace model identifier
    pub fn model_name(&self) -> &'static str {
        match self {
            Self::AllMiniLmL6V2 => "sentence-transformers/all-MiniLM-L6-v2",
            Self::BgeSmallEnV15 => "BAAI/bge-small-en-v1.5",
            Self::NomicEmbedTextV15 => "nomic-ai/nomic-embed-text-v1.5",
            Self::MultilingualE5Small => "intfloat/multilingual-e5-small",
        }
    }

    /// Get the embedding vector dimension
    pub fn dimension(&self) -> usize {
        match self {
            Self::AllMiniLmL6V2 => 384,
            Self::BgeSmallEnV15 => 384,
            Self::NomicEmbedTextV15 => 768,
            Self::MultilingualE5Small => 384,
        }
    }
}

impl std::str::FromStr for EmbeddingModel {
    type Err = LgrepError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "minilm" | "all-minilm-l6-v2" | "default" => Ok(Self::AllMiniLmL6V2),
            "bge" | "bge-small" | "bge-small-en-v1.5" => Ok(Self::BgeSmallEnV15),
            "nomic" | "nomic-embed" | "nomic-embed-text-v1.5" => Ok(Self::NomicEmbedTextV15),
            "multilingual" | "e5" | "multilingual-e5-small" => Ok(Self::MultilingualE5Small),
            _ => Err(LgrepError::Config(format!(
                "Unknown model: {}. Valid options: minilm, bge, nomic, multilingual",
                s
            ))),
        }
    }
}

/// Configuration for lgrep indexing and search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Root directory being indexed
    pub root_path: PathBuf,
    /// Directory to store index data
    pub index_dir: PathBuf,
    /// Embedding model to use
    pub model: EmbeddingModel,
    /// Chunk size in characters
    pub chunk_size: usize,
    /// Overlap between chunks in characters
    pub chunk_overlap: usize,
    /// Maximum file size to index (bytes)
    pub max_file_size: u64,
    /// Number of parallel workers for processing
    pub workers: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            root_path: PathBuf::from("."),
            index_dir: PathBuf::from(".lgrep"),
            model: EmbeddingModel::default(),
            chunk_size: 512,
            chunk_overlap: 64,
            max_file_size: 10 * 1024 * 1024, // 10 MB
            workers: num_cpus::get(),
        }
    }
}

impl Config {
    /// Create a new config for the given root path
    pub fn new(root_path: PathBuf) -> Self {
        let index_dir = root_path.join(".lgrep");
        Self {
            root_path,
            index_dir,
            ..Default::default()
        }
    }

    /// Set the embedding model
    pub fn with_model(mut self, model: EmbeddingModel) -> Self {
        self.model = model;
        self
    }

    /// Set the chunk size
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Get path to the vector index file
    pub fn index_path(&self) -> PathBuf {
        self.index_dir.join("vectors.usearch")
    }

    /// Get path to the metadata file
    pub fn metadata_path(&self) -> PathBuf {
        self.index_dir.join("metadata.bin")
    }

    /// Get path to the config file
    pub fn config_path(&self) -> PathBuf {
        self.index_dir.join("config.json")
    }

    /// Save configuration to disk
    pub fn save(&self) -> Result<()> {
        std::fs::create_dir_all(&self.index_dir)?;
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(self.config_path(), json)?;
        Ok(())
    }

    /// Load configuration from disk
    pub fn load(index_dir: &PathBuf) -> Result<Self> {
        let config_path = index_dir.join("config.json");
        if !config_path.exists() {
            return Err(LgrepError::NoIndex);
        }
        let json = std::fs::read_to_string(config_path)?;
        let config: Config = serde_json::from_str(&json)?;
        Ok(config)
    }
}

/// File extensions that should be indexed
pub const CODE_EXTENSIONS: &[&str] = &[
    // Rust
    "rs",
    // Python
    "py", "pyi", "pyw",
    // JavaScript/TypeScript
    "js", "jsx", "ts", "tsx", "mjs", "cjs",
    // Go
    "go",
    // Java/Kotlin
    "java", "kt", "kts",
    // C/C++
    "c", "h", "cpp", "hpp", "cc", "cxx", "hxx",
    // C#
    "cs",
    // Ruby
    "rb", "rake",
    // PHP
    "php",
    // Swift
    "swift",
    // Scala
    "scala", "sc",
    // Shell
    "sh", "bash", "zsh", "fish",
    // SQL
    "sql",
    // Web
    "html", "htm", "css", "scss", "sass", "less", "vue", "svelte",
    // Config
    "json", "yaml", "yml", "toml", "ini", "cfg", "conf",
    // Documentation
    "md", "mdx", "rst", "txt",
    // Infrastructure
    "tf", "hcl",
    // Data
    "xml", "csv",
];

/// Check if a file should be indexed based on its extension
pub fn should_index_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| CODE_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_parsing() {
        let model: EmbeddingModel = "minilm".parse().unwrap();
        assert_eq!(model.dimension(), 384);

        let model: EmbeddingModel = "nomic".parse().unwrap();
        assert_eq!(model.dimension(), 768);

        assert!("invalid".parse::<EmbeddingModel>().is_err());
    }

    #[test]
    fn test_should_index_file() {
        use std::path::Path;

        assert!(should_index_file(Path::new("main.rs")));
        assert!(should_index_file(Path::new("app.py")));
        assert!(should_index_file(Path::new("index.tsx")));
        assert!(!should_index_file(Path::new("image.png")));
        assert!(!should_index_file(Path::new("binary.exe")));
    }

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        assert_eq!(config.chunk_size, 512);
        assert_eq!(config.chunk_overlap, 64);
        assert_eq!(config.max_file_size, 10 * 1024 * 1024);
    }
}
