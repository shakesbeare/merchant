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

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize)]
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

#[derive(enum_derived::Rand)]
pub enum ItemKind {
    Vehicles,
    Weapons,
    AdventuringGear,
    AlchemicalItems,
    Services,
    TradeGoods,
    AnimalsandGear,
    Materials,
    Armor,
    AssistiveItems,
    Adjustments,
    Shields,
    Other,
    Customizations,
    Consumables,
    Snares,
    HeldItems,
    WornItems,
    Grafts,
    Tattoos,
    SiegeWeapons,
    Runes,
    Artifacts,
    CursedItems,
    Spellhearts,
    Wands,
    Staves,
    IntelligentItems,
    Contracts,
    Relics,
    Grimoires,
    Structures,
    Censer,
    Figurehead,
    BlightedBoons,
    HighTech,
}

impl AsRef<str> for ItemKind {
    fn as_ref(&self) -> &str {
        match self {
            ItemKind::Vehicles => "Vehicles",
            ItemKind::Weapons => "Weapons",
            ItemKind::AdventuringGear => "Adventuring Gear",
            ItemKind::AlchemicalItems => "Alchemical Items",
            ItemKind::Services => "Services",
            ItemKind::TradeGoods => "Trade Goods",
            ItemKind::AnimalsandGear => "Animals and Gear",
            ItemKind::Materials => "Materials",
            ItemKind::Armor => "Armor",
            ItemKind::AssistiveItems => "Assistive Items",
            ItemKind::Adjustments => "Adjustments",
            ItemKind::Shields => "Shields",
            ItemKind::Other => "Other",
            ItemKind::Customizations => "Customizations",
            ItemKind::Consumables => "Consumables",
            ItemKind::Snares => "Snares",
            ItemKind::HeldItems => "Held Items",
            ItemKind::WornItems => "Worn Items",
            ItemKind::Grafts => "Grafts",
            ItemKind::Tattoos => "Tattoos",
            ItemKind::SiegeWeapons => "Siege Weapons",
            ItemKind::Runes => "Runes",
            ItemKind::Artifacts => "Artifacts",
            ItemKind::CursedItems => "Cursed Items",
            ItemKind::Spellhearts => "Spellhearts",
            ItemKind::Wands => "Wands",
            ItemKind::Staves => "Staves",
            ItemKind::IntelligentItems => "Intelligent Items",
            ItemKind::Contracts => "Contracts",
            ItemKind::Relics => "Relics",
            ItemKind::Grimoires => "Grimoires",
            ItemKind::Structures => "Structures",
            ItemKind::Censer => "Censer",
            ItemKind::Figurehead => "Figurehead",
            ItemKind::BlightedBoons => "Blighted Boons",
            ItemKind::HighTech => "HighTech",
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
