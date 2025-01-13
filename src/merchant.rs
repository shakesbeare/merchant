use std::collections::HashMap;

use crate::{
    database,
    item::{Item, ItemKind, Price, Rarity},
};
use anyhow::{Context, Result};
use enum_derived::Rand;
use rand::{seq::SliceRandom, Rng};
use sqlx::{Pool, Sqlite};

const MIN_ARMOR_COST: i32 = 20;
const MIN_CONSUMABLE_COST: i32 = 300;

const UNCOMMON_CHANCE: f32 = 0.005;
const RARE_CHANCE: f32 = 0.001;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct Merchant {
    /// The merchant's wealth in cp
    wealth: i32,
    inventory: Vec<Item>,
}

impl Merchant {
    pub fn new(cp: i32) -> Self {
        Self {
            wealth: cp,
            inventory: vec![],
        }
    }

    pub fn from_gp(gp: i32) -> Self {
        Self::new(gp * 100)
    }

    pub fn by_level(level: i32) -> Self {
        let gp = match level {
            1 => 175,
            2 => 300,
            3 => 500,
            4 => 850,
            5 => 1350,
            6 => 2000,
            7 => 2900,
            8 => 4000,
            9 => 5700,
            10 => 8000,
            11 => 11500,
            12 => 16500,
            13 => 25000,
            14 => 36500,
            15 => 54500,
            16 => 82500,
            17 => 128000,
            18 => 208000,
            19 => 355000,
            20 => 490000,
            _ => unreachable!(),
        };
        Self::from_gp(gp)
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

    pub fn len(&self) -> usize {
        self.inventory.len()
    }

    pub async fn generate_inventory(&mut self, pool: &Pool<Sqlite>) -> Result<()> {
        let max_items = self.wealth / (10 * 100);
        let armor_weapons = self.wealth / 2;
        let remainder = self.wealth - armor_weapons;
        let mut rations_allowance = remainder / 24;
        let mut alch_allowance = 3 * remainder / 20;
        let mut adv_gear_allowance = 4 * remainder / 8;
        let mut armor_allowance = armor_weapons / 2;
        let mut weapons_allowance = armor_weapons / 2;

        let rations = database::get_rations(pool).await;
        let rations_price = rations.price.clone().unwrap().as_cp();

        let mut count = 0;
        while rations_allowance > 0 && count < 10 {
            self.inventory.push(rations.clone());
            rations_allowance -= rations_price;
            count += 1;
        }

        let minimum_value = (self.wealth as f32 * 0.02) as i32;

        self.add_category_to_inv(
            pool,
            ItemKind::AdventuringGear,
            None,
            adv_gear_allowance,
            |allowance, count| allowance > 0 && count < max_items / 4,
        )
        .await;

        self.add_category_to_inv(
            pool,
            ItemKind::Consumables,
            Some("Potions"),
            alch_allowance,
            |allowance, count| {
                allowance > MIN_CONSUMABLE_COST && count < max_items / 4
            },
        )
        .await;

        self.add_category_to_inv(
            pool,
            ItemKind::Armor,
            None,
            weapons_allowance,
            |allowance, count| {
                allowance > MIN_ARMOR_COST && allowance > minimum_value && count < max_items / 4
            },
        )
        .await;

        self.add_category_to_inv(
            pool,
            ItemKind::Weapons,
            None,
            weapons_allowance,
            |allowance, count| allowance > 0 && count < max_items / 4,
        )
        .await;

        Ok(())
    }

    pub fn get_wealth_in_inv(&self) -> i32 {
        let mut sum = 0;
        for i in self.inventory.iter() {
            sum += i.price.as_ref().unwrap().as_cp();
        }

        sum
    }

    async fn add_category_to_inv<F: Fn(i32, i32) -> bool>(
        &mut self,
        pool: &Pool<Sqlite>,
        category: ItemKind,
        subcategory: Option<&str>,
        mut allowance: i32,
        predicate: F,
    ) -> Result<()> {
        let mut rng = rand::thread_rng();

        let mut items = database::get_category(pool, category, Rarity::Common, true).await?;
        if let Some(subcategory) = subcategory {
            items.retain(|i| i.item_subcategory == subcategory);
        }
        let items = items; // immutable rebind

        
        let uncommon = database::get_category(pool, category, Rarity::Uncommon, true).await?;
        let rare = database::get_category(pool, category, Rarity::Rare, true).await?;

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
        let mut adv_gear: HashMap<String, (i32, Price)> = HashMap::new();
        let mut consumables: HashMap<String, (i32, Price)> = HashMap::new();
        let mut weapons: HashMap<String, (i32, Price)> = HashMap::new();
        let mut armor: HashMap<String, (i32, Price)> = HashMap::new();
        for item in self.inventory.iter() {
            // writeln!(f, "{} - {}", item.name, item.price.as_ref().unwrap());
            match item.item_category.as_ref() {
                "Adventuring Gear" => {
                    if adv_gear.contains_key(&item.name) {
                        adv_gear.get_mut(&item.name).unwrap().0 += 1;
                    } else {
                        adv_gear.insert(item.name.clone(), (1, item.price.clone().unwrap()));
                    }
                }
                "Weapons" => {
                    if weapons.contains_key(&item.name) {
                        weapons.get_mut(&item.name).unwrap().0 += 1;
                    } else {
                        weapons.insert(item.name.clone(), (1, item.price.clone().unwrap()));
                    }
                }
                "Armor" => {
                    if armor.contains_key(&item.name) {
                        armor.get_mut(&item.name).unwrap().0 += 1;
                    } else {
                        armor.insert(item.name.clone(), (1, item.price.clone().unwrap()));
                    }
                }
                "Consumables" => {
                    if consumables.contains_key(&item.name) {
                        consumables.get_mut(&item.name).unwrap().0 += 1;
                    } else {
                        consumables
                            .insert(item.name.clone(), (1, item.price.clone().unwrap()));
                    }
                }
                _ => todo!(),
            }
        }

        writeln!(f, "\n---------- ADVENTURING GEAR ---------- ");
        for (key, (count, price)) in adv_gear.into_iter() {
            writeln!(f, "{} x{} - {}", key, count, price);
        }
        writeln!(f, "\n---------- POTIONS ---------- ");
        for (key, (count, price)) in consumables.into_iter() {
            writeln!(f, "{} x{} - {}", key, count, price);
        }
        writeln!(f, "\n---------- ARMOR ---------- ");
        for (key, (count, price)) in armor.into_iter() {
            writeln!(f, "{} x{} - {}", key, count, price);
        }
        writeln!(f, "\n---------- WEAPONS ---------- ");
        for (key, (count, price)) in weapons.into_iter() {
            writeln!(f, "{} x{} - {}", key, count, price);
        }

        Ok(())
    }
}
