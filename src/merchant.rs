use std::collections::HashMap;

use crate::{
    database,
    item::{Item, ItemCategory, Price, Rarity},
};
use anyhow::{Context, Result};
use enum_derived::Rand;
use rand::{seq::SliceRandom, Rng};
use sqlx::{Pool, Sqlite};

const MIN_ARMOR_COST: i32 = 20;
const MIN_CONSUMABLE_COST: i32 = 300;
const UNCOMMON_CHANCE: f32 = 0.005;
const RARE_CHANCE: f32 = 0.001;
const MERCHANT_WEALTH_DIVISOR: i32 = 3;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct Merchant {
    /// The merchant's wealth in cp
    wealth: i32,
    level: i32,
    inventory: Vec<Item>,
}

impl Merchant {
    pub fn new(cp: i32, level: i32) -> Self {
        Self {
            wealth: cp,
            level,
            inventory: vec![],
        }
    }

    pub fn from_gp(gp: i32, level: i32) -> Self {
        Self::new(gp * 100, level)
    }

    pub fn by_level(level: i32) -> Self {
        let gp = match level {
            // numbers taken from Treasure By Level table for players
            1 => 175 / MERCHANT_WEALTH_DIVISOR,
            2 => 300 / MERCHANT_WEALTH_DIVISOR,
            3 => 500 / MERCHANT_WEALTH_DIVISOR,
            4 => 850 / MERCHANT_WEALTH_DIVISOR,
            5 => 1350 / MERCHANT_WEALTH_DIVISOR,
            6 => 2000 / MERCHANT_WEALTH_DIVISOR,
            7 => 2900 / MERCHANT_WEALTH_DIVISOR,
            8 => 4000 / MERCHANT_WEALTH_DIVISOR,
            9 => 5700 / MERCHANT_WEALTH_DIVISOR,
            10 => 8000 / MERCHANT_WEALTH_DIVISOR,
            11 => 11500 / MERCHANT_WEALTH_DIVISOR,
            12 => 16500 / MERCHANT_WEALTH_DIVISOR,
            13 => 25000 / MERCHANT_WEALTH_DIVISOR,
            14 => 36500 / MERCHANT_WEALTH_DIVISOR,
            15 => 54500 / MERCHANT_WEALTH_DIVISOR,
            16 => 82500 / MERCHANT_WEALTH_DIVISOR,
            17 => 128000 / MERCHANT_WEALTH_DIVISOR,
            18 => 208000 / MERCHANT_WEALTH_DIVISOR,
            19 => 355000 / MERCHANT_WEALTH_DIVISOR,
            20 => 490000 / MERCHANT_WEALTH_DIVISOR,
            _ => unreachable!(),
        };
        Self::from_gp(gp, level)
    }

    pub fn read_from_file<S: AsRef<str>>(filename: S) -> Self {
        let ron = std::fs::read_to_string(filename.as_ref()).unwrap();
        ron::from_str(&ron).unwrap()
    }

    pub fn save(&self) -> Result<()> {
        let ron = ron::to_string(self)?;
        let filename = format!(
            "{}.ron",
            chrono::offset::Local::now().format("%Y-%m-%d_%I:%M %p")
        );
        std::fs::write(filename, ron)?;
        Ok(())
    }

    pub fn markdown(&self) {
        let s = self.to_string();
        // TODO: remove trailing ###
        let s = s.replace("----------", "###");
        println!("{}", s);
    }

    pub fn len(&self) -> usize {
        self.inventory.len()
    }

    pub async fn generate_inventory(&mut self, pool: &Pool<Sqlite>) -> Result<()> {
        let mut rations_allowance = self.wealth / 24;

        let rations = database::get_rations(pool).await;
        let rations_price = rations.price.clone().unwrap().as_cp();

        let mut count = 0;
        while rations_allowance > 0 && count < 10 {
            self.inventory.push(rations.clone());
            rations_allowance -= rations_price;
            count += 1;
        }

        self.add_all_to_inv(pool, self.wealth).await;
        self.inventory
            .sort_unstable_by(|a, b| a.item_category.cmp(&b.item_category));

        Ok(())
    }

    pub fn get_wealth_in_inv(&self) -> i32 {
        let mut sum = 0;
        for i in self.inventory.iter() {
            sum += i.price.as_ref().unwrap().as_cp();
        }

        sum
    }

