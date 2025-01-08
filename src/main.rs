#![feature(variant_count)]

use rand::{prelude::Distribution, Rng};

const PROVISIONS_CHANCE: f32      = 0.5;
const SUPPLIES_CHANCE  : f32      = 0.25;
const ARMOR_CHANCE     : f32      = 0.125;
const WEAPONS_CHANCE   : f32      = 0.125;

const PROVISIONS_BREAKPOINT: f32 =  usize::MAX as f32 * PROVISIONS_CHANCE;
const SUPPLIES_BREAKPOINT  : f32 =  usize::MAX as f32 * SUPPLIES_CHANCE + PROVISIONS_BREAKPOINT;
const ARMOR_BREAKPOINT     : f32 =  usize::MAX as f32 * ARMOR_CHANCE    + SUPPLIES_BREAKPOINT;
const WEAPONS_BREAKPOINT   : f32 =  usize::MAX as f32 * WEAPONS_CHANCE  + ARMOR_BREAKPOINT;

const _: bool = {
    let total = PROVISIONS_CHANCE + SUPPLIES_CHANCE + ARMOR_CHANCE + WEAPONS_CHANCE;
    if total != 1.0 {
        panic!("Invalid chances config")
    } else {
        true
    }
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ShopItem {
    Provisions,
    Supplies,
    Armor,
    Weapons,
}

impl From<usize> for ShopItem {
    fn from(value: usize) -> Self {
        let value = value as f32;

        if value < PROVISIONS_BREAKPOINT {
            Self::Provisions
        } else if value < SUPPLIES_BREAKPOINT {
            Self::Supplies
        } else if value < ARMOR_BREAKPOINT {
            Self::Armor
        } else if value < WEAPONS_BREAKPOINT {
            Self::Weapons
        } else {
            eprintln!("{} did not match any breakpoint", value);
            unreachable!()
        }
    }
}

impl Distribution<ShopItem> for ShopItem {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ShopItem {
        (rng.next_u64() as usize).into()
    }
}

fn main() {
    let mut rng = rand::thread_rng();
    let item_choice: ShopItem = rng.sample(ShopItem::Armor);
    println!("{:?}", item_choice);
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use crate::ShopItem;

    #[test]
    fn distribution() {
        println!("This test may be non-deterministic");
        println!("You should try and run it again");
        const MAX_TRIES: usize = 10000;
        const TOLERANCE: f32 = 0.01;
        let mut rng = rand::thread_rng();
        let mut p_count: f32 = 0.0;
        let mut s_count: f32 = 0.0;
        let mut a_count: f32 = 0.0;
        let mut w_count: f32 = 0.0;

        for _ in 0..MAX_TRIES {
            let item_choice: ShopItem = rng.sample(ShopItem::Armor);
            match item_choice {
                ShopItem::Provisions => p_count += 1.0,
                ShopItem::Supplies => s_count += 1.0,
                ShopItem::Armor => a_count += 1.0,
                ShopItem::Weapons => w_count += 1.0,
            }
        }

        assert!((p_count / MAX_TRIES as f32) - crate::PROVISIONS_CHANCE.abs() < TOLERANCE);
        assert!((s_count / MAX_TRIES as f32) - crate::SUPPLIES_CHANCE.abs() < TOLERANCE);
        assert!((a_count / MAX_TRIES as f32) - crate::ARMOR_CHANCE.abs() < TOLERANCE);
        assert!((w_count / MAX_TRIES as f32) - crate::WEAPONS_CHANCE.abs() < TOLERANCE);
    }
}
