//! Query history tracking and suggestions
//!
//! Stores search queries and provides suggestions based on past searches.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;

const MAX_HISTORY_SIZE: usize = 100;

/// A single search query entry in history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryEntry {
    /// The search query
    pub query: String,
    /// Timestamp (Unix timestamp)
    pub timestamp: u64,
    /// Number of results found
    pub result_count: usize,
    /// Filters used (optional)
    pub filters: Option<String>,
}

/// Query history manager
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryHistory {
    /// Recent queries (most recent last)
    queries: VecDeque<QueryEntry>,
    /// Path to history file
    #[serde(skip)]
    history_path: PathBuf,
}

impl QueryHistory {
    /// Create or load query history
    pub fn load(index_dir: &PathBuf) -> Result<Self> {
        let history_path = index_dir.join("history.json");

        if history_path.exists() {
            let content = std::fs::read_to_string(&history_path)?;
            let mut history: QueryHistory = serde_json::from_str(&content)?;
            history.history_path = history_path;
            Ok(history)
        } else {
            Ok(Self {
                queries: VecDeque::with_capacity(MAX_HISTORY_SIZE),
                history_path,
            })
        }
    }

    /// Add a query to history
    pub fn add_query(
        &mut self,
        query: String,
        result_count: usize,
        filters: Option<String>,
    ) -> Result<()> {
        let entry = QueryEntry {
            query: query.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            result_count,
            filters,
        };

        // Don't add duplicate consecutive queries
        if let Some(last) = self.queries.back() {
            if last.query == query {
                return Ok(());
            }
        }

        self.queries.push_back(entry);

        // Keep only last MAX_HISTORY_SIZE entries
        while self.queries.len() > MAX_HISTORY_SIZE {
            self.queries.pop_front();
        }

        self.save()
    }

    /// Save history to disk
    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&self.history_path, json)?;
        Ok(())
    }

    /// Get recent queries (most recent first)
    pub fn recent(&self, limit: usize) -> Vec<&QueryEntry> {
        self.queries
            .iter()
            .rev()
            .take(limit)
            .collect()
    }

    /// Get all queries
    pub fn all(&self) -> Vec<&QueryEntry> {
        self.queries.iter().rev().collect()
    }

    /// Get query suggestions based on partial input
    pub fn suggest(&self, partial: &str, limit: usize) -> Vec<String> {
        let partial_lower = partial.to_lowercase();
        let mut suggestions: Vec<String> = self
            .queries
            .iter()
            .rev()
            .filter(|e| e.query.to_lowercase().contains(&partial_lower))
            .map(|e| e.query.clone())
            .collect();

        // Remove duplicates while preserving order
        suggestions.dedup();
        suggestions.truncate(limit);
        suggestions
    }

    /// Get most frequent queries
    pub fn top_queries(&self, limit: usize) -> Vec<(String, usize)> {
        use std::collections::HashMap;

        let mut frequency: HashMap<String, usize> = HashMap::new();
        for entry in &self.queries {
            *frequency.entry(entry.query.clone()).or_insert(0) += 1;
        }

        let mut queries: Vec<(String, usize)> = frequency.into_iter().collect();
        queries.sort_by(|a, b| b.1.cmp(&a.1));
        queries.truncate(limit);
        queries
    }

    /// Clear all history
    pub fn clear(&mut self) -> Result<()> {
        self.queries.clear();
        self.save()
    }

    /// Get total number of queries
    pub fn len(&self) -> usize {
        self.queries.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.queries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_add_query() {
        let dir = tempdir().unwrap();
        let mut history = QueryHistory::load(&dir.path().to_path_buf()).unwrap();

        history.add_query("test query".to_string(), 5, None).unwrap();
        assert_eq!(history.len(), 1);

        // Duplicate consecutive queries not added
        history.add_query("test query".to_string(), 3, None).unwrap();
        assert_eq!(history.len(), 1);

        // Different query added
        history.add_query("another query".to_string(), 2, None).unwrap();
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_recent_queries() {
        let dir = tempdir().unwrap();
        let mut history = QueryHistory::load(&dir.path().to_path_buf()).unwrap();

        history.add_query("query 1".to_string(), 5, None).unwrap();
        history.add_query("query 2".to_string(), 3, None).unwrap();
        history.add_query("query 3".to_string(), 7, None).unwrap();

        let recent = history.recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].query, "query 3");
        assert_eq!(recent[1].query, "query 2");
    }

    #[test]
    fn test_suggestions() {
        let dir = tempdir().unwrap();
        let mut history = QueryHistory::load(&dir.path().to_path_buf()).unwrap();

        history.add_query("authentication".to_string(), 5, None).unwrap();
        history.add_query("authorization".to_string(), 3, None).unwrap();
        history.add_query("database".to_string(), 2, None).unwrap();

        let suggestions = history.suggest("auth", 10);
        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.contains(&"authorization".to_string()));
        assert!(suggestions.contains(&"authentication".to_string()));
    }

    #[test]
    fn test_top_queries() {
        let dir = tempdir().unwrap();
        let mut history = QueryHistory::load(&dir.path().to_path_buf()).unwrap();

        history.add_query("common query".to_string(), 5, None).unwrap();
        history.add_query("rare query".to_string(), 3, None).unwrap();
        history.add_query("another query".to_string(), 1, None).unwrap();
        history.add_query("common query".to_string(), 2, None).unwrap();
        history.add_query("rare query".to_string(), 7, None).unwrap();
        history.add_query("unique query".to_string(), 4, None).unwrap();
        history.add_query("common query".to_string(), 6, None).unwrap();

        let top = history.top_queries(5);
        
        // Find common query (should appear 3 times)
        let common = top.iter().find(|(q, _)| q == "common query").unwrap();
        assert_eq!(common.1, 3);
        
        // Should have multiple entries
        assert!(top.len() >= 3);
    }

    #[test]
    fn test_max_history_size() {
        let dir = tempdir().unwrap();
        let mut history = QueryHistory::load(&dir.path().to_path_buf()).unwrap();

        // Add more than MAX_HISTORY_SIZE queries
        for i in 0..150 {
            history.add_query(format!("query {}", i), 1, None).unwrap();
        }

        assert!(history.len() <= MAX_HISTORY_SIZE);
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();

        {
            let mut history = QueryHistory::load(&path).unwrap();
            history.add_query("test query".to_string(), 5, None).unwrap();
            history.save().unwrap();
        }

        // Load in new instance
        let history = QueryHistory::load(&path).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history.recent(1)[0].query, "test query");
    }
}