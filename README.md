# 🚀 Peko

**Turn any webpage into a desktop app** - A lightweight Tauri v2 wrapper for macOS with multi-tab support.

## ✨ Features

- 🎐 **Lightweight** - ~5MB vs ~100MB for Electron apps
- 🚀 **Fast** - Built with Rust + Tauri v2, low memory usage
- 📑 **Multi-Tab** - Switch between up to 5 websites
- 💾 **Persistent Login** - Each site keeps you logged in
- ⚡ **Native Menu Bar** - macOS native tabs and settings

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `⌘ 1-5` | Switch to tab 1-5 |
| `⌘ Tab` | Cycle to next tab |
| `⌘ ,` | Open Settings |

## 📦 Installation

### Download (Recommended)
Download the latest `.dmg` from [Releases](https://github.com/your-repo/peko/releases).

### Build from Source

**Prerequisites:**
- Rust ≥ 1.85 ([rustup.rs](https://rustup.rs))
- Node.js ≥ 22 ([nodejs.org](https://nodejs.org))
- pnpm ≥ 10 (`npm install -g pnpm`)
- Xcode CLI Tools (`xcode-select --install`)

```bash
# Clone and install
git clone https://github.com/your-repo/peko.git
cd peko
pnpm install

# Development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

The built app is at `src-tauri/target/release/bundle/macos/Peko.app`

## 🔧 Configuration

### Default Websites
Peko comes pre-configured with:
1. ✨ **Gemini** - https://gemini.google.com/app
2. 📓 **NotebookLM** - https://notebooklm.google.com

### Adding Custom Websites
1. Press `⌘ ,` or go to **Peko → Settings**
2. Click **Add Website**
3. Enter name, URL, and choose an emoji
4. Click **Save & Close**

Settings are stored in `~/Library/Application Support/com.peko.app/settings.json`

## 📁 Project Structure

```
peko/
├── src/                  # Frontend (Settings UI)
│   ├── index.html        # Settings window
│   ├── styles.css        # Dark theme styling
│   └── main.js           # Settings logic
└── src-tauri/            # Backend (Rust)
    ├── Cargo.toml        # Rust dependencies
    ├── tauri.conf.json   # App configuration
    └── src/
        ├── main.rs       # Entry point
        └── lib.rs        # Menu, windows, commands
```

## 🏗️ Architecture

Peko uses a hybrid architecture combining a secure Rust backend with a lightweight web frontend.

```mermaid
graph TD
    subgraph Host[Host OS (macOS)]
        subgraph RustBackend[Rust Backend (src-tauri)]
            Main[main.rs] --> Lib[lib.rs]
            Lib --> State[Shared State<br>AppSettings]
            Lib --> Commands[Commands]
            Commands --> GetSet[get_settings<br>save_websites]
            Commands --> WinMgmt[switch_tab<br>cycle_tab]
        end
        
        subgraph Windows[Webview Windows]
            Settings[Settings Window<br>src/index.html]
            Gemini[Gemini Window<br>External URL]
            NotebookLM[NotebookLM Window<br>External URL]
        end
    end

    Settings -- IPC --> Commands
    Commands -- Create/Manage --> Windows
    State -- Persist --> File[settings.json]
```

### Core Components

1. **Rust Backend (`lib.rs`)**
   - **State Management**: Handles active tab tracking and website configuration using `Mutex<AppSettings>`.
   - **Window Manager**: Dynamically creates/destroys windows based on settings. Each website runs in its own isolated webview.
   - **IPC Commands**: Exposes functions like `save_websites` and `switch_tab` to the frontend context.
   - **Menu System**: Native macOS menu bar integration for tab switching and controls.

2. **Frontend (`src/`)**
   - **Settings UI**: A lightweight HTML/JS interface for managing websites.
   - **IPC Bridge**: Uses `window.__TAURI__.core.invoke` to communicate with Rust.
   - **Asset Handling**: Bundled minimal CSS/JS resources.

3. **Security Model**
   - **Capabilities**: Using `capabilities/default.json` to strictly define permissions.
   - **Isolation**: External websites (Google, etc.) are loaded in restricted webviews.
   - **Permissions**: `shell:allow-open` is restricted to specific URL schemes.

## 🛠️ Development

```bash
# Run with hot reload
pnpm tauri dev

# Check Rust code
cd src-tauri && cargo check

# Format code
cargo fmt
```

## 📋 License

MIT

---

**Inspired by** [Pake](https://github.com/tw93/Pake) and [Tauri](https://tauri.app)
