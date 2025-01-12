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

fn format_inventory(inv: MerchantInventory) -> String {
    let mut provs: HashMap<String, usize> = HashMap::new();
    let mut supps: HashMap<String, usize> = HashMap::new();
    //TODO
    let mut armor: HashMap<String, f32> = HashMap::new();
    let mut weapons: HashMap<String, f32> = HashMap::new();

    for item in inv {
        match item.kind {
            ItemKind::Provisions => {
                if provs.contains_key(&item.name) {
                    *provs.get_mut(&item.name).unwrap() += item.value as usize;
                } else {
                    provs.insert(item.name, item.value as usize);
                }
            }
            ItemKind::Supplies => {
                if supps.contains_key(&item.name) {
                    *supps.get_mut(&item.name).unwrap() += 1;
                } else {
                    supps.insert(item.name, 1);
                }
            }
            ItemKind::Armor => {
                armor.insert(item.name, item.value);
            }
            ItemKind::Weapons => {
                weapons.insert(item.name, item.value);
            }
        }
    }
    let mut out = String::new();

    for (key, value) in provs {
        let line = format!("{} x{}-days\n", key, value);
        out.push_str(&line);
    }

    for (key, value) in supps {
        let line = format!("{} x{}\n", key, value);
        out.push_str(&line);
    }

    for (key, value) in armor {
        let line = format!("Armor: {} for {}g\n", key, value);
        out.push_str(&line);
    }

    for (key, value) in weapons {
        let line = format!("Weapon: {} for {}g\n", key, value);
        out.push_str(&line);
    }

    out
}

fn main() {
    let wealth = 200.0;
    let inv = generate_inventory(wealth);
    println!("{}", format_inventory(inv));
}

