mod save;
mod fill;
mod transfer;
mod bruteforce;
mod stats;
mod util;
mod images;

use tokio;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    program: Option<Program>,
    /// Adds a random delay to each cdn request
    #[arg(long, short)]
    delay: bool,
    #[arg(long, short)]
    gay: bool,
}

#[derive(Subcommand)]
enum Program {
    /// Saves all of the images you have into the `all_images.txt` file.
    Save,
    /// Fills all of the missing full images and thumbnails inside of the mail folder.
    Fill,
    /// Sends all of thumbnail or full images to a specific folder
    Transfer {
        #[arg(short, long)]
        full_folder: Option<PathBuf>,
        #[arg(short, long)]
        thumb_folder: Option<PathBuf>,
    },
    /// Bruteforces an image link.
    Bruteforce {
        /// Example input: "\d/\a\_", \d inserts [0-9], \a inserts [a-z], \_ inserts [a-z0-9_]
        /// Instead of `0/amb_00_5ca_thumb_170.png`, insert `0/amb_00_5ca` or `0/amb_00_5ca_thumb`
        input: Option<String>,
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Gives you some stats
    Stats,
}

#[tokio::main]
async fn main() {
    let cli_parsed = Cli::parse();

    match cli_parsed.program {
        None | Some(Program::Save) => save::save(cli_parsed.gay),
        Some(Program::Fill) => fill::fill(cli_parsed.delay, cli_parsed.gay).await,
        Some(Program::Bruteforce { input, file }) => {
            if let Some(input) = input { 
                bruteforce::bruteforce(&input, cli_parsed.gay).await
            }
            if let Some(file) = file {
                for line in std::fs::read_to_string(file).unwrap().lines() {
                    bruteforce::bruteforce(line.trim(), cli_parsed.gay).await
                }
            }
        },
        Some(Program::Transfer { full_folder, thumb_folder }) => {
            transfer::transfer(full_folder, thumb_folder, cli_parsed.gay)
        },
        Some(Program::Stats) => stats::stats(cli_parsed.gay),
    }
}

