use crate::item::Item;
use anyhow::{Context, Result};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct Merchant {
    wealth: i32,
    inventory: Vec<Item>,
}

impl Merchant {
    pub fn new(wealth: i32) -> Self {
        Self {
            wealth,
            inventory: vec![], // TODO
        }
    }

    pub fn save(&self) -> Result<()> {
        let ron = ron::to_string(self)?;
        let filename = format!(
            "{}.txt",
            chrono::offset::Local::now().format("%Y-%m-%d_%I:%M %p")
        );
        std::fs::write(filename, ron)?;
        Ok(())
    }

    pub fn generate_inventory(&mut self) {

    }
}
