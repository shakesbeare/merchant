#![allow(unused)]
#![allow(dead_code)]

mod database;
mod item;
mod merchant;

use clap::{command, Parser};
use item::ItemCategory;
use merchant::Merchant;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(name = "merchant")]
#[command(version, about)]
struct Cli {
    #[clap(subcommand)]
    pub subcmd: Subcommand,
}

#[derive(Debug, Parser)]
enum Subcommand {
    #[clap(name = "gen")]
    /// Generate a new merchant inventory
    Generate {
        level: i32,
        /// Save the merchant to a .ron file
        #[arg(long = "save", short)]
        save: bool,
        /// Format Stdout as markdown
        #[arg(long = "markdown", short)]
        markdown: bool,
    },

    /// Load and display an existing merchant
    Load {
        filename: String,
    }
}

#[tokio::main]
async fn main() {
    let env_filter = EnvFilter::builder().parse_lossy("sqlx=warn");
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_env_filter(env_filter)
        .init();
    #[cfg(not(debug_assertions))]
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .init();

    let pool = database::init_db().await.unwrap_or_else(|e| {
        tracing::error!("An error occurred: {}", e);
        std::process::exit(1);
    });

    let cli = Cli::parse();

    match cli.subcmd {
        Subcommand::Generate { level, save, markdown } => {
            let mut merchant = Merchant::by_level(level);
            merchant.generate_inventory(&pool).await.unwrap();

            if save { 
                merchant.save();
            }

            if markdown {
                merchant.markdown();
            } else {
                println!("{}", merchant);
            }
        }
        Subcommand::Load{ filename } => {
            let merchant = Merchant::read_from_file(filename);
            println!("{}", merchant);
        }
    }
}
