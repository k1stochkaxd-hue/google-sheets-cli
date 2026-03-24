use crate::app::App;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use colored::*;

/// Renders the current spreadsheet data into a formatted terminal table
pub fn render_table(app: &App) {
    if app.data.is_empty() {
        println!("{}", "No data available on this sheet.".yellow());
        return;
    }

    let num_rows = app.data.len();
    let num_cols = if num_rows > 0 { app.data[0].len() } else { 0 };

    // Calculate row window
    let start_row = (app.row_page - 1) * app.page_size_rows;
    // Always keep header (row 0) if it's the first page, otherwise we'll show data from start_row
    let effective_start_row = if app.row_page == 1 { 1 } else { start_row };
    let end_row = (start_row + app.page_size_rows).min(num_rows);

    // Calculate column window
    let start_col = (app.col_page - 1) * app.page_size_cols;
    let end_col = (start_col + app.page_size_cols).min(num_cols);

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);

    // Build header row: # + sliced columns
    let mut header_cells = vec![Cell::new("#").add_attribute(Attribute::Bold).fg(Color::Cyan)];
    if !app.data.is_empty() {
        for c in start_col..end_col {
            let col_letter = get_column_letter(c + 1);
            let header_text = format!("({}) {}", col_letter, app.data[0].get(c).unwrap_or(&String::new()));
            header_cells.push(Cell::new(header_text).add_attribute(Attribute::Bold).fg(Color::Green));
        }
    }
    table.set_header(header_cells);

    // Build data rows for the current window
    for r_idx in effective_start_row..end_row {
        let row_data = &app.data[r_idx];
        let mut row_cells = vec![Cell::new(r_idx.to_string()).fg(Color::Cyan)];
        
        for c in start_col..end_col {
            let mut cell_content = row_data.get(c).cloned().unwrap_or_default();
            
            // Limit cell size to prevent UI breaking, while allowing dynamic arrangement
            if cell_content.len() > 30 {
                cell_content = format!("{}...", &cell_content[..27]);
            }

            let mut cell = Cell::new(cell_content);
            
            // Highlight selected cell
            if Some(r_idx) == app.selected_row && Some(c + 1) == app.selected_col {
                cell = cell.bg(Color::Yellow).fg(Color::Black);
            } else if Some(r_idx) == app.selected_row {
                cell = cell.bg(Color::DarkGrey);
            }
            
            row_cells.push(cell);
        }
        table.add_row(row_cells);
    }

    // Display Sheet Title and Pagination Status
    let total_row_pages = (num_rows as f64 / app.page_size_rows as f64).ceil() as usize;
    let total_col_pages = (num_cols as f64 / app.page_size_cols as f64).ceil() as usize;
    
    let title = &app.sheets[app.current_sheet_idx].title;
    println!("\n    {}", format!(" Sheet: {} ", title).on_green().black().bold());
    println!("    {}", format!("Row Page: {}/{} | Col Page: {}/{} (Total Rows: {}, Cols: {})", 
        app.row_page, total_row_pages, app.col_page, total_col_pages, num_rows, num_cols).dimmed());

    // Display Dropdown Options Hint if available for selected cell
    let mut options_line = String::new();
    if !app.cell_options.is_empty() {
        options_line.push_str(&format!("{} ", " 💡 Options:".yellow().bold()));
        for (i, opt) in app.cell_options.iter().enumerate() {
            options_line.push_str(&format!("{}:{} ", format!("v{}", i + 1).cyan().bold(), opt));
        }
        println!("{}\n", options_line);
    }

    println!("{table}");
}

/// Converts column index (1-based) to Excel-style letters (A, B, C... AA, AB...)
fn get_column_letter(n: usize) -> String {
    let mut s = String::new();
    let mut num = n;
    while num > 0 {
        let m = (num - 1) % 26;
        s.insert(0, (b'A' + m as u8) as char);
        num = (num - 1) / 26;
    }
    s
}
