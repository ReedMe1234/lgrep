//! Text chunking for semantic search
//!
//! Splits source files into overlapping chunks suitable for embedding.
//! Preserves line number information for search result display.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// A chunk of text with metadata for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// Unique identifier for this chunk
    pub id: u64,
    /// The text content
    pub text: String,
    /// Source file path (relative to index root)
    pub file_path: String,
    /// Starting line number (1-indexed)
    pub start_line: usize,
    /// Ending line number (1-indexed)
    pub end_line: usize,
    /// SHA-256 hash of source file for change detection
    pub file_hash: String,
    /// Programming language hint for syntax highlighting
    pub language: Option<String>,
}

/// Metadata for all indexed chunks
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IndexMetadata {
    /// All chunks in the index
    pub chunks: Vec<Chunk>,
    /// Map of file paths to their hashes for change detection
    pub file_hashes: std::collections::HashMap<String, String>,
    /// Next chunk ID to assign
    pub next_id: u64,
    /// Model name used to create embeddings
    pub model_name: String,
    /// Embedding vector dimension
    pub dimension: usize,
}

impl IndexMetadata {
    /// Create new metadata for an index
    pub fn new(model_name: String, dimension: usize) -> Self {
        Self {
            model_name,
            dimension,
            ..Default::default()
        }
    }
}

/// Splits text into overlapping chunks
pub struct Chunker {
    chunk_size: usize,
    overlap: usize,
}

impl Chunker {
    /// Create a new chunker with specified sizes
    ///
    /// # Arguments
    /// * `chunk_size` - Target size for each chunk in characters
    /// * `overlap` - Number of characters to overlap between chunks
    pub fn new(chunk_size: usize, overlap: usize) -> Self {
        Self { chunk_size, overlap }
    }

    /// Split text into chunks with metadata
    ///
    /// Chunks are split on line boundaries to preserve code structure.
    /// Overlapping ensures context isn't lost at chunk boundaries.
    pub fn chunk_text(
        &self,
        text: &str,
        file_path: &str,
        file_hash: &str,
        start_id: u64,
    ) -> Vec<Chunk> {
        let language = detect_language(file_path);
        let lines: Vec<&str> = text.lines().collect();

        if lines.is_empty() {
            return vec![];
        }

        let mut chunks = Vec::new();
        let mut current_chunk_lines: Vec<&str> = Vec::new();
        let mut current_size = 0;
        let mut chunk_start_line = 1;
        let mut chunk_id = start_id;

        for (i, line) in lines.iter().enumerate() {
            let line_len = line.len() + 1; // +1 for newline

            // If adding this line exceeds chunk size, finalize current chunk
            if current_size + line_len > self.chunk_size && !current_chunk_lines.is_empty() {
                let chunk_text = current_chunk_lines.join("\n");
                let end_line = chunk_start_line + current_chunk_lines.len() - 1;

                chunks.push(Chunk {
                    id: chunk_id,
                    text: chunk_text,
                    file_path: file_path.to_string(),
                    start_line: chunk_start_line,
                    end_line,
                    file_hash: file_hash.to_string(),
                    language: language.clone(),
                });
                chunk_id += 1;

                // Keep some lines for overlap/context
                let overlap_lines = self.calculate_overlap_lines(&current_chunk_lines);
                let keep_count = overlap_lines.min(current_chunk_lines.len());

                if keep_count > 0 {
                    let start_idx = current_chunk_lines.len() - keep_count;
                    current_chunk_lines = current_chunk_lines[start_idx..].to_vec();
                    current_size = current_chunk_lines.iter().map(|l| l.len() + 1).sum();
                    chunk_start_line = i + 1 - keep_count + 1;
                } else {
                    current_chunk_lines.clear();
                    current_size = 0;
                    chunk_start_line = i + 2;
                }
            }

            current_chunk_lines.push(line);
            current_size += line_len;
        }

        // Don't forget the last chunk
        if !current_chunk_lines.is_empty() {
            let chunk_text = current_chunk_lines.join("\n");
            let end_line = chunk_start_line + current_chunk_lines.len() - 1;

            chunks.push(Chunk {
                id: chunk_id,
                text: chunk_text,
                file_path: file_path.to_string(),
                start_line: chunk_start_line,
                end_line,
                file_hash: file_hash.to_string(),
                language: language.clone(),
            });
        }

        chunks
    }

    /// Calculate how many lines to keep for overlap
    fn calculate_overlap_lines(&self, lines: &[&str]) -> usize {
        let mut size = 0;
        let mut count = 0;

        for line in lines.iter().rev() {
            size += line.len() + 1;
            if size > self.overlap {
                break;
            }
            count += 1;
        }

        count.max(1) // Keep at least 1 line for context
    }
}

/// Detect programming language from file extension
fn detect_language(file_path: &str) -> Option<String> {
    let path = Path::new(file_path);
    let ext = path.extension()?.to_str()?;

    let lang = match ext.to_lowercase().as_str() {
        "rs" => "rust",
        "py" | "pyi" | "pyw" => "python",
        "js" | "mjs" | "cjs" => "javascript",
        "ts" | "mts" | "cts" => "typescript",
        "jsx" => "javascriptreact",
        "tsx" => "typescriptreact",
        "go" => "go",
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "c" | "h" => "c",
        "cpp" | "hpp" | "cc" | "cxx" | "hxx" => "cpp",
        "cs" => "csharp",
        "rb" | "rake" => "ruby",
        "php" => "php",
        "swift" => "swift",
        "scala" | "sc" => "scala",
        "sh" | "bash" | "zsh" => "shell",
        "sql" => "sql",
        "html" | "htm" => "html",
        "css" => "css",
        "scss" | "sass" => "scss",
        "vue" => "vue",
        "svelte" => "svelte",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "md" | "mdx" => "markdown",
        "tf" | "hcl" => "terraform",
        "xml" => "xml",
        _ => return None,
    };

    Some(lang.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunking_small_file() {
        let chunker = Chunker::new(100, 20);
        let text = "line 1\nline 2\nline 3";
        let chunks = chunker.chunk_text(text, "test.py", "abc123", 0);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].start_line, 1);
        assert_eq!(chunks[0].end_line, 3);
    }

    #[test]
    fn test_chunking_large_file() {
        let chunker = Chunker::new(50, 10);
        let text = "line 1 with some content\nline 2 with more content\nline 3 with even more\nline 4 final";
        let chunks = chunker.chunk_text(text, "test.rs", "def456", 0);

        assert!(chunks.len() > 1);
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.id, i as u64);
        }
    }

    #[test]
    fn test_language_detection() {
        assert_eq!(detect_language("main.rs"), Some("rust".to_string()));
        assert_eq!(detect_language("app.py"), Some("python".to_string()));
        assert_eq!(detect_language("index.tsx"), Some("typescriptreact".to_string()));
        assert_eq!(detect_language("unknown.xyz"), None);
    }

    #[test]
    fn test_empty_file() {
        let chunker = Chunker::new(100, 20);
        let chunks = chunker.chunk_text("", "empty.rs", "hash", 0);
        assert!(chunks.is_empty());
    }
}
