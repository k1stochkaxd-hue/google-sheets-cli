use anyhow::Result;
use crate::sheets::{SheetsClient, SheetMeta};
use crate::config::AppConfig;

/// Represents a single change made to a cell, used for undo/redo functionality
#[derive(Debug, Clone)]
pub struct CellAction {
    pub sheet_idx: usize,
    pub row: usize,
    pub col: usize,
    pub old_value: String,
    pub new_value: String,
}

/// The main application state holding the Google Sheets client and UI data
pub struct App {
    pub client: SheetsClient,
    pub sheets: Vec<SheetMeta>,
    pub current_sheet_idx: usize,
    pub data: Vec<Vec<String>>,
    pub selected_row: Option<usize>,
    pub selected_col: Option<usize>,
    pub cell_options: Vec<String>, // Options for data validation (dropdowns)
    pub undo_stack: Vec<CellAction>,
    pub redo_stack: Vec<CellAction>,
}

impl App {
    /// Creates a new application instance and fetches spreadsheet metadata
    pub async fn new(client: SheetsClient) -> Result<Self> {
        let sheets = client.fetch_metadata().await?;
        Ok(Self {
            client,
            sheets,
            current_sheet_idx: 0,
            data: Vec::new(),
            selected_row: None,
            selected_col: None,
            cell_options: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        })
    }

