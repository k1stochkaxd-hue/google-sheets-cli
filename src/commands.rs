pub enum Command {
    Sheet(usize),
    Row(Option<usize>),
    Col(Option<usize>),
    Edit(String),
    Val(usize),
    Delete,
    New,
    Add(Vec<String>, bool), // Values, is_reverse
    Undo,
    Redo,
    Help,
    Remove,
    Menu,
    NewSheet(String),
    Exit,
}

pub fn parse_command(input: &str) -> Vec<Command> {
    let mut commands = Vec::new();
    let parts: Vec<&str> = input.split_whitespace().collect();
    let mut i = 0;
    while i < parts.len() {
        let part = parts[i].to_lowercase();

        if part.starts_with('l') {
            let num = part[1..].parse::<usize>().ok();
            if num.is_none() && i + 1 < parts.len() {
                if let Ok(n) = parts[i + 1].parse::<usize>() {
                    commands.push(Command::Row(Some(n)));
                    i += 1;
                } else {
                    commands.push(Command::Row(None));
                }
            } else {
                commands.push(Command::Row(num));
            }
        } else if part.starts_with('s') {
            let rest = &part[1..];
            let mut col_val = None;

            if !rest.is_empty() {
                if let Ok(n) = rest.parse::<usize>() {
                    col_val = Some(n);
                } else {
                    col_val = letter_to_id(rest);
                }
            } else if i + 1 < parts.len() {
                let next = parts[i + 1].to_uppercase();
                if let Ok(n) = next.parse::<usize>() {
                    col_val = Some(n);
                    i += 1;
                } else if let Some(n) = letter_to_id(&next) {
                    col_val = Some(n);
                    i += 1;
                }
            }
            commands.push(Command::Col(col_val));
        } else if part.starts_with('v') {
            let num = part[1..].parse::<usize>().ok();
            if num.is_none() && i + 1 < parts.len() {
                if let Ok(n) = parts[i + 1].parse::<usize>() {
                    commands.push(Command::Val(n));
                    i += 1;
                }
            } else if let Some(n) = num {
                commands.push(Command::Val(n));
            }
        } else if part == "ed" {
            let val = parts[i + 1..].join(" ");
            commands.push(Command::Edit(val));
            break;
        } else if part == "del" {
            commands.push(Command::Delete);
        } else if part == "new" {
            commands.push(Command::New);
        } else if part == "add" {
            let full_input = parts[i..].join(" ");
            if let (Some(start), Some(end)) = (full_input.find('<'), full_input.rfind('>')) {
                let mut inner = full_input[start + 1..end].trim().to_string();
                let mut reverse = false;
                if inner.ends_with("(-1)") {
                    reverse = true;
                    inner = inner[..inner.len() - 4].trim().to_string();
                }
                let values: Vec<String> = inner.split(';').map(|s| s.trim().to_string()).collect();
                commands.push(Command::Add(values, reverse));
            }
            break;
        } else if part == "ns" {
            let title = parts[i + 1..].join(" ");
            commands.push(Command::NewSheet(title));
            break;
        } else if part == "menu" || part == "eq" {
            commands.push(Command::Menu);
        } else if part == "rm" {
            commands.push(Command::Remove);
        } else if part == "cz" {
            commands.push(Command::Undo);
        } else if part == "csz" {
            commands.push(Command::Redo);
        } else if part == "h" {
            commands.push(Command::Help);
        } else if part == "exit" || part == "quit" {
            commands.push(Command::Exit);
        } else if let Ok(n) = part.parse::<usize>() {
            commands.push(Command::Sheet(n));
        }

        i += 1;
    }
    commands
}

fn letter_to_id(s: &str) -> Option<usize> {
    let mut n = 0;
    for c in s.chars() {
        if !c.is_alphabetic() {
            return None;
        }
        n = n * 26 + (c.to_ascii_uppercase() as usize - 'A' as usize + 1);
    }
    if n == 0 {
        None
    } else {
        Some(n)
    }
}
