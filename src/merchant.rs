use std::collections::HashMap;

use crate::{
    database,
    item::{Item, ItemKind, Price},
};
use anyhow::{Context, Result};
use rand::{seq::SliceRandom, Rng};
use sqlx::{Pool, Sqlite};

const MIN_ARMOR_COST: i32 = 20;

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
            inventory: vec![], // TODO
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
        let quotient = (-(self.wealth as f32 / 26400.0).log10() + 2.5);
        tracing::debug!("armor_weapons quotient  is {}", quotient);
        let armor_weapons = (self.wealth as f32 / quotient) as i32;
        tracing::debug!("armor_weapons allowance is {}", armor_weapons);
        let remainder = self.wealth - armor_weapons;
        let mut rations_allowance = remainder / 8;
        let mut adv_gear_allowance = 7 * remainder / 8;
        let mut armor_allowance = armor_weapons / 2;
        let mut weapons_allowance = armor_weapons / 2;

        let rations = database::get_rations(pool).await;
        let rations_price = rations.price.clone().unwrap().as_cp();

        let mut adventuring_gear =
            database::get_category(pool, ItemKind::AdventuringGear, true).await?;
        let mut weapons = database::get_category(pool, ItemKind::Weapons, true).await?;
        let mut armor = database::get_category(pool, ItemKind::Armor, true).await?;

        adventuring_gear.sort_unstable_by(|a, b| {
            b.price
                .as_ref()
                .unwrap()
                .as_cp()
                .cmp(&a.price.as_ref().unwrap().as_cp())
        });
        weapons.sort_unstable_by(|a, b| {
            b.price
                .as_ref()
                .unwrap()
                .as_cp()
                .cmp(&a.price.as_ref().unwrap().as_cp())
        });
        armor.sort_unstable_by(|a, b| {
            b.price
                .as_ref()
                .unwrap()
                .as_cp()
                .cmp(&a.price.as_ref().unwrap().as_cp())
        });
        let mut rng = rand::thread_rng();

        tracing::trace!("Beginning inv generation");

        while rations_allowance > 0 {
            self.inventory.push(rations.clone());
            rations_allowance -= rations_price;
        }

        tracing::trace!("Rations done");

        let minimum_value = (self.wealth as f32 * 0.02) as i32;

        let mut i = rng.gen_range(0..adventuring_gear.len());
        while adv_gear_allowance > 0 {
            let mut choice = adventuring_gear.get(i).unwrap();
            let mut price = choice.price.as_ref().unwrap().as_cp();
            while price > adv_gear_allowance {
                i += 1;
                choice = adventuring_gear.get(i).unwrap();
                price = choice.price.as_ref().unwrap().as_cp();
            }
            self.inventory.push(choice.clone());
            adv_gear_allowance -= price;
            i = rng.gen_range(0..adventuring_gear.len());
        }

        tracing::trace!("adventuring gear done");

        let mut i = rng.gen_range(0..armor.len());
        while armor_allowance > MIN_ARMOR_COST && armor_allowance > minimum_value {
            let mut choice = armor.get(i).unwrap();
            let mut price = choice.price.as_ref().unwrap().as_cp();
            while price > armor_allowance {
                i += 1;
                choice = armor.get(i).unwrap();
                price = choice.price.as_ref().unwrap().as_cp();
            }
            self.inventory.push(choice.clone());
            armor_allowance -= price;
            i = rng.gen_range(0..armor.len());
        }

        tracing::trace!("armor done");

        let mut i = rng.gen_range(0..weapons.len());
        while weapons_allowance > 0 {
            let mut choice = weapons.get(i).unwrap();
            let mut price = choice.price.as_ref().unwrap().as_cp();
            while price > weapons_allowance {
                i += 1;
                choice = weapons.get(i).unwrap();
                price = choice.price.as_ref().unwrap().as_cp();
            }
            self.inventory.push(choice.clone());
            weapons_allowance -= price;
            i = rng.gen_range(0..weapons.len());
        }

        tracing::trace!("weapons done");

        Ok(())
    }
}

impl std::fmt::Display for Merchant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut adv_gear: HashMap<String, (i32, Price)> = HashMap::new();
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
                _ => todo!()
            }
        }

        for (key, (count, price)) in adv_gear.into_iter() {
            writeln!(f, "{} x{} - {}", key, count, price);
        }
        for (key, (count, price)) in armor.into_iter() {
            writeln!(f, "{} x{} - {}", key, count, price);
        }
        for (key, (count, price)) in weapons.into_iter() {
            writeln!(f, "{} x{} - {}", key, count, price);
        }


        Ok(())
    }
}
