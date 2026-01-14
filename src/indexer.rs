//! File discovery and indexing
//!
//! Walks directories respecting .gitignore, chunks files,
//! generates embeddings, and builds the search index.

use crate::chunker::Chunker;
use crate::config::{should_index_file, Config};
use crate::embedder::Embedder;
use crate::error::Result;
use crate::index::VectorIndex;
use ignore::WalkBuilder;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

/// File to be indexed with content and hash
#[derive(Debug)]
struct FileToIndex {
    #[allow(dead_code)]
    path: PathBuf,
    relative_path: String,
    content: String,
    hash: String,
}

/// Indexer for building and updating the semantic index
pub struct Indexer {
    config: Config,
    embedder: Embedder,
    chunker: Chunker,
}

impl Indexer {
    /// Create a new indexer with the given configuration
    pub fn new(config: Config) -> Result<Self> {
        let embedder = Embedder::new(&config.model)?;
        let chunker = Chunker::new(config.chunk_size, config.chunk_overlap);

        Ok(Self {
            config,
            embedder,
            chunker,
        })
    }

    /// Build a fresh index from scratch
    pub fn build_index(&self) -> Result<VectorIndex> {
        info!("Building fresh index for {:?}", self.config.root_path);

        let mut index = VectorIndex::new(self.config.clone())?;
        let files = self.discover_files()?;

        if files.is_empty() {
            info!("No files to index");
            return Ok(index);
        }

        self.index_files(&mut index, files)?;
        index.save()?;

        Ok(index)
    }

    /// Update an existing index (incremental)
    pub fn update_index(&self, index: &mut VectorIndex) -> Result<UpdateStats> {
        info!("Updating index for {:?}", self.config.root_path);

        let files = self.discover_files()?;
        let mut stats = UpdateStats::default();

        // Find files that need updating
        let mut files_to_add: Vec<FileToIndex> = Vec::new();
        let mut files_to_remove: HashSet<String> = index
            .indexed_files()
            .iter()
            .map(|s| s.to_string())
            .collect();

        for file in files {
            files_to_remove.remove(&file.relative_path);

            // Check if file has changed
            if let Some(existing_hash) = index.get_file_hash(&file.relative_path) {
                if existing_hash == &file.hash {
                    debug!("Skipping unchanged file: {}", file.relative_path);
                    stats.unchanged += 1;
                    continue;
                }
                // File changed - remove old chunks
                index.remove_file(&file.relative_path)?;
                stats.updated += 1;
            } else {
                stats.added += 1;
            }

            files_to_add.push(file);
        }

        // Remove deleted files
        for file_path in &files_to_remove {
            index.remove_file(file_path)?;
            stats.removed += 1;
        }

        // Index new/changed files
        if !files_to_add.is_empty() {
            self.index_files(index, files_to_add)?;
        }

        index.save()?;

        Ok(stats)
    }

    /// Discover all indexable files in the root directory
    fn discover_files(&self) -> Result<Vec<FileToIndex>> {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message("Discovering files...");

        let root = self.config.root_path.canonicalize()?;
        let files = Arc::new(Mutex::new(Vec::new()));

        // Use ignore crate to respect .gitignore
        let walker = WalkBuilder::new(&root)
            .hidden(true)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .ignore(true)
            .parents(true)
            .add_custom_ignore_filename(".lgrepignore")
            .build();

        // Collect file paths first
        let file_paths: Vec<PathBuf> = walker
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.file_type().map(|ft| ft.is_file()).unwrap_or(false)
                    && should_index_file(entry.path())
            })
            .filter(|entry| {
                entry
                    .metadata()
                    .map(|m| m.len() <= self.config.max_file_size)
                    .unwrap_or(false)
            })
            .map(|entry| entry.path().to_path_buf())
            .collect();

        pb.set_message(format!("Found {} files, reading...", file_paths.len()));

        // Read files in parallel
        let root_clone = root.clone();

        file_paths.par_iter().for_each(|path| {
            if let Ok(content) = std::fs::read_to_string(path) {
                let relative_path = path
                    .strip_prefix(&root_clone)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();

                let hash = compute_hash(&content);

                let file = FileToIndex {
                    path: path.clone(),
                    relative_path,
                    content,
                    hash,
                };

                files.lock().unwrap().push(file);
            }
        });

        pb.finish_with_message("File discovery complete");

        let result = Arc::try_unwrap(files).unwrap().into_inner().unwrap();
        info!("Discovered {} indexable files", result.len());

        Ok(result)
    }

    /// Index a list of files
    fn index_files(&self, index: &mut VectorIndex, files: Vec<FileToIndex>) -> Result<()> {
        if files.is_empty() {
            return Ok(());
        }

        // Create chunks from all files
        let pb = ProgressBar::new(files.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} files chunked")
                .unwrap()
                .progress_chars("=>-"),
        );

        let mut all_chunks = Vec::new();
        let mut next_id = index.next_id();

        for file in &files {
            let chunks =
                self.chunker
                    .chunk_text(&file.content, &file.relative_path, &file.hash, next_id);

            next_id += chunks.len() as u64;
            all_chunks.extend(chunks);
            pb.inc(1);
        }

        pb.finish_with_message(format!(
            "Created {} chunks from {} files",
            all_chunks.len(),
            files.len()
        ));

        if all_chunks.is_empty() {
            return Ok(());
        }

        // Generate embeddings
        let pb = ProgressBar::new(all_chunks.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} chunks embedded")
                .unwrap()
                .progress_chars("=>-"),
        );

        let texts: Vec<String> = all_chunks.iter().map(|c| c.text.clone()).collect();
        let batch_size = 32;

        let embeddings =
            self.embedder
                .embed_batch_with_progress(texts, batch_size, |done, _total| {
                    pb.set_position(done as u64);
                })?;

        pb.finish_with_message("Embeddings generated");

        // Add to index
        info!("Adding {} chunks to index", all_chunks.len());
        index.add_chunks(all_chunks, embeddings)?;

        Ok(())
    }
}

/// Statistics for index updates
#[derive(Debug, Default)]
pub struct UpdateStats {
    /// Number of new files added
    pub added: usize,
    /// Number of files updated (changed content)
    pub updated: usize,
    /// Number of files removed (deleted)
    pub removed: usize,
    /// Number of unchanged files
    pub unchanged: usize,
}

impl std::fmt::Display for UpdateStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Added: {}, Updated: {}, Removed: {}, Unchanged: {}",
            self.added, self.updated, self.removed, self.unchanged
        )
    }
}

/// Compute SHA-256 hash of content
fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}
