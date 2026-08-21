use clap::{Parser, Subcommand};
use std::path::PathBuf;
use trailhead::Index;

const INDEX_FILE: &str = "trailhead.index.json";

#[derive(Parser)]
#[command(name = "trailhead", about = "A from-scratch full-text search engine")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build an index from a directory of .txt/.md files and save it.
    Index {
        dir: PathBuf,
    },
    /// Search the saved index and print the top N ranked results.
    Search {
        query: String,
        #[arg(long, default_value_t = 5)]
        top: usize,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Index { dir } => {
            let index = match Index::build_from_dir(&dir) {
                Ok(index) => index,
                Err(e) => {
                    eprintln!("failed to index {}: {}", dir.display(), e);
                    std::process::exit(1);
                }
            };
            if let Err(e) = index.save(&PathBuf::from(INDEX_FILE)) {
                eprintln!("failed to save index: {}", e);
                std::process::exit(1);
            }
            println!(
                "indexed {} document(s) with {} unique term(s), saved to {}",
                index.docs.len(),
                index.postings.len(),
                INDEX_FILE
            );
        }
        Command::Search { query, top } => {
            let index = match Index::load(&PathBuf::from(INDEX_FILE)) {
                Ok(index) => index,
                Err(e) => {
                    eprintln!(
                        "failed to load index ({}), run `trailhead index <dir>` first: {}",
                        INDEX_FILE, e
                    );
                    std::process::exit(1);
                }
            };
            let results = index.search(&query, top);
            if results.is_empty() {
                println!("no results for \"{}\"", query);
                return;
            }
            for (rank, (path, score)) in results.iter().enumerate() {
                println!("{}. {}  (score: {:.4})", rank + 1, path, score);
            }
        }
    }
}
