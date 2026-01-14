//! Search filters for metadata-based filtering
//!
//! Allows filtering search results by file type, language, path patterns, etc.

use crate::chunker::Chunk;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Search filter criteria
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilter {
    /// Filter by file extensions (e.g., ["rs", "py"])
    pub extensions: Option<Vec<String>>,
    /// Filter by programming languages (e.g., ["rust", "python"])
    pub languages: Option<Vec<String>>,
    /// Filter by file path patterns (regex)
    pub path_pattern: Option<String>,
    /// Exclude file path patterns (regex)
    pub exclude_pattern: Option<String>,
    /// Minimum similarity score (0.0 to 1.0)
    pub min_score: Option<f32>,
    /// Maximum results to return
    pub max_results: Option<usize>,
}

impl SearchFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Set file extensions filter
    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = Some(extensions);
        self
    }

    /// Set languages filter
    pub fn with_languages(mut self, languages: Vec<String>) -> Self {
        self.languages = Some(languages);
        self
    }

    /// Set path pattern filter
    pub fn with_path_pattern(mut self, pattern: String) -> Self {
        self.path_pattern = Some(pattern);
        self
    }

    /// Set exclude pattern filter
    pub fn with_exclude_pattern(mut self, pattern: String) -> Self {
        self.exclude_pattern = Some(pattern);
        self
    }

    /// Set minimum score threshold
    pub fn with_min_score(mut self, score: f32) -> Self {
        self.min_score = Some(score);
        self
    }

    /// Set maximum results
    pub fn with_max_results(mut self, max: usize) -> Self {
        self.max_results = Some(max);
        self
    }

    /// Check if a chunk matches the filter criteria
    pub fn matches(&self, chunk: &Chunk, score: f32) -> bool {
        // Check minimum score
        if let Some(min_score) = self.min_score {
            if score < min_score {
                return false;
            }
        }

        // Check file extension
        if let Some(ref extensions) = self.extensions {
            let file_ext = std::path::Path::new(&chunk.file_path)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());

            match file_ext {
                Some(ext) => {
                    if !extensions.iter().any(|e| e.to_lowercase() == ext) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // Check language
        if let Some(ref languages) = self.languages {
            match &chunk.language {
                Some(lang) => {
                    if !languages.iter().any(|l| l.to_lowercase() == lang.to_lowercase()) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // Check path pattern
        if let Some(ref pattern) = self.path_pattern {
            if let Ok(regex) = Regex::new(pattern) {
                if !regex.is_match(&chunk.file_path) {
                    return false;
                }
            }
        }

        // Check exclude pattern
        if let Some(ref pattern) = self.exclude_pattern {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(&chunk.file_path) {
                    return false;
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_chunk(file_path: &str, language: Option<String>) -> Chunk {
        Chunk {
            id: 0,
            text: "test content".to_string(),
            file_path: file_path.to_string(),
            start_line: 1,
            end_line: 1,
            file_hash: "hash".to_string(),
            language,
        }
    }

    #[test]
    fn test_extension_filter() {
        let filter = SearchFilter::new().with_extensions(vec!["rs".to_string(), "py".to_string()]);

        let chunk_rs = create_test_chunk("src/main.rs", Some("rust".to_string()));
        let chunk_js = create_test_chunk("src/main.js", Some("javascript".to_string()));

        assert!(filter.matches(&chunk_rs, 0.8));
        assert!(!filter.matches(&chunk_js, 0.8));
    }

    #[test]
    fn test_language_filter() {
        let filter = SearchFilter::new().with_languages(vec!["rust".to_string()]);

        let chunk_rs = create_test_chunk("src/main.rs", Some("rust".to_string()));
        let chunk_py = create_test_chunk("app.py", Some("python".to_string()));

        assert!(filter.matches(&chunk_rs, 0.8));
        assert!(!filter.matches(&chunk_py, 0.8));
    }

    #[test]
    fn test_min_score_filter() {
        let filter = SearchFilter::new().with_min_score(0.7);

        let chunk = create_test_chunk("test.rs", Some("rust".to_string()));

        assert!(filter.matches(&chunk, 0.8));
        assert!(!filter.matches(&chunk, 0.6));
    }

    #[test]
    fn test_path_pattern_filter() {
        let filter = SearchFilter::new().with_path_pattern("src/.*".to_string());

        let chunk_src = create_test_chunk("src/main.rs", Some("rust".to_string()));
        let chunk_test = create_test_chunk("tests/test.rs", Some("rust".to_string()));

        assert!(filter.matches(&chunk_src, 0.8));
        assert!(!filter.matches(&chunk_test, 0.8));
    }

    #[test]
    fn test_exclude_pattern_filter() {
        let filter = SearchFilter::new().with_exclude_pattern("test.*".to_string());

        let chunk_src = create_test_chunk("src/main.rs", Some("rust".to_string()));
        let chunk_test = create_test_chunk("tests/test.rs", Some("rust".to_string()));

        assert!(filter.matches(&chunk_src, 0.8));
        assert!(!filter.matches(&chunk_test, 0.8));
    }

    #[test]
    fn test_combined_filters() {
        let filter = SearchFilter::new()
            .with_extensions(vec!["rs".to_string()])
            .with_languages(vec!["rust".to_string()])
            .with_min_score(0.7);

        let chunk_match = create_test_chunk("src/main.rs", Some("rust".to_string()));
        let chunk_wrong_ext = create_test_chunk("src/main.py", Some("python".to_string()));
        let chunk_wrong_lang = create_test_chunk("src/main.rs", Some("javascript".to_string()));

        assert!(filter.matches(&chunk_match, 0.8));
        assert!(!filter.matches(&chunk_wrong_ext, 0.8));
        assert!(!filter.matches(&chunk_wrong_lang, 0.8));
        assert!(!filter.matches(&chunk_match, 0.6)); // Low score
    }
}