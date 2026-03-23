mod app;
mod auth;
mod commands;
mod config;
mod sheets;
mod table;

use crate::app::App;
use crate::commands::{parse_command, Command};
use crate::config::AppConfig;
use crate::sheets::SheetsClient;
use crate::table::render_table;
use anyhow::Result;
use colored::*;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, execute, terminal};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<()> {
    // Enter alternate screen to prevent terminal scrolling pollution
    execute!(io::stdout(), EnterAlternateScreen)?;

    let res = run_app().await;

    // Restore the terminal state upon exit
    execute!(io::stdout(), LeaveAlternateScreen)?;
    
    res
}

/// Main application entry point that manages the connection and sheet switching
async fn run_app() -> Result<()> {
    let mut config = AppConfig::load();

    loop {
        // Step 1: Show the spreadsheet selection menu
        let selected_url = match select_spreadsheet(&mut config) {
            Some(url) => url,
            None => break, // Exit if user quits the menu
        };

        show_splash();
        println!("{}", "Connecting to Google Sheets...".yellow());
        
        // Authenticate via OAuth 2.0 (opens browser)
        let token = match auth::get_token().await {
            Ok(t) => t,
            Err(e) => {
                println!("{} {}", "Authentication Error:".red(), e);
                pause();
                continue;
            }
        };

        // Extract spreadsheet ID from the provided URL
        let spreadsheet_id = match extract_id(&selected_url) {
            Some(id) => id,
            None => {
                println!("{}", "Error: Invalid Spreadsheet URL format".red());
                pause();
                continue;
            }
        };

        // Initialize the API client and application state
        let client = SheetsClient::new(token, spreadsheet_id);
        let mut app = match App::new(client).await {
            Ok(a) => a,
            Err(e) => {
                println!("{} {}", "Initialization Error:".red(), e);
                pause();
                continue;
            }
        };
        app.load_current_sheet().await?;

        let mut rl = rustyline::DefaultEditor::new()?;
        let mut return_to_menu = false;
        
        // Inner loop: Interacting with the selected spreadsheet
        loop {
            clear_screen();
            render_table(&app);

            let prompt = format!(
                "(Sheet {}/{}) > ",
                app.current_sheet_idx + 1,
                app.sheets.len()
            );
            let readline = rl.readline(&prompt);

            match readline {
                Ok(line) => {
                    rl.add_history_entry(line.as_str())?;
                    let cmds = parse_command(&line);

                    let mut should_exit = false;
                    for cmd in cmds {
                        // Capture execution results to prevent app-wide crashes
                        let res = match cmd {
                            Command::Menu => {
                                return_to_menu = true;
                                should_exit = true;
                                Ok(())
                            }
                            Command::Sheet(n) => {
                                if n > 0 && n <= app.sheets.len() {
                                    app.current_sheet_idx = n - 1;
                                    app.load_current_sheet().await
                                } else {
                                    Ok(())
                                }
                            }
                            Command::Row(n) => {
                                app.selected_row = n;
                                Ok(())
                            }
                            Command::Col(n) => {
                                app.selected_col = n;
                                Ok(())
                            }
                            Command::Edit(mut val) => {
                                if val.starts_with("&?") {
                                    val = val.replacen("&?", "=", 1);
                                }
                                if let (Some(r), Some(c)) = (app.selected_row, app.selected_col) {
                                    app.apply_change(r, c, val, true).await
                                } else {
                                    Ok(())
                                }
                            }
                            Command::Val(n) => {
                                if let (Some(r), Some(c)) = (app.selected_row, app.selected_col) {
                                    if n > 0 && n <= app.cell_options.len() {
                                        let val = app.cell_options[n - 1].clone();
                                        app.apply_change(r, c, val, true).await
                                    } else {
                                        Ok(())
                                    }
                                } else {
                                    Ok(())
                                }
                            }
                            Command::Delete => {
                                if let (Some(r), Some(c)) = (app.selected_row, app.selected_col) {
                                    app.apply_change(r, c, "".to_string(), true).await
                                } else {
                                    Ok(())
                                }
                            }
                            Command::New => {
                                let title = &app.sheets[app.current_sheet_idx].title;
                                let sheet_id = app.sheets[app.current_sheet_idx].id;
                                let cols = if app.data.is_empty() {
                                    0
                                } else {
                                    app.data[0].len()
                                };
                                let res = app
                                    .client
                                    .append_row(title, vec!["".to_string(); cols])
                                    .await;
                                if res.is_err() {
                                    return res;
                                }

                                let from_row = app.data.len();
                                let to_row = from_row + 1;
                                if from_row >= 1 {
                                    app.client
                                        .copy_row_validation(sheet_id, from_row, to_row, cols)
                                        .await
                                        .ok();
                                }
                                app.load_current_sheet().await
                            }
                            Command::Add(values, reverse) => {
                                let row = app.selected_row.unwrap_or(1);
                                let max_cols = if !app.data.is_empty() { app.data[0].len() } else { 26 };
                                
                                for (i, mut val) in values.into_iter().enumerate() {
                                    if val.starts_with("&?") {
                                        val = val.replacen("&?", "=", 1);
                                    }
                                    
                                    let col = if reverse {
                                        if max_cols > i { max_cols - i } else { 1 }
                                    } else {
                                        i + 1
                                    };

                                    if col > 0 && col <= max_cols {
                                        app.apply_change(row, col, val, true).await?;
                                    }
                                }
                                app.load_current_sheet().await
                            }
                            Command::Undo => app.undo().await,
                            Command::Redo => app.redo().await,
                            Command::Help => {
                                show_help();
                                pause();
                                Ok(())
                            }
                            Command::NewSheet(title) => {
                                let res = app.client.add_sheet(&title).await;
                                if res.is_err() {
                                    return res;
                                }
                                app.sheets = app.client.fetch_metadata().await?;
                                if let Some(pos) = app.sheets.iter().position(|s| s.title == title)
                                {
                                    app.current_sheet_idx = pos;
                                } else {
                                    app.current_sheet_idx = app.sheets.len() - 1;
                                }
                                app.load_current_sheet().await
                            }
                            Command::Remove => {
                                if app.sheets.len() <= 1 {
                                    println!("{}", "Error: Cannot delete the only worksheet.".red());
                                    pause();
                                    Ok(())
                                } else {
                                    let title = &app.sheets[app.current_sheet_idx].title;
                                    println!("Confirm deletion of '{}'? (y/n): ", title);
                                    io::stdout().flush().ok();
                                    let mut conf = String::new();
                                    io::stdin().read_line(&mut conf).ok();
                                    if conf.trim().to_lowercase() == "y" {
                                        let id = app.sheets[app.current_sheet_idx].id;
                                        app.client.delete_sheet(id).await?;
                                        app.sheets = app.client.fetch_metadata().await?;
                                        app.current_sheet_idx = 0;
                                        app.load_current_sheet().await
                                    } else {
                                        Ok(())
                                    }
                                }
                            }
                            Command::Exit => {
                                return_to_menu = false;
                                should_exit = true;
                                Ok(())
                            }
                        };

                        if let Err(e) = res {
                            println!("{} {}", "Action Failed:".red().bold(), e);
                            pause();
                        }
                    }

                    if should_exit {
                        break;
                    }

                    // Refresh dropdown options for the selected cell
                    if app.selected_row.is_some() && app.selected_col.is_some() {
                        if let Err(e) = app.fetch_options().await {
                            println!("{} {}", "Warning (options):".red(), e);
                        }
                    }
                }
                Err(_) => {
                    return_to_menu = false;
                    break;
                }
            }
        }

        if !return_to_menu {
            break;
        }
    }

    Ok(())
}

