use serde::{Serialize, Deserialize};
use std::fs;
use anyhow::{Result};

#[derive(Serialize, Deserialize, Clone)]
pub struct SpreadsheetConfig {
    pub name: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub spreadsheets: Vec<SpreadsheetConfig>,
}

impl AppConfig {
    pub fn load() -> Self {
        if let Ok(data) = fs::read_to_string("config.json") {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<()> {
        let data = serde_json::to_string_pretty(self)?;
        fs::write("config.json", data)?;
        Ok(())
    }

    pub fn add(&mut self, name: String, url: String) -> Result<()> {
        self.spreadsheets.push(SpreadsheetConfig { name, url });
        self.save()
    }

    pub fn remove(&mut self, index: usize) -> Result<()> {
        if index < self.spreadsheets.len() {
            self.spreadsheets.remove(index);
            self.save()?;
        }
        Ok(())
    }
}
