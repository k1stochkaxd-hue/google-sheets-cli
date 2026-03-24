use serde::{Serialize, Deserialize};
use std::fs;
use std::collections::HashMap;
use anyhow::Result;
use rand::Rng;
use colored::Colorize;

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
    /// Maps "sheetId:row:col" -> list_id for persistent cell dropdown assignments
    #[serde(default)]
    pub cell_list_map: HashMap<String, String>,
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

    /// Returns the cell key used in cell_list_map
    pub fn cell_key(sheet_id: i64, row: usize, col: usize) -> String {
        format!("{}:{}:{}", sheet_id, row, col)
    }

    /// Assigns a list ID to a specific cell and persists immediately
    pub fn assign_list_to_cell(&mut self, sheet_id: i64, row: usize, col: usize, list_id: String) {
        let key = Self::cell_key(sheet_id, row, col);
        self.cell_list_map.insert(key, list_id);
        self.save().ok();
    }

    /// Returns the list assigned to a cell (if any)
    pub fn get_cell_list(&self, sheet_id: i64, row: usize, col: usize) -> Option<&NamedList> {
        let key = Self::cell_key(sheet_id, row, col);
        let list_id = self.cell_list_map.get(&key)?;
        self.lists.iter().find(|l| &l.id == list_id)
    }

    /// Finds a list that has exactly the same elements. Used to avoid duplicates.
    pub fn find_list_by_elements(&self, elements: &[String]) -> Option<String> {
        self.lists.iter()
            .find(|l| l.elements == elements)
            .map(|l| l.id.clone())
    }

    /// Shows all named lists with their elements
    pub fn list_named_lists(&self) -> Vec<String> {
        self.lists.iter()
            .map(|l| format!(
                "ID: {} | {} element(s): {}",
                l.id.green().bold(),
                l.elements.len(),
                l.elements.join(", ").yellow()
            ))
            .collect()
    }

    /// Creates a new named list, auto-generating an ID if needed or if duplicate
    pub fn add_named_list(&mut self, elements: Vec<String>, mut id: String) -> String {
        let mut rng = rand::thread_rng();
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

    /// Removes a named list and cleans up all cell assignments pointing to it
    pub fn remove_named_list(&mut self, id: &str) -> bool {
        let pos = self.lists.iter().position(|l| l.id == id);
        if let Some(p) = pos {
            self.lists.remove(p);
            self.cell_list_map.retain(|_, v| v != id);
            self.save().ok();
            true
        } else {
            false
        }
    }
}
