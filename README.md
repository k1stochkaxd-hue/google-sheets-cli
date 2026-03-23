# Google Sheets CLI (G-CLI) 📊 🚀

A professional, high-performance terminal interface for Google Sheets built with **Rust**. Manage your spreadsheets like a pro directly from your command line!

## ✨ Key Features

- **OAuth 2.0 Browser login**: Simply log in with your personal Google account. No manual service account setup required for end-users.
- **Lightning Fast Performance**: Built with Rust for maximum efficiency and speed.
- **Full TUI Experience**: Professional terminal interface using an alternate screen buffer (no scrollback clutter).
- **Formula Support**: Standard Google Sheets formula support by prefixing entries with `&?` (e.g., `&?SUM(A1:A10)`).
- **Data Integrity**: Intelligent Undo/Redo system and basic input validation.
- **Persistence**: Remembers your spreadsheet URLs and persists login tokens securely.

## 🚀 Getting Started

### For Users (Download & Run)
1. Go to the [Releases](https://github.com/k1stochkaxd-hue/google-sheets-cli/releases) section.
2. Download the `gcli.exe` executable.
3. Run the executable. It will open a browser window for Google authentication.
4. Paste your Google Sheets URL and start editing!

### 📦 For Developers (Build from Source)
1. **Clone the repository**:
   ```bash
   git clone https://github.com/k1stochkaxd-hue/google-sheets-cli.git
   cd google-sheets-cli
   ```
2. **Setup OAuth**:
   - Create a **Desktop App** OAuth 2.0 Client ID in your [Google Cloud Console](https://console.cloud.google.com/).
   - Download the JSON and save it as `client_secret.json` in the root folder.
3. **Build**:
   ```bash
   cargo build --release
   ```

## ⌨️ Command Reference

| Command | Description |
| --- | --- |
| `1, 2...` | Switch between worksheets (tabs) |
| `l1, l2...` | Select a row |
| `sA, sB...` | Select a column (by letter) |
| `ed <val>` | Edit selected cell (use `&?` for formulas) |
| `del` | Clear selected cell |
| `v <N>` | Select option N from a cell dropdown |
| `new` | Append a new row at the bottom |
| `ns <name>`| Create a new worksheet |
| `rm` | Delete the current worksheet |
| `cz / csz` | Undo / Redo |
| `menu / eq` | Return to spreadsheet selection menu |
| `h` | Show in-app help |
| `exit` | Quit the application |

## 🛠 Tech Stack
- **Rust** (Core logic)
- **reqwest** (API communication)
- **yup-oauth2** (OAuth2 flow)
- **rustyline** (Command line interaction)
- **crossterm** (TUI rendering)

---
Developed by **k1stochkaxd-hue**. Licensed under the [MIT License](LICENSE).