    /// Loads data and all metadata (including dropdowns) for the current worksheet
    pub async fn load_current_sheet(&mut self, config: &mut AppConfig) -> Result<()> {
        let title = &self.sheets[self.current_sheet_idx].title;
        let sheet_id = self.sheets[self.current_sheet_idx].id;
        
        let full_data = self.client.get_sheet_full(title).await?;
        self.data.clear();
        
        if let Some(sheets) = full_data["sheets"].as_array() {
            if let Some(sheet) = sheets.get(0) {
                if let Some(grid_data) = sheet["data"].as_array() {
                    if let Some(grid) = grid_data.get(0) {
                        if let Some(row_data) = grid["rowData"].as_array() {
                            for (r_idx, row) in row_data.iter().enumerate() {
                                let mut values = Vec::new();
                                if let Some(cell_values) = row["values"].as_array() {
                                    for (c_idx, cell) in cell_values.iter().enumerate() {
                                        let val = cell["formattedValue"].as_str().unwrap_or("").to_string();
                                        values.push(val);
                                        
                                        // AUTO-IMPORT DROPDOWN if found in this cell
                                        if let Some(v_rule) = cell["dataValidation"].as_object() {
                                            if let Some(cond) = v_rule["condition"].as_object() {
                                                let c_type = cond.get("type").and_then(|t| t.as_str()).unwrap_or("");
                                                if c_type == "ONE_OF_LIST" {
                                                    if let Some(vals) = cond.get("values").and_then(|v| v.as_array()) {
                                                        let mut elements = Vec::new();
                                                        for v in vals {
                                                            if let Some(uv) = v["userEnteredValue"].as_str() {
                                                                elements.push(uv.to_string());
                                                            }
                                                        }
                                                        if !elements.is_empty() && config.get_cell_list(sheet_id, r_idx, c_idx + 1).is_none() {
                                                            let addr = gspread_addr(r_idx, c_idx + 1);
                                                            let base_id = format!("G_{}", addr);
                                                            let new_id = config.add_named_list(elements, base_id);
                                                            config.assign_list_to_cell(sheet_id, r_idx, c_idx + 1, new_id);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                self.data.push(values);
                            }
                        }
                    }
                }
            }
        }
        
        // Refresh options for current selection
        self.restore_options_from_config(config);
        Ok(())
    }

    /// Applies a change to a specific cell and pushes to undo stack if requested
    pub async fn apply_change(&mut self, row: usize, col: usize, new_value: String, record: bool) -> Result<()> {
        let title = &self.sheets[self.current_sheet_idx].title;
        let addr = gspread_addr(row, col);
        let range = format!("{}!{}", title, addr);

        let old_value = self.get_cell_value(row, col).to_string();
        
        self.client.update_cell(&range, &new_value).await?;

        if record {
            self.undo_stack.push(CellAction {
                sheet_idx: self.current_sheet_idx,
                row,
                col,
                old_value,
                new_value,
            });
            self.redo_stack.clear();
        }
        Ok(())
    }

    /// Returns the current value of a cell at given row/col index
    pub fn get_cell_value(&self, row: usize, col: usize) -> &str {
        if row > 0 && row <= self.data.len() {
            let row_data = &self.data[row]; // Data index matches row index (headers included)
            if col > 0 && col <= row_data.len() {
                return &row_data[col - 1];
            }
        }
        ""
    }

    /// Fetches data validation options (dropdown menus) for the selected cell
    pub async fn fetch_options(&mut self) -> Result<()> {
        self.cell_options.clear();
        if let (Some(r), Some(c)) = (self.selected_row, self.selected_col) {
            if r == 0 || c == 0 {
                return Ok(());
            }
            let title = &self.sheets[self.current_sheet_idx].title;
            let addr = gspread_addr(r, c);
            let range = format!("'{}'!{}", title, addr);
            
            let resp = self.client.get_cell_metadata(&range).await?;
            
            // Navigate through the complex Google Sheets API response to find validation rules
            if let Some(sheets) = resp["sheets"].as_array() {
                if let Some(sheet) = sheets.get(0) {
                    if let Some(data) = sheet["data"].as_array() {
                        if let Some(grid) = data.get(0) {
                            if let Some(row_data) = grid["rowData"].as_array() {
                                if let Some(row) = row_data.get(0) {
                                    if let Some(values) = row["values"].as_array() {
                                        if let Some(cell) = values.get(0) {
                                            if let Some(v_rule) = cell["dataValidation"].as_object() {
                                                if let Some(cond) = v_rule["condition"].as_object() {
                                                    let c_type = cond.get("type").and_then(|t| t.as_str()).unwrap_or("");
                                                    if let Some(vals) = cond.get("values").and_then(|v| v.as_array()) {
                                                        for v in vals {
                                                            if let Some(uv) = v["userEnteredValue"].as_str() {
                                                                self.cell_options.push(uv.to_string());
                                                            }
                                                        }
                                                    }
                                                    
                                                    // Handle dropdowns dependent on another range (ONE_OF_RANGE)
                                                    if c_type == "ONE_OF_RANGE" && !self.cell_options.is_empty() {
                                                        let range_expr = self.cell_options[0].clone();
                                                        let r_vals = self.client.get_values(&range_expr).await?;
                                                        self.cell_options.clear();
                                                        for r in r_vals {
                                                            if let Some(first) = r.get(0) {
                                                                self.cell_options.push(first.clone());
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Restores cell_options from the local config for the currently selected cell.
    pub fn restore_options_from_config(&mut self, config: &AppConfig) {
        self.cell_options.clear();
        if let (Some(r), Some(c)) = (self.selected_row, self.selected_col) {
            let sheet_id = self.sheets[self.current_sheet_idx].id;
            if let Some(list) = config.get_cell_list(sheet_id, r, c) {
                self.cell_options = list.elements.clone();
            }
        }
    }

    /// Reverts the last recorded cell action
    pub async fn undo(&mut self, config: &mut AppConfig) -> Result<()> {
        if let Some(action) = self.undo_stack.pop() {
            self.current_sheet_idx = action.sheet_idx;
            let title = &self.sheets[self.current_sheet_idx].title;
            let addr = gspread_addr(action.row, action.col);
            let range = format!("{}!{}", title, addr);
            
            self.client.update_cell(&range, &action.old_value).await?;
            self.redo_stack.push(action);
            self.load_current_sheet(config).await?;
        }
        Ok(())
    }

    /// Re-applies an action that was previously undone
    pub async fn redo(&mut self, config: &mut AppConfig) -> Result<()> {
        if let Some(action) = self.redo_stack.pop() {
            self.current_sheet_idx = action.sheet_idx;
            let title = &self.sheets[self.current_sheet_idx].title;
            let addr = gspread_addr(action.row, action.col);
            let range = format!("{}!{}", title, addr);
            
            self.client.update_cell(&range, &action.new_value).await?;
            self.undo_stack.push(action);
            self.load_current_sheet(config).await?;
        }
        Ok(())
    }
}

/// Converts numeric row and column indices into Google Sheets address format (e.g., A1, B2)
fn gspread_addr(row: usize, col: usize) -> String {
    let mut s = String::new();
    let mut n = col;
    while n > 0 {
        let m = (n - 1) % 26;
        s.insert(0, (b'A' + m as u8) as char);
        n = (n - 1) / 26;
    }
    format!("{}{}", s, row + 1)
}
