# Peko Technical Documentation

**Version**: 0.1.0  
**Last Updated**: 2026-01-13

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [API Reference](#api-reference)
3. [Data Models](#data-models)
4. [Configuration](#configuration)
5. [Security](#security)
6. [Development Guide](#development-guide)

---

## Architecture Overview

Peko is a Tauri v2 desktop application with a Rust backend and HTML/JavaScript frontend.

```
┌─────────────────────────────────────────────────────────┐
│                    macOS Application                     │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │
│  │   Gemini    │  │ NotebookLM  │  │   Notes     │      │
│  │   Window    │  │   Window    │  │   Panel     │      │
│  │ (WebView)   │  │ (WebView)   │  │ (WebView)   │      │
│  └─────────────┘  └─────────────┘  └─────────────┘      │
├─────────────────────────────────────────────────────────┤
│                    Tauri IPC Bridge                      │
├─────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────┐   │
│  │              Rust Backend (lib.rs)                │   │
│  │  ┌────────────┐ ┌────────────┐ ┌──────────────┐  │   │
│  │  │  Commands  │ │   State    │ │ Menu System  │  │   │
│  │  │  (12 IPC)  │ │ (Mutex)    │ │ (Native)     │  │   │
│  │  └────────────┘ └────────────┘ └──────────────┘  │   │
│  └──────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────┐   │
│  │              File System                          │   │
│  │  ~/Library/Application Support/com.peko.desktop/ │   │
│  │  ├── settings.json                               │   │
│  │  └── webview_*/  (per-site data)                 │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | File | Purpose |
|:----------|:-----|:--------|
| Entry Point | `main.rs` | Application bootstrap |
| Core Logic | `lib.rs` | Commands, state, menu |
| Settings UI | `src/main.js` | Website management |
| Notes Panel | `src/notes.js` | Markdown notes |

---

## API Reference

### Tauri Commands

All commands are invoked via `window.__TAURI__.core.invoke()`.

#### Settings Commands

| Command | Parameters | Returns | Description |
|:--------|:-----------|:--------|:------------|
| `get_settings` | — | `AppSettings` | Get current settings |
| `save_websites` | `websites: Website[]` | `()` | Save website list (max 5) |
| `save_default_website` | `websiteId: string` | `()` | Set default startup site |
| `save_notes` | `content: string` | `()` | Save notes content |
| `get_notes` | — | `string` | Get notes content |

#### Navigation Commands

| Command | Parameters | Returns | Description |
|:--------|:-----------|:--------|:------------|
| `switch_tab` | `tabId: string` | `()` | Switch to specific tab |
| `cycle_tab` | — | `()` | Cycle to next tab |
| `go_back` | — | `()` | Navigate back in history |
| `go_forward` | — | `()` | Navigate forward |

#### Window Commands

| Command | Parameters | Returns | Description |
|:--------|:-----------|:--------|:------------|
| `open_settings` | — | `()` | Open settings window |
| `toggle_notes` | — | `string` | Cycle notes mode, returns new mode |
| `toggle_auto_paste` | — | `bool` | Toggle auto-paste, returns state |

### Usage Example

```javascript
const { invoke } = window.__TAURI__.core;

// Get current settings
const settings = await invoke('get_settings');

// Save websites
await invoke('save_websites', {
  websites: [
    { id: 'gemini', name: 'Gemini', url: 'https://gemini.google.com', emoji: '✨' }
  ]
});

// Switch tabs
await invoke('switch_tab', { tabId: 'gemini' });
```

---

## Data Models

### Website

```typescript
interface Website {
  id: string;      // Unique identifier (e.g., "gemini", "site_1704067200000")
  name: string;    // Display name
  url: string;     // Full URL with protocol
  emoji: string;   // Unicode emoji for tab/menu
}
```

### AppSettings

```typescript
interface AppSettings {
  websites: Website[];           // Max 5 websites
  active_tab: string;            // Currently visible tab ID
  default_website?: string;      // Startup tab ID
  auto_paste_on_focus: boolean;  // Auto-paste clipboard on focus
  notes_content: string;         // Markdown notes content
  notes_mode: "hidden" | "sidebar" | "window";
}
```

### Default Settings

```json
{
  "websites": [
    { "id": "gemini", "name": "Gemini", "url": "https://gemini.google.com/app", "emoji": "✨" },
    { "id": "notebooklm", "name": "NotebookLM", "url": "https://notebooklm.google.com/", "emoji": "📓" }
  ],
  "active_tab": "gemini",
  "default_website": "gemini",
  "auto_paste_on_focus": false,
  "notes_content": "",
  "notes_mode": "hidden"
}
```

---

## Configuration

### File Locations

| File | Path | Purpose |
|:-----|:-----|:--------|
| Settings | `~/Library/Application Support/com.peko.desktop/settings.json` | User preferences |
| Webview Data | `~/Library/Application Support/com.peko.desktop/webview_<id>/` | Per-site cookies, storage |
| Logs | stderr | Runtime logs (env_logger) |

### Tauri Configuration

**`src-tauri/tauri.conf.json`**

```json
{
  "app": {
    "withGlobalTauri": true,
    "security": {
      "csp": "default-src 'self' https:; ..."
    }
  }
}
```

### Capabilities

**`src-tauri/capabilities/default.json`**

Permissions granted to all windows:
- `core:default`, `core:window:*`, `core:webview:*`
- `shell:allow-open` (restricted to `https://`, `http://`, `mailto:`)

---

## Security

### Content Security Policy

```
default-src 'self' https:;
script-src 'self' 'unsafe-inline' 'unsafe-eval' https:;
style-src 'self' 'unsafe-inline' https:;
img-src 'self' data: blob: https:;
connect-src 'self' https: wss:;
frame-src https:;
font-src 'self' data: https:;
```

### Shell Restrictions

Only allowed URL schemes for `shell:allow-open`:
- ✅ `https://**`
- ✅ `http://**`
- ✅ `mailto:*`
- ❌ `file://` (blocked)
- ❌ `javascript:` (blocked)

### Webview Isolation

Each website runs in a separate webview with:
- Isolated storage (cookies, localStorage)
- Separate data directories
- CSP enforcement

---

## Development Guide

### Prerequisites

| Tool | Version | Install |
|:-----|:--------|:--------|
| Rust | ≥1.85 | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Node.js | ≥22 | `brew install node` |
| pnpm | ≥10 | `npm install -g pnpm` |
| Xcode CLI | — | `xcode-select --install` |

### Commands

```bash
# Development with hot reload
pnpm tauri dev

# Production build (app + DMG)
pnpm tauri build

# Run tests
cd src-tauri && cargo test

# Lint Rust code
cd src-tauri && cargo clippy

# Security audit
cd src-tauri && cargo audit
pnpm audit
```

### Adding a New Command

1. Add function in `lib.rs`:
```rust
#[tauri::command]
fn my_command(app: AppHandle, param: String) -> Result<(), String> {
    // Implementation
    Ok(())
}
```

2. Register in `invoke_handler`:
```rust
.invoke_handler(tauri::generate_handler![
    // existing commands...
    my_command
])
```

3. Call from JavaScript:
```javascript
await invoke('my_command', { param: 'value' });
```

### Project Structure

```
peko/
├── src/                      # Frontend
│   ├── index.html            # Settings window
│   ├── notes.html            # Notes panel
│   ├── main.js               # Settings logic
│   ├── notes.js              # Notes logic
│   ├── styles.css            # Main styles
│   └── notes.css             # Notes styles
├── src-tauri/                # Backend
│   ├── Cargo.toml            # Rust dependencies
│   ├── tauri.conf.json       # App config
│   ├── capabilities/         # Permission definitions
│   │   └── default.json
│   ├── icons/                # App icons
│   └── src/
│       ├── main.rs           # Entry point
│       └── lib.rs            # Core logic (860 lines)
├── security/                 # Security artifacts
│   ├── SECURITY_ASSESSMENT.md
│   └── sbom.cdx.json         # CycloneDX SBOM
├── tests/                    # Test artifacts
│   └── test_report.md
└── docs/                     # Documentation
    ├── TECHNICAL.md          # This file
    └── CODE_REVIEW.md        # Code review report
```

---

## Reference

- [Tauri v2 Documentation](https://v2.tauri.app/)
- [Rust API Reference](https://docs.rs/tauri/latest/tauri/)
- [CycloneDX SBOM](./security/sbom.cdx.json)
