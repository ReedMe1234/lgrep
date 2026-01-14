//! File system watcher for live index updates
//!
//! Watches for file changes and automatically updates the index.
//! Uses debouncing to avoid excessive updates on rapid changes.

use crate::config::{should_index_file, Config};
use crate::error::{LgrepError, Result};
use crate::index::VectorIndex;
use crate::indexer::Indexer;
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEvent};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, info, warn};

/// File system watcher for live index updates
pub struct IndexWatcher {
    config: Config,
    indexer: Indexer,
    index: Arc<Mutex<VectorIndex>>,
}

impl IndexWatcher {
    /// Create a new watcher for the given configuration
    pub fn new(config: Config) -> Result<Self> {
        let indexer = Indexer::new(config.clone())?;

        // Try to load existing index or build new one
        let index = match VectorIndex::load(config.clone()) {
            Ok(idx) => {
                info!("Loaded existing index");
                idx
            }
            Err(_) => {
                info!("Building new index...");
                indexer.build_index()?
            }
        };

        Ok(Self {
            config,
            indexer,
            index: Arc::new(Mutex::new(index)),
        })
    }

    /// Start watching for file changes
    ///
    /// This blocks until interrupted (Ctrl+C).
    pub fn watch(&mut self) -> Result<()> {
        let root = self.config.root_path.canonicalize()?;
        info!("Watching {:?} for changes...", root);

        // First do an incremental update
        {
            let mut index = self.index.lock().unwrap();
            let stats = self.indexer.update_index(&mut index)?;
            info!("Initial sync: {}", stats);
        }

        // Set up file watcher with debouncing
        let (tx, rx) = channel();

        let mut debouncer = new_debouncer(Duration::from_millis(500), tx)
            .map_err(|e| LgrepError::Watch(e.to_string()))?;

        debouncer
            .watcher()
            .watch(&root, RecursiveMode::Recursive)
            .map_err(|e| LgrepError::Watch(e.to_string()))?;

        println!(
            "\n✓ Index ready ({} files, {} chunks)",
            self.index.lock().unwrap().file_count(),
            self.index.lock().unwrap().chunk_count()
        );
        println!("  Watching for changes. Press Ctrl+C to stop.\n");

        // Process events
        self.process_events(rx, &root)?;

        Ok(())
    }

    /// Process file system events from the watcher
    fn process_events(
        &mut self,
        rx: Receiver<std::result::Result<Vec<DebouncedEvent>, notify::Error>>,
        root: &Path,
    ) -> Result<()> {
        loop {
            match rx.recv() {
                Ok(Ok(events)) => {
                    let mut changed_files: HashSet<PathBuf> = HashSet::new();

                    for event in events {
                        let path = &event.path;

                        // Skip non-indexable files
                        if !should_index_file(path) {
                            continue;
                        }

                        // Skip files in .lgrep directory
                        if path.starts_with(root.join(".lgrep")) {
                            continue;
                        }

                        changed_files.insert(path.clone());
                    }

                    if !changed_files.is_empty() {
                        self.handle_changes(changed_files)?;
                    }
                }
                Ok(Err(e)) => {
                    warn!("Watch error: {:?}", e);
                }
                Err(e) => {
                    debug!("Watch channel closed: {:?}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle detected file changes
    fn handle_changes(&mut self, changed_files: HashSet<PathBuf>) -> Result<()> {
        info!("Processing {} changed files...", changed_files.len());

        let mut index = self.index.lock().unwrap();
        let stats = self.indexer.update_index(&mut index)?;

        if stats.added > 0 || stats.updated > 0 || stats.removed > 0 {
            println!(
                "  Updated: +{} ~{} -{} (total: {} chunks)",
                stats.added,
                stats.updated,
                stats.removed,
                index.chunk_count()
            );
        }

        Ok(())
    }

    /// Get the current index for searching
    #[allow(dead_code)]
    pub fn index(&self) -> Arc<Mutex<VectorIndex>> {
        Arc::clone(&self.index)
    }
}
