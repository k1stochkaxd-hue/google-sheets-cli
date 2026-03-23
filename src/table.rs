use comfy_table::{Table, Row, Cell, Color, Attribute};
use crossterm::{terminal, style::{Stylize}};
use crate::app::App;

pub fn render_table(app: &App) {
    let mut table = Table::new();
    let title = format!(" Sheet: {} ", app.sheets[app.current_sheet_idx].title);
    table.set_header(vec![
        Cell::new("#").add_attribute(Attribute::Dim),
    ]);

    if app.data.is_empty() {
        println!("No data available");
        return;
    }

    let headers = &app.data[0];
    let mut header_row = vec![Cell::new("#").add_attribute(Attribute::Dim)];
    for (i, h) in headers.iter().enumerate() {
        let col_id = i + 1;
        let letter = col_letter(i);
        let mut cell = Cell::new(format!("({}) {}", letter, h));
        if Some(col_id) == app.selected_col {
            cell = cell.fg(Color::Yellow).add_attribute(Attribute::Bold);
        } else {
            cell = cell.fg(Color::Cyan);
        }
        header_row.push(cell);
    }
    table.set_header(header_row);

    for (r_idx, row_data) in app.data.iter().enumerate().skip(1) {
        let row_id = r_idx;
        let mut row = Row::new();
        
        let mut id_cell = Cell::new(row_id.to_string());
        if Some(row_id) == app.selected_row {
            id_cell = id_cell.bg(Color::Blue).add_attribute(Attribute::Bold);
        }
        row.add_cell(id_cell);

        for (c_idx, val) in row_data.iter().enumerate() {
            let col_id = c_idx + 1;
            let mut cell = Cell::new(val);
            
            if Some(row_id) == app.selected_row && Some(col_id) == app.selected_col {
                cell = cell.fg(Color::Black).bg(Color::Yellow).add_attribute(Attribute::Bold);
            } else if Some(row_id) == app.selected_row {
                cell = cell.add_attribute(Attribute::Bold).fg(Color::Blue);
            }
            
            row.add_cell(cell);
        }
        table.add_row(row);
    }

    let mut caption = String::new();
    if !app.cell_options.is_empty() {
        caption.push_str("Dropdown options: ");
        for (i, opt) in app.cell_options.iter().enumerate() {
            caption.push_str(&format!("{}: {}  ", (i + 1).to_string().yellow().bold(), opt));
        }
        caption.push_str(&format!("(use {})", "v <num>".green().bold()));
    }

    // Center the table
    let (width, _) = terminal::size().unwrap_or((80, 24));
    let table_str = table.to_string();
    let table_lines: Vec<&str> = table_str.lines().collect();
    
    // Print title
    let title_padding = (width as usize).saturating_sub(title.len()) / 2;
    println!("{}{}", " ".repeat(title_padding), title.green().bold());

    for line in table_lines {
        let stripped_len = strip_ansi_escapes::strip(line).len();
        let padding = (width as usize).saturating_sub(stripped_len) / 2;
        println!("{}{}", " ".repeat(padding), line);
    }
    
    if !caption.is_empty() {
        let caption_padding = (width as usize).saturating_sub(strip_ansi_escapes::strip(&caption).len()) / 2;
        println!("{}{}", " ".repeat(caption_padding), caption);
    }
}

mod strip_ansi_escapes {
    pub fn strip(s: &str) -> Vec<u8> {
        let mut res = Vec::new();
        let mut skip = false;
        let mut i = 0;
        let bytes = s.as_bytes();
        while i < bytes.len() {
            if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i+1] == b'[' {
                skip = true;
                i += 2;
                continue;
            }
            if skip {
                if (bytes[i] >= b'A' && bytes[i] <= b'Z') || (bytes[i] >= b'a' && bytes[i] <= b'z') {
                    skip = false;
                }
                i += 1;
                continue;
            }
            res.push(bytes[i]);
            i += 1;
        }
        res
    }
}

fn col_letter(idx: usize) -> String {
    let mut s = String::new();
    let mut n = idx + 1;
    while n > 0 {
        let m = (n - 1) % 26;
        s.insert(0, (b'A' + m as u8) as char);
        n = (n - 1) / 26;
    }
    s
}
