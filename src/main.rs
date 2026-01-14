//! lgrep CLI - Local semantic grep
//!
//! A 100% offline semantic code search tool.

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use lgrep::{
    format_results, format_results_json, Config, EmbeddingModel, IndexWatcher, Indexer, Searcher,
    VectorIndex,
};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "lgrep")]
#[command(author, version, about = "Local semantic grep - 100% offline semantic code search", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Search query (when no subcommand is provided)
    #[arg(trailing_var_arg = true)]
    query: Vec<String>,

    /// Maximum number of results
    #[arg(short = 'm', long, default_value = "10", env = "LGREP_MAX_COUNT")]
    max_count: usize,

    /// Show content of results
    #[arg(short = 'c', long, env = "LGREP_CONTENT")]
    content: bool,

    /// Output as JSON
    #[arg(long, env = "LGREP_JSON")]
    json: bool,

    /// Sync index before searching
    #[arg(short = 's', long, env = "LGREP_SYNC")]
    sync: bool,

    /// Path to search in
    #[arg(short = 'p', long, default_value = ".")]
    path: PathBuf,

    /// Embedding model to use
    #[arg(long, default_value = "minilm", env = "LGREP_MODEL")]
    model: String,

    /// Enable verbose logging
    #[arg(short = 'v', long)]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Build or rebuild the search index
    Index {
        /// Path to index
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Embedding model to use
        #[arg(long, default_value = "minilm")]
        model: String,

        /// Force rebuild even if index exists
        #[arg(short, long)]
        force: bool,
    },

    /// Watch for file changes and update index automatically
    Watch {
        /// Path to watch
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Embedding model to use
        #[arg(long, default_value = "minilm")]
        model: String,
    },

    /// Search the index
    Search {
        /// Search query
        query: String,

        /// Path to search in
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Maximum number of results
        #[arg(short = 'm', long, default_value = "10")]
        max_count: usize,

        /// Show content of results
        #[arg(short = 'c', long)]
        content: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Sync index before searching
        #[arg(short = 's', long)]
        sync: bool,
    },

    /// Show index statistics
    Stats {
        /// Path to index
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// List available embedding models
    Models,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("warn")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    match cli.command {
        Some(Commands::Index { path, model, force }) => cmd_index(path, model, force),
        Some(Commands::Watch { path, model }) => cmd_watch(path, model),
        Some(Commands::Search {
            query,
            path,
            max_count,
            content,
            json,
            sync,
        }) => cmd_search(query, path, max_count, content, json, sync),
        Some(Commands::Stats { path }) => cmd_stats(path),
        Some(Commands::Models) => cmd_models(),
        None => {
            // Default: search if query provided, otherwise show help
            if cli.query.is_empty() {
                println!("{}", "lgrep - Local semantic grep".cyan().bold());
                println!("\nUsage:");
                println!("  {} \"your search query\"", "lgrep".green());
                println!(
                    "  {} \"query\" -c              # show content",
                    "lgrep".green()
                );
                println!(
                    "  {} index .                  # build index first",
                    "lgrep".green()
                );
                println!(
                    "  {} watch .                  # watch for changes",
                    "lgrep".green()
                );
                println!("\nRun {} for more options.", "lgrep --help".yellow());
                return Ok(());
            }

            let query = cli.query.join(" ");
            cmd_search(
                query,
                cli.path,
                cli.max_count,
                cli.content,
                cli.json,
                cli.sync,
            )
        }
    }
}

fn cmd_index(path: PathBuf, model: String, force: bool) -> Result<()> {
    let path = path.canonicalize()?;
    println!("{} {:?}", "Indexing".cyan().bold(), path);

    let model: EmbeddingModel = model.parse()?;
    let config = Config::new(path.clone()).with_model(model);

    if !force && config.index_path().exists() {
        println!("Index already exists. Updating...");
        let indexer = Indexer::new(config.clone())?;
        let mut index = VectorIndex::load(config)?;
        let stats = indexer.update_index(&mut index)?;
        println!("\n{} {}", "✓".green(), stats);
    } else {
        let indexer = Indexer::new(config)?;
        let index = indexer.build_index()?;
        println!(
            "\n{} Indexed {} files, {} chunks",
            "✓".green(),
            index.file_count(),
            index.chunk_count()
        );
    }

    Ok(())
}

fn cmd_watch(path: PathBuf, model: String) -> Result<()> {
    let path = path.canonicalize()?;
    println!("{} {:?}", "Watching".cyan().bold(), path);

    let model: EmbeddingModel = model.parse()?;
    let config = Config::new(path).with_model(model);

    let mut watcher = IndexWatcher::new(config)?;
    watcher.watch()?;

    Ok(())
}

fn cmd_search(
    query: String,
    path: PathBuf,
    max_count: usize,
    content: bool,
    json: bool,
    sync: bool,
) -> Result<()> {
    let path = path.canonicalize()?;

    // Check if index exists
    let index_dir = path.join(".lgrep");
    if !index_dir.exists() {
        eprintln!(
            "{} No index found. Run {} first.",
            "Error:".red().bold(),
            "lgrep index".yellow()
        );
        std::process::exit(1);
    }

    // Sync if requested
    if sync {
        let config = Config::load(&index_dir)?;
        let indexer = Indexer::new(config.clone())?;
        let mut index = VectorIndex::load(config)?;
        let stats = indexer.update_index(&mut index)?;
        if stats.added > 0 || stats.updated > 0 || stats.removed > 0 {
            eprintln!("Synced: {}", stats);
        }
    }

    // Search
    let searcher = Searcher::load(&path)?;
    let results = searcher.search(&query, max_count)?;

    if results.is_empty() {
        println!("No results found for: {}", query.yellow());
        return Ok(());
    }

    // Output results
    if json {
        println!("{}", format_results_json(&results)?);
    } else {
        println!(
            "\n{} results for \"{}\":\n",
            results.len().to_string().green().bold(),
            query.cyan()
        );
        print!("{}", format_results(&results, content, &path));
    }

    Ok(())
}

fn cmd_stats(path: PathBuf) -> Result<()> {
    let path = path.canonicalize()?;
    let searcher = Searcher::load(&path)?;
    let stats = searcher.stats();

    println!("{}", "Index Statistics".cyan().bold());
    println!("  Files:  {}", stats.files.to_string().green());
    println!("  Chunks: {}", stats.chunks.to_string().green());
    println!("  Model:  {}", stats.model.yellow());

    Ok(())
}

fn cmd_models() -> Result<()> {
    println!("{}", "Available Embedding Models".cyan().bold());
    println!();
    println!("  {} (default)", "minilm".green().bold());
    println!("    Fast, lightweight model (384 dims, ~30MB)");
    println!("    Best for: Quick indexing, smaller codebases");
    println!();
    println!("  {}", "bge".green().bold());
    println!("    High quality retrieval model (384 dims, ~90MB)");
    println!("    Best for: Better semantic understanding");
    println!();
    println!("  {}", "nomic".green().bold());
    println!("    Optimized for code and technical content (768 dims, ~90MB)");
    println!("    Best for: Code search, technical documentation");
    println!();
    println!("  {}", "multilingual".green().bold());
    println!("    Supports 100+ languages (384 dims, ~470MB)");
    println!("    Best for: Multi-language codebases");
    println!();
    println!("Usage: {} --model nomic", "lgrep index".yellow());

    Ok(())
}
