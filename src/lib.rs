//! # lgrep - Local semantic grep
//!
//! A fully local, privacy-preserving semantic code search tool.
//! No API calls, no cloud dependencies - everything runs on your machine.
//!
//! ## Features
//!
//! - **100% Local**: All processing happens on your machine using ONNX runtime
//! - **Semantic Search**: Find code by meaning, not just keywords
//! - **Git-Aware**: Respects .gitignore and supports incremental updates
//! - **Fast**: Uses HNSW algorithm for efficient nearest neighbor search
//! - **Watch Mode**: Automatically updates index as files change
//!
//! ## Example
//!
//! ```no_run
//! use lgrep::{Config, Indexer, Searcher};
//! use std::path::PathBuf;
//!
//! fn main() -> anyhow::Result<()> {
//!     // Build index
//!     let config = Config::new(PathBuf::from("."));
//!     let indexer = Indexer::new(config.clone())?;
//!     let index = indexer.build_index()?;
//!
//!     // Search
//!     let searcher = Searcher::from_index(index)?;
//!     let results = searcher.search("authentication handler", 10)?;
//!
//!     for result in results {
//!         println!("{}:{} (score: {:.2})",
//!             result.chunk.file_path,
//!             result.chunk.start_line,
//!             result.score
//!         );
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod chunker;
pub mod config;
pub mod embedder;
pub mod error;
pub mod index;
pub mod indexer;
pub mod searcher;
pub mod watcher;

// Re-export commonly used types
pub use chunker::{Chunk, Chunker, IndexMetadata};
pub use config::{Config, EmbeddingModel};
pub use embedder::Embedder;
pub use error::{LgrepError, Result};
pub use index::{SearchResult, VectorIndex};
pub use indexer::{Indexer, UpdateStats};
pub use searcher::{format_results, format_results_json, IndexStats, Searcher};
pub use watcher::IndexWatcher;
