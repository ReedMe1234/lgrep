//! Search functionality and result formatting
//!
//! Provides semantic search over the index and formats results
//! for terminal display or JSON output.

use crate::config::Config;
use crate::embedder::Embedder;
use crate::error::Result;
use crate::index::{SearchResult, VectorIndex};
use colored::*;
use std::path::Path;

/// Semantic searcher
pub struct Searcher {
    index: VectorIndex,
    embedder: Embedder,
}

impl Searcher {
    /// Create a new searcher by loading an existing index
    pub fn load(root_path: &Path) -> Result<Self> {
        let index_dir = root_path.join(".lgrep");
        let config = Config::load(&index_dir)?;
        let index = VectorIndex::load(config.clone())?;
        let embedder = Embedder::new(&config.model)?;

        Ok(Self { index, embedder })
    }

    /// Create a searcher from an existing index
    pub fn from_index(index: VectorIndex) -> Result<Self> {
        let embedder = Embedder::new(&index.config().model)?;
        Ok(Self { index, embedder })
    }

    /// Search for chunks matching the query
    pub fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>> {
        let query_embedding = self.embedder.embed_one(query)?;
        self.index.search(&query_embedding, top_k)
    }

    /// Get index statistics
    pub fn stats(&self) -> IndexStats {
        IndexStats {
            files: self.index.file_count(),
            chunks: self.index.chunk_count(),
            model: self.index.config().model.model_name().to_string(),
        }
    }
}

/// Index statistics
pub struct IndexStats {
    /// Number of indexed files
    pub files: usize,
    /// Number of chunks
    pub chunks: usize,
    /// Model name used for embeddings
    pub model: String,
}

/// Format search results for terminal display
pub fn format_results(results: &[SearchResult], show_content: bool, _root_path: &Path) -> String {
    let mut output = String::new();

    for (i, result) in results.iter().enumerate() {
        // File path and line range
        let file_display = format!(
            "{}:{}",
            result.chunk.file_path,
            if result.chunk.start_line == result.chunk.end_line {
                format!("{}", result.chunk.start_line)
            } else {
                format!("{}-{}", result.chunk.start_line, result.chunk.end_line)
            }
        );

        // Score indicator with color
        let score_pct = (result.score * 100.0) as u32;
        let score_color = if score_pct >= 80 {
            "green"
        } else if score_pct >= 60 {
            "yellow"
        } else {
            "red"
        };

        output.push_str(&format!(
            "\n{} {} ({}%)\n",
            format!("[{}]", i + 1).dimmed(),
            file_display.cyan().bold(),
            format!("{}", score_pct).color(score_color)
        ));

        if show_content {
            output.push_str(&format!("{}\n", "─".repeat(60).dimmed()));

            // Show content with line numbers
            let lines: Vec<&str> = result.chunk.text.lines().collect();
            let max_lines = 15;
            let show_lines = if lines.len() > max_lines {
                &lines[..max_lines]
            } else {
                &lines
            };

            for (j, line) in show_lines.iter().enumerate() {
                let line_num = result.chunk.start_line + j;
                output.push_str(&format!("{} {}\n", format!("{:4}", line_num).dimmed(), line));
            }

            if lines.len() > max_lines {
                output.push_str(&format!(
                    "{}\n",
                    format!("     ... ({} more lines)", lines.len() - max_lines).dimmed()
                ));
            }
        }
    }

    output
}

/// Format results as JSON
pub fn format_results_json(results: &[SearchResult]) -> Result<String> {
    use serde::Serialize;

    #[derive(Serialize)]
    struct JsonResult {
        file: String,
        start_line: usize,
        end_line: usize,
        score: f32,
        content: String,
        language: Option<String>,
    }

    let json_results: Vec<JsonResult> = results
        .iter()
        .map(|r| JsonResult {
            file: r.chunk.file_path.clone(),
            start_line: r.chunk.start_line,
            end_line: r.chunk.end_line,
            score: r.score,
            content: r.chunk.text.clone(),
            language: r.chunk.language.clone(),
        })
        .collect();

    Ok(serde_json::to_string_pretty(&json_results)?)
}
