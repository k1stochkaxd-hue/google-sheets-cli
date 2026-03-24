use crate::auth::Token;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Simplified metadata for a single worksheet (tab) inside a spreadsheet
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SheetMeta {
    pub id: i64,
    pub title: String,
}

/// Root structure for spreadsheet metadata response from Google API
#[derive(Debug, Deserialize)]
pub struct SpreadsheetMetadata {
    pub sheets: Vec<SheetContainer>,
}

#[derive(Debug, Deserialize)]
pub struct SheetContainer {
    pub properties: SheetProperties,
}

#[derive(Debug, Deserialize)]
pub struct SheetProperties {
    #[serde(rename = "sheetId")]
    pub sheet_id: i64,
    pub title: String,
}

/// Client responsible for all HTTP interactions with the Google Sheets API v4
pub struct SheetsClient {
    token: Token,
    spreadsheet_id: String,
    client: reqwest::Client,
}

impl SheetsClient {
    /// Creates a new client instance for a specific spreadsheet
    pub fn new(token: Token, spreadsheet_id: String) -> Self {
        Self {
            token,
            spreadsheet_id,
            client: reqwest::Client::new(),
        }
    }

    /// Helper to generate the Authorization header for requests
    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token.0)
    }

    /// Fetches the list of all worksheets and their basic properties
    pub async fn fetch_metadata(&self) -> Result<Vec<SheetMeta>> {
        let url = format!(
            "https://sheets.googleapis.com/v4/spreadsheets/{}",
            self.spreadsheet_id
        );
        let resp = self
            .client
            .get(url)
            .header("Authorization", self.auth_header())
            .send()
            .await?
            .error_for_status()?
            .json::<SpreadsheetMetadata>()
            .await?;

        let meta = resp
            .sheets
            .into_iter()
            .map(|s| SheetMeta {
                id: s.properties.sheet_id,
                title: s.properties.title,
            })
            .collect();

        Ok(meta)
    }

    /// Retrieves all cell values for a given range (e.g., "Sheet1!A1:Z100")
    pub async fn get_values(&self, range: &str) -> Result<Vec<Vec<String>>> {
        let url = format!(
            "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}",
            self.spreadsheet_id, range
        );
        let resp = self
            .client
            .get(url)
            .header("Authorization", self.auth_header())
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;

        let values = resp["values"].as_array().cloned().unwrap_or_default();
        let rows = values
            .into_iter()
            .map(|row| {
                row.as_array()
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|v| v.as_str().unwrap_or("").to_string())
                    .collect()
            })
            .collect();

        Ok(rows)
    }

    /// Updates the value of a single cell. Uses USER_ENTERED to parse formulas ($?, etc.)
    pub async fn update_cell(&self, range: &str, value: &str) -> Result<()> {
        let url = format!(
            "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}?valueInputOption=USER_ENTERED",
            self.spreadsheet_id, range
        );
        let body = json!({
            "values": [[value]]
        });
        self.client
            .put(url)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Appends a new row at the end of the specified sheet
    pub async fn append_row(&self, sheet_title: &str, values: Vec<String>) -> Result<()> {
        let url = format!(
            "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}!A1:append?valueInputOption=USER_ENTERED",
            self.spreadsheet_id, sheet_title
        );
        let body = json!({
            "values": [values]
        });
        self.client
            .post(url)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Copies data validation rules (like dropdowns) from one row to another
    pub async fn copy_row_validation(
        &self,
        sheet_id: i64,
        from_row: usize,
        to_row: usize,
        cols: usize,
    ) -> Result<()> {
        let url = format!(
            "https://sheets.googleapis.com/v4/spreadsheets/{}:batchUpdate",
            self.spreadsheet_id
        );
        let body = json!({
            "requests": [{
                "copyPaste": {
                    "source": {
                        "sheetId": sheet_id,
                        "startRowIndex": from_row - 1,
                        "endRowIndex": from_row,
                        "startColumnIndex": 0,
                        "endColumnIndex": cols
                    },
                    "destination": {
                        "sheetId": sheet_id,
                        "startRowIndex": to_row - 1,
                        "endRowIndex": to_row,
                        "startColumnIndex": 0,
                        "endColumnIndex": cols
                    },
                    "pasteType": "PASTE_DATA_VALIDATION"
                }
            }]
        });
        self.client
            .post(url)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Fetches full metadata for a specific range, including UI properties like dropdowns
    pub async fn get_cell_metadata(&self, range: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://sheets.googleapis.com/v4/spreadsheets/{}?includeGridData=true&ranges={}",
            self.spreadsheet_id, range
        );
        let resp = self
            .client
            .get(url)
            .header("Authorization", self.auth_header())
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;
        Ok(resp)
    }

    /// Permanently deletes a worksheet (tab) by its numeric ID
    pub async fn delete_sheet(&self, sheet_id: i64) -> Result<()> {
        let url = format!(
            "https://sheets.googleapis.com/v4/spreadsheets/{}:batchUpdate",
            self.spreadsheet_id
        );
        let body = json!({
            "requests": [{
                "deleteSheet": {
                    "sheetId": sheet_id
                }
            }]
        });
        self.client
            .post(url)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Creates a new, empty worksheet with the specified title
    pub async fn add_sheet(&self, title: &str) -> Result<()> {
        let url = format!(
            "https://sheets.googleapis.com/v4/spreadsheets/{}:batchUpdate",
            self.spreadsheet_id
        );
        let body = json!({
            "requests": [{
                "addSheet": {
                    "properties": {
                        "title": title
                    }
                }
            }]
        });
        self.client.post(url)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Fetches the entire sheet structure including all cell values and metadata (like drop-downs)
    pub async fn get_sheet_full(&self, sheet_title: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://sheets.googleapis.com/v4/spreadsheets/{}?ranges='{}'&includeGridData=true",
            self.spreadsheet_id,
            sheet_title
        );

        let resp = self.client.get(url)
            .header("Authorization", self.auth_header())
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok(resp)
    }

    /// Sets a data validation rule (dropdown list) for a specific cell
    pub async fn set_data_validation(
        &self,
        sheet_id: i64,
        row: usize,
        col: usize,
        values: Vec<String>,
    ) -> Result<()> {
        let url = format!(
            "https://sheets.googleapis.com/v4/spreadsheets/{}:batchUpdate",
            self.spreadsheet_id
        );

        let condition_values: Vec<serde_json::Value> = values
            .into_iter()
            .map(|v| json!({ "userEnteredValue": v }))
            .collect();

        let body = json!({
            "requests": [
                {
                    "setDataValidation": {
                        "range": {
                            "sheetId": sheet_id,
                            "startRowIndex": row,
                            "endRowIndex": row + 1,
                            "startColumnIndex": col - 1,
                            "endColumnIndex": col
                        },
                        "rule": {
                            "condition": {
                                "type": "ONE_OF_LIST",
                                "values": condition_values
                            },
                            "showCustomUi": true,
                            "strict": true
                        }
                    }
                }
            ]
        });

        self.client.post(url)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
