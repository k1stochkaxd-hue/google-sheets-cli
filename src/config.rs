use serde::{Serialize, Deserialize};
use std::fs;
use anyhow::{Result};
use rand::Rng;
use colored::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct SpreadsheetConfig {
    pub name: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NamedList {
    pub id: String,
    pub elements: Vec<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub spreadsheets: Vec<SpreadsheetConfig>,
    pub lists: Vec<NamedList>,
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

    pub fn list_named_lists(&self) -> Vec<String> {
        self.lists.iter()
            .map(|l| format!("ID: {} ({} elements)", l.id.green().bold(), l.elements.len()))
            .collect()
    }

    pub fn add_named_list(&mut self, elements: Vec<String>, mut id: String) -> String {
        let mut rng = rand::thread_rng();
        // Check for empty or duplicate ID
        if id.is_empty() || self.lists.iter().any(|l| l.id == id) {
            loop {
                let candidate = format!("{:05}", rng.gen_range(10000..100000));
                if !self.lists.iter().any(|l| l.id == candidate) {
                    id = candidate;
                    break;
                }
            }
        }
        
        self.lists.push(NamedList { id: id.clone(), elements });
        self.save().ok();
        id
    }

    pub fn remove_named_list(&mut self, id: &str) -> bool {
        let pos = self.lists.iter().position(|l| l.id == id);
        if let Some(p) = pos {
            self.lists.remove(p);
            self.save().ok();
            true
        } else {
            false
        }
    }
}
