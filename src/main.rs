mod index;
mod models;
mod parser;
mod search;
mod server;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;
use tracing_subscriber;

use crate::index::IndexManager;
use crate::models::{Language, SearchMode};
use crate::search::SearchEngine;

#[derive(Parser)]
#[command(name = "dictv")]
#[command(about = "German-English Dictionary Server", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Import dictionary from FreeDict or local files
    Import {
        /// Download from FreeDict (freedict-eng-deu or freedict-deu-eng)
        #[arg(long)]
        download: Option<String>,

        /// Local dictionary file path (.dict.dz)
        #[arg(long, requires = "index")]
        local: Option<String>,

        /// Local index file path (.index)
        #[arg(long, requires = "local")]
        index: Option<String>,

        /// Language direction (en-de or de-en)
        #[arg(long, default_value = "de-en")]
        lang: String,
    },

    /// Rebuild the search index from all dictionary files
    Rebuild,

    /// Show index statistics
    Stats,

    /// Start the HTTP server
    Serve {
        /// Run as daemon in background
        #[arg(long)]
        daemon: bool,

        /// Port to listen on
        #[arg(long, default_value = "3000")]
        port: u16,
    },

    /// Query the dictionary directly
    Query {
        /// Search query
        query: String,

        /// Search mode (exact, fuzzy, prefix)
        #[arg(long, default_value = "fuzzy")]
        mode: String,

        /// Language direction (en-de or de-en)
        #[arg(long, default_value = "de-en")]
        lang: String,

        /// Maximum edit distance for fuzzy search
        #[arg(long, default_value = "2")]
        max_distance: u8,

        /// Maximum number of results
        #[arg(long, default_value = "10")]
        limit: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Import {
            download,
            local,
            index,
            lang,
        } => {
            let manager = IndexManager::default()?;

            // Show data directory location
            let home = dirs::home_dir().unwrap_or_default();
            let data_dir = home.join(".dictv");
            println!("📁 Data directory: {}", data_dir.display());
            println!("   - Dictionaries: {}/data", data_dir.display());
            println!("   - Search index: {}/index\n", data_dir.display());

            if let Some(dict_name) = download {
                info!("Downloading dictionary: {}", dict_name);
                manager.import_freedict(&dict_name)?;
                println!("✓ Successfully imported {}", dict_name);
            } else if let (Some(dict_path), Some(index_path)) = (local, index) {
                info!("Importing local dictionary: {}", dict_path);
                manager.import_local(&dict_path, &index_path, &lang)?;
                println!("✓ Successfully imported dictionary");
            } else {
                eprintln!("Error: Either --download or both --local and --index must be provided");
                std::process::exit(1);
            }
        }

        Commands::Rebuild => {
            let manager = IndexManager::default()?;

            let home = dirs::home_dir().unwrap_or_default();
            let data_dir = home.join(".dictv");
            println!("📁 Data directory: {}", data_dir.display());

            info!("Rebuilding index...");
            manager.rebuild()?;
            println!("✓ Index rebuilt successfully");
        }

        Commands::Stats => {
            let manager = IndexManager::default()?;
            let (total, en_de, de_en, size) = manager.stats()?;

            let home = dirs::home_dir().unwrap_or_default();
            let data_dir = home.join(".dictv");

            println!("📊 Dictionary Statistics:");
            println!("  Data directory: {}", data_dir.display());
            println!("  Total entries: {}", total);
            println!("  English → German: {}", en_de);
            println!("  German → English: {}", de_en);
            println!("  Index size: {} MB", size / 1_000_000);
        }

        Commands::Serve { daemon, port } => {
            if daemon {
                println!("Daemon mode not yet implemented");
                std::process::exit(1);
            }

            let manager = IndexManager::default()?;
            let engine = SearchEngine::new(manager.index_dir())?;

            let home = dirs::home_dir().unwrap_or_default();
            let data_dir = home.join(".dictv");
            println!("📁 Using data directory: {}", data_dir.display());
            println!("🚀 Starting server on http://localhost:{}", port);
            server::serve(engine, port).await?;
        }

        Commands::Query {
            query,
            mode,
            lang,
            max_distance,
            limit,
        } => {
            let manager = IndexManager::default()?;
            let engine = SearchEngine::new(manager.index_dir())?;

            let search_mode: SearchMode = mode.parse()?;
            let language: Language = lang.parse()?;

            let results = engine.search(&query, search_mode, language, max_distance, limit)?;

            if results.is_empty() {
                println!("No results found for '{}'", query);
            } else {
                println!("Results for '{}':\n", query);
                for result in results {
                    if let Some(distance) = result.edit_distance {
                        println!(
                            "• {} [distance: {}]: {}",
                            result.word, distance, result.definition
                        );
                    } else {
                        println!("• {}: {}", result.word, result.definition);
                    }
                }
            }
        }
    }

    Ok(())
}
