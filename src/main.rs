use rand::{distributions::WeightedIndex, prelude::Distribution, seq::SliceRandom, Rng};
use enum_derived::Rand;

include!(concat!(env!("OUT_DIR"), "/data.rs"));

const VALUE_DIST_MEAN: f32 = 0.0;
const VALUE_DIST_SD: f32 = 3.0;

type MerchantInventory = Vec<Item>;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Item {
    kind: ItemKind,
    name: String,
    value: f32,
}

// the higher the prob, the lower the value
fn get_value<R: Rng>(mut rng: R) -> f32 {
    let normal = rand_distr::Normal::new(VALUE_DIST_MEAN, VALUE_DIST_SD).unwrap();
    let val = normal.sample(&mut rng).abs();
    if val == 0.0 { 1.0 } else { val }
}

fn generate_one<R: Rng>(mut rng: R) -> Item {
    let data: &Vec<(&ItemKind, &f32)> = &PROBABILITIES.iter().collect();
    let weights = data.iter().map(|(_, b)| **b);
    let mult = 1.0 / weights.clone().reduce(f32::min).unwrap();
    let weights: Vec<i32> = weights.map(|f| (f * mult) as i32).collect();
    let choices: Vec<ItemKind> = data.iter().map(|(a, _)| **a).collect();

    let dist = WeightedIndex::new(&weights).unwrap();
    let kind = choices[dist.sample(&mut rng)];

    let mut value = get_value(&mut rng);

    match kind {
        ItemKind::Provisions => {
            value = value.round();
        }
        ItemKind::Supplies => {}
        ItemKind::Weapons => {
            value *= 30.0;
        }
        ItemKind::Armor => {
            value *= 17.5;
        }
    }

    let name = if kind == ItemKind::Provisions {
        format!("{:?}", ProvisionsKind::rand())
    } else if kind == ItemKind::Supplies {
        format!("{}", SuppliesKind::rand())
    }
    else {
        "Placeholder".to_string()
    };

    Item {
        kind,
        name,
        value,
    }
}

fn generate_inventory(wealth: f32) -> MerchantInventory {
    let mut rng = rand::thread_rng();
    let mut wealth_accum = wealth;
    let mut inventory: MerchantInventory = Vec::new();
    while wealth_accum > 0.0 {
        let new_item = generate_one(&mut rng);
        wealth_accum -= new_item.value;
        inventory.push(new_item);
    }
    inventory
}

fn main() {
    let wealth = 200.0;
    let inv = generate_inventory(wealth);
    dbg!(inv);
}