/// Renders a keyboard-navigable menu for selecting stored Spreadsheet configurations
fn select_spreadsheet(config: &mut AppConfig) -> Option<String> {
    enable_raw_mode().ok();
    
    // Flush input buffer to prevent double-Enter accidental selection
    std::thread::sleep(std::time::Duration::from_millis(150));
    while let Ok(true) = event::poll(std::time::Duration::from_millis(50)) {
        let _ = event::read();
    }

    let mut selected_idx = 0;
    
    let res = loop {
        clear_screen();
        show_splash();
        println!("{}", "--- SELECT SPREADSHEET ---".cyan().bold());
        
        for (i, sheet) in config.spreadsheets.iter().enumerate() {
            if i == selected_idx {
                println!(" {} {}", " > ".yellow(), sheet.name.yellow().bold());
            } else {
                println!("   {}", sheet.name);
            }
        }
        
        let new_idx = config.spreadsheets.len();
        if selected_idx == new_idx {
            println!(" {} {}", " > ".green(), "[+] Add New Spreadsheet".green().bold());
        } else {
            println!("   {}", "[+] Add New Spreadsheet".dimmed());
        }

        println!("\n (Arrow Keys to move, Enter to Select)");
        println!(" (Delete/Backspace to Remove Item)");

        match event::read() {
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Up,
                kind: event::KeyEventKind::Press,
                ..
            })) => {
                if selected_idx > 0 { 
                    selected_idx -= 1;
                    std::thread::sleep(std::time::Duration::from_millis(100)); // Smooth scrolling
                }
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Down,
                kind: event::KeyEventKind::Press,
                ..
            })) => {
                if selected_idx < config.spreadsheets.len() { 
                    selected_idx += 1;
                    std::thread::sleep(std::time::Duration::from_millis(100)); // Smooth scrolling
                }
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Delete,
                kind: event::KeyEventKind::Press,
                ..
            }) | Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                kind: event::KeyEventKind::Press,
                ..
            })) => {
                if selected_idx < config.spreadsheets.len() {
                    config.remove(selected_idx).ok();
                    // Prevent rapid-fire deletion with a slight pause
                    std::thread::sleep(std::time::Duration::from_millis(250));
                    // Adjust index if we just deleted the last item
                    if selected_idx >= config.spreadsheets.len() && selected_idx > 0 {
                        selected_idx -= 1;
                    }
                }
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: event::KeyEventKind::Press,
                ..
            })) => {
                if selected_idx == config.spreadsheets.len() {
                    disable_raw_mode().ok();
                    println!("\nName: ");
                    let mut name = String::new();
                    io::stdin().read_line(&mut name).ok();
                    println!("URL: ");
                    let mut url = String::new();
                    io::stdin().read_line(&mut url).ok();
                    
                    let name = name.trim().to_string();
                    let url = url.trim().to_string();
                    if !url.is_empty() {
                        config.add(name, url).ok();
                    }
                    enable_raw_mode().ok();
                    // Small delay after adding new to prevent accidental selection
                    std::thread::sleep(std::time::Duration::from_millis(200));
                } else {
                    break Some(config.spreadsheets[selected_idx].url.clone())
                }
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Char('q'), ..
            })) => {
                break None;
            }
            _ => (),
        };
    };
    
    disable_raw_mode().ok();
    res
}

