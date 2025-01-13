#![allow(unused)]
#![allow(dead_code)]

mod database;
mod item;
mod merchant;

use item::ItemKind;
use merchant::Merchant;

// load the csv file
// create a database populated with the same data

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();
    #[cfg(not(debug_assertions))]
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .init();

    let pool = database::init_db().await.unwrap_or_else(|e| {
        tracing::error!("An error occurred: {}", e);
        std::process::exit(1);
    });

    let merchant = Merchant::new(123);
    merchant.save();
}
