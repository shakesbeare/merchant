use std::collections::HashMap;

use crate::{
    database,
    item::{Item, ItemKind, Price},
};
use anyhow::{Context, Result};
use rand::seq::SliceRandom;
use sqlx::{Pool, Sqlite};

const MIN_ARMOR_COST: i32 = 20;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct Merchant {
    /// The merchant's wealth in cp
    wealth: i32,
    level: i32,
    inventory: Vec<Item>,
}

impl Merchant {
    pub fn new(wealth: i32, level: i32) -> Self {
        Self {
            wealth,
            level,
            inventory: vec![], // TODO
        }
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
        let mut rations_allowance = self.wealth / 8;
        let mut adv_gear_allowance = self.wealth / 4;
        let mut armor_allowance = self.wealth / 8;
        let mut weapons_allowance = self.wealth / 8;

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
        dbg!(&adventuring_gear);

        let mut rng = rand::thread_rng();

        tracing::trace!("Beginning inv generation");

        while rations_allowance > 0 {
            self.inventory.push(rations.clone());
            rations_allowance -= rations_price;
        }

        tracing::trace!("Rations done");

        let mut i = 0;
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
        }

        tracing::trace!("adventuring gear done");

        let mut i = 0;
        while armor_allowance > MIN_ARMOR_COST {
            let mut choice = armor.get(i).unwrap();
            let mut price = choice.price.as_ref().unwrap().as_cp();
            while price > armor_allowance {
                i += 1;
                choice = armor.get(i).unwrap();
                price = choice.price.as_ref().unwrap().as_cp();
            }
            self.inventory.push(choice.clone());
            armor_allowance -= price;
        }

        tracing::trace!("armor done");

        let mut i = 0;
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
        }

        tracing::trace!("weapons done");

        Ok(())
    }
}

impl std::fmt::Display for Merchant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut items: HashMap<String, (i32, Price)> = HashMap::new();
        for item in self.inventory.iter() {
            // writeln!(f, "{} - {}", item.name, item.price.as_ref().unwrap());
            if items.contains_key(&item.name) {
                items.get_mut(&item.name).unwrap().0 += 1;
            } else {
                items.insert(item.name.clone(), (1, item.price.clone().unwrap()));
            }
        }

        for (key, (count, price)) in items.into_iter() {
            writeln!(f, "{} x{} - {}", key, count, price);
        }

        Ok(())
    }
}
