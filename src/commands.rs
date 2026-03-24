use std::collections::HashSet;

pub enum Command {
    Sheet(usize),
    Row(Option<usize>),
    Col(Option<usize>),
    Edit(String),
    Val(usize),
    Delete,
    New,
    Add(Vec<String>, bool, HashSet<usize>), // Values, is_reverse, skip_columns
    Undo,
    Redo,
    Help,
    Remove,
    Menu,
    NewSheet(String),
    Exit,
    ListLists,
    NewList(Vec<String>, String),
    RemoveList(String),
    EditList(String),
    RowPage(usize),
    ColPage(usize),
}

pub fn parse_command(input: &str) -> Vec<Command> {
    let mut commands = Vec::new();
    let parts: Vec<&str> = input.split_whitespace().collect();
    let mut i = 0;
    while i < parts.len() {
        let part_lower = parts[i].to_lowercase();

        if part_lower == "list" {
            commands.push(Command::ListLists);
        } else if part_lower.starts_with("rr") && part_lower[2..].parse::<usize>().is_ok() {
            let n = part_lower[2..].parse::<usize>().unwrap_or(1);
            commands.push(Command::RowPage(n));
        } else if part_lower.starts_with("cc") && part_lower[2..].parse::<usize>().is_ok() {
            let n = part_lower[2..].parse::<usize>().unwrap_or(1);
            commands.push(Command::ColPage(n));
        } else if part_lower.starts_with("nl") {
            let full_input = parts[i..].join(" ");
            if let (Some(b_start), Some(b_end)) = (full_input.find('<'), full_input.rfind('>')) {
                let inner = &full_input[b_start + 1..b_end];
                let elements: Vec<String> = inner.split(';').map(|s| s.trim().to_string()).collect();
                
                let id = if let (Some(p_start), Some(p_end)) = (full_input.find('('), full_input.rfind(')')) {
                    full_input[p_start + 1..p_end].trim().to_string()
                } else {
                    String::new()
                };
                commands.push(Command::NewList(elements, id));
                break;
            }
        } else if part_lower.starts_with("rl") {
            let id = if part_lower == "rl" {
                parts.get(i + 1).map(|s| s.to_string()).unwrap_or_default()
            } else {
                part_lower[2..].to_string()
            };
            commands.push(Command::RemoveList(id));
        } else if part_lower.starts_with("edl") {
            let id = if part_lower == "edl" {
                parts.get(i + 1).map(|s| s.to_string()).unwrap_or_default()
            } else {
                parts[i][3..].to_string()
            };
            commands.push(Command::EditList(id));
        } else if part_lower.starts_with('l') && part_lower[1..].parse::<usize>().is_ok() {
            let n = part_lower[1..].parse::<usize>().ok();
            commands.push(Command::Row(n));
        } else if part_lower.starts_with('s') && part_lower[1..].parse::<usize>().is_ok() {
            let n = part_lower[1..].parse::<usize>().ok();
            commands.push(Command::Col(n));
        } else if part_lower.starts_with('s') && part_lower.len() > 1 && part_lower[1..].chars().all(|c| c.is_alphabetic()) {
            let col_str = &part_lower[1..];
            let mut col = 0;
            for c in col_str.chars() {
                col = col * 26 + (c as usize - 'a' as usize + 1);
            }
            commands.push(Command::Col(Some(col)));
        } else if part_lower == "v" {
             if let Some(n_str) = parts.get(i+1) {
                 if let Ok(n) = n_str.parse::<usize>() {
                     commands.push(Command::Val(n));
                     i += 1;
                 }
             }
        } else if part_lower.starts_with('v') && part_lower[1..].parse::<usize>().is_ok() {
            let n = part_lower[1..].parse::<usize>().unwrap_or(0);
            commands.push(Command::Val(n));
        } else if part_lower == "ed" {
            let val = parts[i + 1..].join(" ");
            commands.push(Command::Edit(val));
            break;
        } else if part_lower == "del" {
            commands.push(Command::Delete);
        } else if part_lower == "new" {
            commands.push(Command::New);
        } else if part_lower == "add" {
            let full_input = parts[i..].join(" ");
            if let (Some(start), Some(end)) = (full_input.find('<'), full_input.rfind('>')) {
                let mut inner = full_input[start + 1..end].trim().to_string();
                let mut reverse = false;
                if inner.ends_with("(-1)") {
                    reverse = true;
                    inner = inner[..inner.len() - 4].trim().to_string();
                }
                let values: Vec<String> = inner.split(';').map(|s| s.trim().to_string()).collect();
                
                let mut skip_cols = HashSet::new();
                if let (Some(s_start), Some(s_end)) = (full_input.find('['), full_input.find(']')) {
                    let skip_part = &full_input[s_start + 1..s_end];
                    for item in skip_part.split(',') {
                        if item.contains('-') {
                            let range: Vec<&str> = item.split('-').collect();
                            if range.len() == 2 {
                                if let (Ok(s), Ok(e)) = (range[0].trim().parse::<usize>(), range[1].trim().parse::<usize>()) {
                                    for col in s..=e { skip_cols.insert(col); }
                                }
                            }
                        } else if let Ok(col) = item.trim().parse::<usize>() {
                            skip_cols.insert(col);
                        }
                    }
                }
                
                commands.push(Command::Add(values, reverse, skip_cols));
            }
            break;
        } else if part_lower == "ns" {
            let title = parts[i + 1..].join(" ");
            commands.push(Command::NewSheet(title));
            break;
        } else if part_lower == "menu" || part_lower == "eq" {
            commands.push(Command::Menu);
        } else if part_lower == "rm" {
            commands.push(Command::Remove);
        } else if part_lower == "cz" {
            commands.push(Command::Undo);
        } else if part_lower == "csz" {
            commands.push(Command::Redo);
        } else if part_lower == "h" {
            commands.push(Command::Help);
        } else if part_lower == "exit" || part_lower == "quit" {
            commands.push(Command::Exit);
        } else if let Ok(n) = part_lower.parse::<usize>() {
            commands.push(Command::Sheet(n));
        }

        i += 1;
    }
    commands
}
