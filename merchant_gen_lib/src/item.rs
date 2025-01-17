use anyhow::{Context, Result};
use regex::Regex;

use crate::database::DbItem;

// r#""^(?:(\d* \w{2}),? ?)*(.*)?\n"#
lazy_static::lazy_static! {
    static ref PRICE_RE: Regex = {
        // Regex::new(r#""^(?:(\d* \w{2}),? ?)*(.*)?\n"#).unwrap()
        Regex::new(r#"^(?:([\d,]* \w{2})(?:, )?)*"#).unwrap()
    };
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct Item {
    pub name: String,
    pub pfs: String,
    pub source: String,
    pub rarity: String,
    pub r#trait: String,
    pub item_category: String,
    pub item_subcategory: String,
    pub level: i32,
    pub price: Option<Price>,
    pub bulk: String,
    pub usage: String,
    pub spoilers: String,
}

impl From<DbItem> for Item {
    fn from(value: DbItem) -> Self {
        Self {
            name: value.name,
            pfs: value.pfs,
            source: value.source,
            rarity: value.rarity,
            r#trait: value.r#trait,
            item_category: value.item_category,
            item_subcategory: value.item_subcategory,
            level: value.level,
            price: Price::parse(&value.price).unwrap_or_else(|e| {
                tracing::error!("{:?}", e);
                tracing::error!("{:?}", value.price);
                panic!();
            }),
            bulk: value.bulk,
            usage: value.usage,
            spoilers: value.spoilers,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Price {
    text: String,
    pp: i32,
    gp: i32,
    sp: i32,
    cp: i32,
}

impl<S: AsRef<str>> From<S> for Price {
    fn from(value: S) -> Self {
        Self::parse(value)
            .context("Failed to parse price")
            .unwrap()
            .unwrap()
    }
}

impl Price {
    pub fn parse<S: AsRef<str>>(input: S) -> Result<Option<Price>> {
        if input.as_ref().is_empty() {
            return Ok(None);
        }
        let mut pp = 0;
        let mut gp = 0;
        let mut sp = 0;
        let mut cp = 0;

        let captures = PRICE_RE
            .captures(input.as_ref())
            .context("Failed to find price")?;

        for c in captures.iter().flatten() {
            if c.is_empty() {
                continue;
            }
            let (amount, coin) = c.as_str().split_once(" ").context("Failed to split text")?;
            let Some(coin) = coin.get(0..2) else { continue };
            let amount = amount
                .chars()
                .filter(|c| c.is_numeric())
                .collect::<String>()
                .parse::<i32>()
                .context("Failed to parse amount as i32")?;
            match coin {
                "pp" => pp = amount,
                "gp" => gp = amount,
                "sp" => sp = amount,
                "cp" => cp = amount,
                _ => {
                    #[cfg(not(test))]
                    tracing::error!("Unrecognized coin: {}", coin);
                    #[cfg(test)]
                    panic!("Unrecognized coin: {}", coin);
                }
            }
        }

        Ok(Some(Price {
            text: input.as_ref().to_string(),
            pp,
            gp,
            sp,
            cp,
        }))
    }

    pub fn as_cp(&self) -> i32 {
        let pp = self.pp * 1000;
        let gp = self.gp * 100;
        let sp = self.sp * 10;

        pp + gp + sp + self.cp
    }
}

impl std::fmt::Display for Price {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

#[derive(
    enum_derived::Rand,
    Debug,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    enum_iterator::Sequence,
    Hash,
)]
pub enum ItemCategory {
    #[weight(0)]
    Vehicles,

    #[weight(60)]
    Weapons,

    #[weight(25)]
    AdventuringGear,

    #[weight(25)]
    AlchemicalItems,

    #[weight(0)]
    Services,

    #[weight(25)]
    TradeGoods,

    #[weight(0)]
    AnimalsandGear,

    #[weight(25)]
    Materials,

    #[weight(60)]
    Armor,

    AssistiveItems,

    Adjustments,

    #[weight(25)]
    Shields,

    Other,

    Customizations,

    #[weight(50)]
    Consumables,

    Snares,

    #[weight(25)]
    HeldItems,

    #[weight(25)]
    WornItems,

    #[weight(12)]
    Grafts,

    #[weight(4)]
    Tattoos,

    #[weight(0)]
    SiegeWeapons,

    #[weight(10)]
    Runes,

    #[weight(0)]
    Artifacts,

    #[weight(0)]
    CursedItems,

    #[weight(10)]
    Spellhearts,

    #[weight(60)]
    Wands,

    #[weight(60)]
    Staves,

    #[weight(0)]
    IntelligentItems,

    #[weight(0)]
    Contracts,

    #[weight(0)]
    Relics,

    #[weight(10)]
    Grimoires,

    #[weight(3)]
    Structures,

    #[weight(3)]
    Censer,

    #[weight(3)]
    Figurehead,

    #[weight(0)]
    BlightedBoons,

    #[weight(0)]
    HighTech,
}

impl AsRef<str> for ItemCategory {
    fn as_ref(&self) -> &str {
        match self {
            ItemCategory::Vehicles => "Vehicles",
            ItemCategory::Weapons => "Weapons",
            ItemCategory::AdventuringGear => "Adventuring Gear",
            ItemCategory::AlchemicalItems => "Alchemical Items",
            ItemCategory::Services => "Services",
            ItemCategory::TradeGoods => "Trade Goods",
            ItemCategory::AnimalsandGear => "Animals and Gear",
            ItemCategory::Materials => "Materials",
            ItemCategory::Armor => "Armor",
            ItemCategory::AssistiveItems => "Assistive Items",
            ItemCategory::Adjustments => "Adjustments",
            ItemCategory::Shields => "Shields",
            ItemCategory::Other => "Other",
            ItemCategory::Customizations => "Customizations",
            ItemCategory::Consumables => "Consumables",
            ItemCategory::Snares => "Snares",
            ItemCategory::HeldItems => "Held Items",
            ItemCategory::WornItems => "Worn Items",
            ItemCategory::Grafts => "Grafts",
            ItemCategory::Tattoos => "Tattoos",
            ItemCategory::SiegeWeapons => "Siege Weapons",
            ItemCategory::Runes => "Runes",
            ItemCategory::Artifacts => "Artifacts",
            ItemCategory::CursedItems => "Cursed Items",
            ItemCategory::Spellhearts => "Spellhearts",
            ItemCategory::Wands => "Wands",
            ItemCategory::Staves => "Staves",
            ItemCategory::IntelligentItems => "Intelligent Items",
            ItemCategory::Contracts => "Contracts",
            ItemCategory::Relics => "Relics",
            ItemCategory::Grimoires => "Grimoires",
            ItemCategory::Structures => "Structures",
            ItemCategory::Censer => "Censer",
            ItemCategory::Figurehead => "Figurehead",
            ItemCategory::BlightedBoons => "Blighted Boons",
            ItemCategory::HighTech => "HighTech",
        }
    }
}

#[derive(enum_derived::Rand)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
}

impl AsRef<str> for Rarity {
    fn as_ref(&self) -> &str {
        match self {
            Rarity::Common => "Common",
            Rarity::Uncommon => "Uncommon",
            Rarity::Rare => "Rare",
        }
    }
}

mod tests {
    #![allow(unused)]
    use crate::item::Price;

    #[test]
    fn parse_prices() {
        let input = [
            ("1 sp (price for 10)", [0, 0, 1, 0]),
            ("1 sp, 7 cp (per 1,000 bricks)", [0, 0, 1, 7]),
            ("30 gp", [0, 30, 0, 0]),
            ("4 sp (1 week)", [0, 0, 4, 0]),
            ("1,500 gp", [0, 1500, 0, 0]),
        ];

        for (input, expected) in input {
            let price = Price::parse(input);
            let price = price.unwrap().unwrap();
            assert_eq!(price.text, input);
            assert_eq!(price.pp, expected[0]);
            assert_eq!(price.gp, expected[1]);
            assert_eq!(price.sp, expected[2]);
            assert_eq!(price.cp, expected[3]);
        }
    }
}
