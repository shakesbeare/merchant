use anyhow::{Context, Result};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Row, Sqlite};
use std::{collections::HashMap, fs::File, path::Path};

use crate::item::{Item, ItemCategory, Rarity};

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

pub async fn get_all(
    pool: &Pool<Sqlite>,
    rarity: Rarity,
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
    WHERE rarity = $1
    {}
    ;",
        priceless_filter
    );
    let results = sqlx::query_as::<_, DbItem>(&q)
        .bind(rarity.as_ref())
        .fetch_all(pool)
        .await
        .context("Failed to retrieve category from db")?;

    Ok(results.into_iter().map(|c| c.into()).collect())
}

/// Get all items for a given category
/// Category must match string exactly as it appears on AoN
pub async fn get_category(
    pool: &Pool<Sqlite>,
    category: ItemCategory,
    rarity: Rarity,
    level: i32,
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
    AND rarity = $2
    AND level < $3
    {}
    ;",
        priceless_filter
    );
    let results = sqlx::query_as::<_, DbItem>(&q)
        .bind(category.as_ref())
        .bind(rarity.as_ref())
        .bind(level)
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

    Ok(results)
}

pub async fn get_rations(pool: &Pool<Sqlite>) -> Item {
    let result = sqlx::query_as::<_, DbItem>(
        "
        SELECT * FROM equipment 
        WHERE name = $1;
        ",
    )
    .bind("Rations")
    .fetch_one(pool)
    .await
    .context("Rations should exist.")
    .unwrap();

    result.into()
}

pub async fn get_min_for_each_category(pool: &Pool<Sqlite>, level: i32) -> Result<HashMap<ItemCategory, i32>> {
    let mut out = HashMap::new();
    for category in enum_iterator::all::<ItemCategory>() {
        let items = get_category(pool, category, Rarity::Common, level, true).await?;
        let min = items
            .into_iter()
            .reduce(|a, b| {
                let lhs_price = a.price.as_ref().unwrap().as_cp();
                let rhs_price = b.price.as_ref().unwrap().as_cp();
                if lhs_price < rhs_price {
                    a
                } else {
                    b
                }
            })
            .map(|i| i.price.as_ref().unwrap().as_cp());
        if let Some(min) = min {
            out.insert(category, min);
        }
    }
    Ok(out)
}
