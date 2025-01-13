use anyhow::{Context, Result};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Row, Sqlite};
use std::{fs::File, path::Path};

use crate::item::Item;

const DATABASE_PATH: &str = "./database.db";

#[derive(sqlx::FromRow, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DbItem {
    pub name: String,
    pub pfs: String,
    pub source: String,
    pub rarity: String,
    pub r#trait: String,
    pub item_category: String,
    pub item_subcategory: String,
    pub level: i32,
    pub price: String,
    pub bulk: String,
    pub usage: String,
    pub spoilers: String,
}

#[derive(sqlx::FromRow, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CategoryColumn {
    pub item_category: String,
}

pub async fn init_db() -> Result<Pool<Sqlite>> {
    let mut do_populate = false;
    let pool = match Path::new(DATABASE_PATH).exists() {
        true => {
            SqlitePoolOptions::new()
                .max_connections(1)
                .connect(&format!("sqlite://{}", DATABASE_PATH))
                .await
        }
        false => {
            do_populate = true;
            let _ = File::create(DATABASE_PATH);
            SqlitePoolOptions::new()
                .max_connections(1)
                .connect(&format!("sqlite://{}", DATABASE_PATH))
                .await
        }
    }
    .context("Failed to connect to database object")?;

    ensure_tables(&pool).await?;
    if do_populate {
        let res = populate_tables(&pool).await;
        if res.is_err() {
            std::fs::remove_file(DATABASE_PATH).unwrap();
            tracing::error!("{res:?}");
            std::process::exit(1);
        }
    }

    Ok(pool)
}

async fn ensure_tables(pool: &Pool<Sqlite>) -> Result<()> {
    sqlx::query(
        "
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS equipment(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT,
            pfs TEXT,
            source TEXT,
            rarity TEXT,
            trait TEXT,
            item_category TEXT,
            item_subcategory TEXT,
            level INTEGER,
            price TEXT,
            bulk TEXT,
            usage TEXT,
            spoilers TEXT
        );
        ",
    )
    .execute(pool)
    .await
    .context("Failed to ensure tables")?;

    Ok(())
}

async fn populate_tables(pool: &Pool<Sqlite>) -> Result<()> {
    let mut rdr = csv::Reader::from_path("assets/equipment.csv")?;
    for row in rdr.deserialize() {
        let row: DbItem = row?;

        sqlx::query("INSERT INTO equipment (name, pfs, source, rarity, trait, item_category, item_subcategory, level, price, bulk, usage, spoilers)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12);")
            .bind(row.name)
            .bind(row.pfs)
            .bind(row.source)
            .bind(row.rarity)
            .bind(row.r#trait)
            .bind(row.item_category)
            .bind(row.item_subcategory)
            .bind(row.level)
            .bind(row.price)
            .bind(row.bulk)
            .bind(row.usage)
            .bind(row.spoilers)
            .execute(pool)
            .await
            .context("Failed to insert row into table")?;
    }

    Ok(())
}

/// Get all items for a given category
/// Category must match string exactly as it appears on AoN
pub async fn get_category<S: AsRef<str>>(
    pool: &Pool<Sqlite>,
    category: S,
    ignore_priceless: bool,
) -> Result<Vec<Item>> {
    let priceless_filter = if ignore_priceless {
        "AND price IS NOT NULL AND price != \"\""
    } else {
        ""
    };

    let q = format!(
        "
    SELECT * FROM equipment
    WHERE item_category = $1 
    {}
    ;",
        priceless_filter
    );
    let results = sqlx::query_as::<_, DbItem>(&q)
        .bind(category.as_ref())
        .fetch_all(pool)
        .await
        .context("Failed to retrieve category from db")?;

    Ok(results.into_iter().map(|c| c.into()).collect())
}

/// Lists the available options in a column
#[allow(unused)]
pub async fn get_distinct<S: AsRef<str>>(
    pool: &Pool<Sqlite>,
    column: S,
) -> Result<Vec<CategoryColumn>> {
    let results = sqlx::query_as::<_, CategoryColumn>(
        "
    SELECT DISTINCT item_category FROM equipment;",
    )
    // .bind(column.as_ref())
    .fetch_all(pool)
    .await
    .context("Failed to retrieve info from db")?;

    Ok(results.into_iter().map(|c| c.into()).collect())
}