    async fn add_all_to_inv(&mut self, pool: &Pool<Sqlite>, mut allowance: i32) -> Result<()> {
        let mut rng = rand::thread_rng();
        let minimums = database::get_min_for_each_category(pool, self.level).await?;
        let mut minimum = 0;

        while allowance > 0 {
            let category = ItemCategory::rand();
            let temp = minimums.get(&category);

            if temp.is_none() {
                continue;
            }
            minimum = *temp.unwrap();

            let items =
                database::get_category(pool, category, Rarity::Common, self.level, true).await?;
            let uncommon =
                database::get_category(pool, category, Rarity::Uncommon, self.level, true).await?;
            let rare =
                database::get_category(pool, category, Rarity::Rare, self.level, true).await?;

            if allowance < minimum {
                continue;
            }

            let mut choice = items.choose(&mut rng).unwrap();
            let mut price = choice.price.as_ref().unwrap().as_cp();
            while price > allowance {
                choice = items.choose(&mut rng).unwrap();
                price = choice.price.as_ref().unwrap().as_cp();
            }

            let upgrade_roll = rng.gen_range(0.0..1.0);
            if upgrade_roll <= UNCOMMON_CHANCE {
                let maybe_choice = uncommon.choose(&mut rng);
                if maybe_choice.is_some() {
                    choice = maybe_choice.unwrap();
                }
            } else if upgrade_roll <= RARE_CHANCE {
                let maybe_choice = rare.choose(&mut rng);
                if maybe_choice.is_some() {
                    choice = maybe_choice.unwrap();
                }
            }

            self.inventory.push(choice.clone());
            allowance -= price;
        }

        Ok(())
    }

    async fn add_category_to_inv<F: Fn(i32, i32) -> bool>(
        &mut self,
        pool: &Pool<Sqlite>,
        category: ItemCategory,
        subcategory: Option<&str>,
        mut allowance: i32,
        predicate: F,
    ) -> Result<()> {
        let mut rng = rand::thread_rng();

        let mut items =
            database::get_category(pool, category, Rarity::Common, self.level, true).await?;
        if let Some(subcategory) = subcategory {
            items.retain(|i| i.item_subcategory == subcategory);
        }
        let items = items; // immutable rebind

        let uncommon =
            database::get_category(pool, category, Rarity::Uncommon, self.level, true).await?;
        let rare = database::get_category(pool, category, Rarity::Rare, self.level, true).await?;

        let mut count = 0;

        while predicate(allowance, count) {
            let mut choice = items.choose(&mut rng).unwrap();
            let mut price = choice.price.as_ref().unwrap().as_cp();
            while price > allowance {
                choice = items.choose(&mut rng).unwrap();
                price = choice.price.as_ref().unwrap().as_cp();
            }

            let upgrade_roll = rng.gen_range(0.0..1.0);
            if upgrade_roll <= UNCOMMON_CHANCE {
                choice = uncommon.choose(&mut rng).unwrap();
            } else if upgrade_roll <= RARE_CHANCE {
                choice = rare.choose(&mut rng).unwrap();
            }

            self.inventory.push(choice.clone());
            allowance -= price;
            count += 1;
        }

        Ok(())
    }
}

impl std::fmt::Display for Merchant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut categories: HashMap<&str, HashMap<&str, (i32, &Price)>> = HashMap::new();
        for item in self.inventory.iter() {
            if categories.contains_key(item.item_category.as_str()) {
                // outer has key
                let inner = categories.get_mut(item.item_category.as_str()).unwrap();

                if inner.contains_key(item.name.as_str()) {
                    // inner has key
                    inner.get_mut(item.name.as_str()).unwrap().0 += 1;
                } else {
                    // inner doesn't have key
                    inner.insert(item.name.as_str(), (1, item.price.as_ref().unwrap()));
                }
            } else {
                // outer doesn't have key
                let mut new_inner = HashMap::new();
                new_inner.insert(item.name.as_str(), (1, item.price.as_ref().unwrap()));
                categories.insert(item.item_category.as_str(), new_inner);
            }
        }

        for (key, items) in categories {
            writeln!(f, "\n---------- {} ----------", key);
            for (name, (count, price)) in items {
                writeln!(f, "{} x{} - {}", name, count, price);
            }
        }

        Ok(())
    }
}
