#![allow(unused)]
#![allow(dead_code)]

mod database;
mod item;
mod merchant;

use item::ItemCategory;
use merchant::Merchant;
use tracing_subscriber::EnvFilter;

// load the csv file
// create a database populated with the same data

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

    let mut merchant = Merchant::by_level(5);
    merchant.generate_inventory(&pool).await;
    println!("{}", &merchant);
    dbg!(merchant.len());
    dbg!(merchant.get_wealth_in_inv() / 100);
}