/// Displays the application banner (splash screen)
fn show_splash() {
    clear_screen();
    println!(
        "{}",
        r#"
 ██████╗ ██████╗  ██████╗ ██████╗ ██╗     ███████╗███████╗
██╔════╝ ██╔══██╗██╔════╝ ██╔══██╗██║     ██╔════╝██╔════╝
██║  ███╗██████╔╝██║  ███╗██████╔╝██║     █████╗  ███████╗ 
██║   ██║██╔══██╗██║   ██║██╔══██╗██║     ██╔══╝  ╚════██║
╚██████╔╝██║  ██║╚██████╔╝██║  ██║███████╗███████╗███████║
 ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝
    "#
        .green()
        .bold()
    );
    println!("{}", "Welcome to Google Sheets CLI!".blue().bold());
}

/// Displays the command reference (Help)
fn show_help() {
    println!("{}", "--- G-CLI HELP REFERENCE ---".cyan().bold());
    println!("1, 2...    - Switch Worksheet");
    println!("l<N>       - Select Row N");
    println!("s<N>       - Select Column (by Letter)");
    println!("ed <val>   - Edit cell (Formula prefix: &?)");
    println!("v <N>      - Select from dropdown value N");
    println!("del        - Clear selected cell");
    println!("new        - Append new row at bottom");
    println!("add <x;y;z>  - Fill selected row items from Left to Right");
    println!("add <x;y (-1)>- Fill selected row items from Right to Left");
    println!("ns <name>  - Create a new Sheet (tab)");
    println!("rm         - Delete current Sheet (tab)");
    println!("cz / csz   - Undo / Redo");
    println!("menu / eq  - Return to spreadsheet selection");
    println!("h          - Show this help reference");
    println!("exit       - Exit the application");
}

/// Standard full-screen clear for the terminal
fn clear_screen() {
    execute!(
        io::stdout(),
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    ).ok();
    io::stdout().flush().ok();
}

/// Blocking wait for any keypress to continue
fn pause() {
    print!(" Press Enter to continue...");
    io::stdout().flush().unwrap();
    let mut _s = String::new();
    io::stdin().read_line(&mut _s).unwrap();
}

/// Parsing function to extract Spreadsheet ID from various URL formats
fn extract_id(url: &str) -> Option<String> {
    if let Some(pos) = url.find("/d/") {
        let sub = &url[pos + 3..];
        if let Some(end) = sub.find('/') {
            return Some(sub[..end].to_string());
        } else {
            return Some(sub.to_string());
        }
    }
    None
}
