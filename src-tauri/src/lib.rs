use tauri::{
    Manager, WebviewUrl, WebviewWindowBuilder,
    menu::{Menu, MenuItem, Submenu, PredefinedMenuItem, MenuEvent, CheckMenuItem},
    AppHandle, WindowEvent,
};
use std::fs;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Website {
    pub id: String,
    pub name: String,
    pub url: String,
    pub emoji: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub websites: Vec<Website>,
    pub active_tab: String,
    #[serde(default)]
    pub default_website: Option<String>,
    #[serde(default)]
    pub auto_paste_on_focus: bool,
    #[serde(default)]
    pub notes_content: String,
    #[serde(default = "default_notes_mode")]
    pub notes_mode: String,  // "hidden", "sidebar", "window"
}

fn default_notes_mode() -> String {
    "hidden".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            websites: vec![
                Website {
                    id: "gemini".to_string(),
                    name: "Gemini".to_string(),
                    url: "https://gemini.google.com/app".to_string(),
                    emoji: "✨".to_string(),
                },
                Website {
                    id: "notebooklm".to_string(),
                    name: "NotebookLM".to_string(),
                    url: "https://notebooklm.google.com/".to_string(),
                    emoji: "📓".to_string(),
                },
            ],
            active_tab: "gemini".to_string(),
            default_website: Some("gemini".to_string()),
            auto_paste_on_focus: false,
            notes_content: String::new(),
            notes_mode: "hidden".to_string(),
        }
    }
}

struct SettingsState(Mutex<AppSettings>);
static AUTO_PASTE_ENABLED: AtomicBool = AtomicBool::new(false);

fn get_settings_path(app: &AppHandle) -> std::path::PathBuf {
    app.path().app_data_dir()
        .expect("Failed to get app data directory")
        .join("settings.json")
}

fn load_settings(app: &AppHandle) -> AppSettings {
    let path = get_settings_path(app);
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str(&content) {
                return settings;
            }
        }
    }
    AppSettings::default()
}

fn save_settings_to_file(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = get_settings_path(app);
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    fs::write(&path, content)
        .map_err(|e| format!("Failed to write settings: {}", e))?;
    Ok(())
}

#[tauri::command]
fn get_settings(app: AppHandle) -> Result<AppSettings, String> {
    let state = app.state::<SettingsState>();
    let settings = state.0.lock().unwrap();
    Ok(settings.clone())
}

#[tauri::command]
fn save_websites(app: AppHandle, websites: Vec<Website>) -> Result<(), String> {
    if websites.len() > 5 {
        return Err("Maximum 5 websites allowed".to_string());
    }
    
    let state = app.state::<SettingsState>();
    let mut settings = state.0.lock().unwrap();
    
    let old_ids: Vec<String> = settings.websites.iter().map(|w| w.id.clone()).collect();
    let new_ids: Vec<String> = websites.iter().map(|w| w.id.clone()).collect();
    
    // Close windows for removed websites
    for old_id in &old_ids {
        if !new_ids.contains(old_id) {
            if let Some(window) = app.get_webview_window(old_id) {
                let _ = window.close();
            }
        }
    }
    
    // Create windows for new websites
    let app_data_dir = app.path().app_data_dir().unwrap();
    for website in &websites {
        if !old_ids.contains(&website.id) {
            let data_dir = app_data_dir.join(format!("webview_{}", website.id));
            let _ = fs::create_dir_all(&data_dir);
            
            let _ = WebviewWindowBuilder::new(
                &app,
                &website.id,
                WebviewUrl::External(website.url.parse().unwrap())
            )
            .title(format!("Peko - {}", website.name))
            .inner_size(1200.0, 800.0)
            .min_inner_size(600.0, 400.0)
            .resizable(true)
            .decorations(true)
            .visible(false)
            .data_directory(data_dir)
            .build();
        }
    }
    
    settings.websites = websites;
    
    if !new_ids.contains(&settings.active_tab) && !new_ids.is_empty() {
        settings.active_tab = new_ids[0].clone();
    }
    
    save_settings_to_file(&app, &settings)?;
    drop(settings);
    let _ = rebuild_menu(&app);
    
    Ok(())
}

#[tauri::command]
fn switch_tab(app: AppHandle, tab_id: String) -> Result<(), String> {
    log::info!("Switching to tab: {}", tab_id);
    
    let state = app.state::<SettingsState>();
    let mut settings = state.0.lock().unwrap();
    
    for website in &settings.websites {
        if let Some(window) = app.get_webview_window(&website.id) {
            if website.id == tab_id {
                window.show().map_err(|e| format!("show: {}", e))?;
                window.set_focus().map_err(|e| format!("focus: {}", e))?;
            } else {
                window.hide().map_err(|e| format!("hide: {}", e))?;
            }
        }
    }
    
    settings.active_tab = tab_id;
    save_settings_to_file(&app, &settings)?;
    
    Ok(())
}

#[tauri::command]
fn open_settings(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    } else {
        WebviewWindowBuilder::new(
            &app,
            "settings",
            WebviewUrl::App("index.html".into())
        )
        .title("Peko Settings")
        .inner_size(500.0, 400.0)
        .resizable(false)
        .decorations(true)
        .center()
        .build()
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn cycle_tab(app: AppHandle) -> Result<(), String> {
    let state = app.state::<SettingsState>();
    let settings = state.0.lock().unwrap();
    
    if settings.websites.is_empty() {
        return Ok(());
    }
    
    let current_idx = settings.websites.iter()
        .position(|w| w.id == settings.active_tab)
        .unwrap_or(0);
    
    let next_idx = (current_idx + 1) % settings.websites.len();
    let next_id = settings.websites[next_idx].id.clone();
    
    drop(settings);
    switch_tab(app, next_id)
}

#[tauri::command]
fn toggle_auto_paste(app: AppHandle) -> Result<bool, String> {
    let state = app.state::<SettingsState>();
    let mut settings = state.0.lock().unwrap();
    
    settings.auto_paste_on_focus = !settings.auto_paste_on_focus;
    AUTO_PASTE_ENABLED.store(settings.auto_paste_on_focus, Ordering::SeqCst);
    
    log::info!("Auto-paste on focus: {}", settings.auto_paste_on_focus);
    save_settings_to_file(&app, &settings)?;
    
    // Rebuild menu to update checkbox state
    let enabled = settings.auto_paste_on_focus;
    drop(settings);
    let _ = rebuild_menu(&app);
    
    Ok(enabled)
}

#[tauri::command]
fn go_back(app: AppHandle) -> Result<(), String> {
    let state = app.state::<SettingsState>();
    let settings = state.0.lock().unwrap();
    let active = settings.active_tab.clone();
    drop(settings);
    
    if let Some(webview) = app.get_webview_window(&active) {
        webview.eval("history.back()").map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn go_forward(app: AppHandle) -> Result<(), String> {
    let state = app.state::<SettingsState>();
    let settings = state.0.lock().unwrap();
    let active = settings.active_tab.clone();
    drop(settings);
    
    if let Some(webview) = app.get_webview_window(&active) {
        webview.eval("history.forward()").map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn save_default_website(app: AppHandle, website_id: String) -> Result<(), String> {
    let state = app.state::<SettingsState>();
    let mut settings = state.0.lock().unwrap();
    settings.default_website = Some(website_id);
    save_settings_to_file(&app, &settings)?;
    Ok(())
}

#[tauri::command]
fn toggle_notes(app: AppHandle) -> Result<String, String> {
    let state = app.state::<SettingsState>();
    let mut settings = state.0.lock().unwrap();
    
    // Cycle: hidden -> sidebar -> window -> hidden
    let new_mode = match settings.notes_mode.as_str() {
        "hidden" => "sidebar",
        "sidebar" => "window",
        "window" => "hidden",
        _ => "hidden",
    };
    
    settings.notes_mode = new_mode.to_string();
    let active_tab = settings.active_tab.clone();
    save_settings_to_file(&app, &settings)?;
    drop(settings);
    
    let sidebar_width: u32 = 350;
    
    match new_mode {
        "hidden" => {
            // Hide notes window
            if let Some(notes_window) = app.get_webview_window("notes") {
                notes_window.hide().map_err(|e| e.to_string())?;
            }
            // Restore main window size
            if let Some(main_window) = app.get_webview_window(&active_tab) {
                let _ = main_window.set_focus();
            }
        }
        "sidebar" => {
            // Position notes window attached to right of main window
            if let Some(main_window) = app.get_webview_window(&active_tab) {
                let pos = main_window.outer_position().unwrap_or_default();
                let size = main_window.outer_size().unwrap_or_default();
                
                let notes_x = pos.x + size.width as i32;
                let notes_y = pos.y;
                let notes_height = size.height;
                
                if let Some(notes_window) = app.get_webview_window("notes") {
                    notes_window.set_position(tauri::PhysicalPosition::new(notes_x, notes_y)).ok();
                    notes_window.set_size(tauri::PhysicalSize::new(sidebar_width, notes_height)).ok();
                    notes_window.set_decorations(false).ok();  // No title bar in sidebar mode
                    notes_window.show().map_err(|e| e.to_string())?;
                } else {
                    WebviewWindowBuilder::new(
                        &app,
                        "notes",
                        WebviewUrl::App("notes.html".into())
                    )
                    .title("Notes")
                    .position(notes_x as f64, notes_y as f64)
                    .inner_size(sidebar_width as f64, notes_height as f64)
                    .min_inner_size(200.0, 300.0)
                    .resizable(true)
                    .decorations(false)  // No title bar - looks like sidebar
                    .build()
                    .map_err(|e| e.to_string())?;
                }
            }
        }
        "window" => {
            // Standalone floating window with decorations
            if let Some(notes_window) = app.get_webview_window("notes") {
                notes_window.set_decorations(true).ok();  // Restore title bar
                notes_window.center().ok();
                notes_window.set_size(tauri::PhysicalSize::new(400, 600)).ok();
                notes_window.show().map_err(|e| e.to_string())?;
                notes_window.set_focus().map_err(|e| e.to_string())?;
            } else {
                WebviewWindowBuilder::new(
                    &app,
                    "notes",
                    WebviewUrl::App("notes.html".into())
                )
                .title("Notes")
                .inner_size(400.0, 600.0)
                .min_inner_size(300.0, 400.0)
                .resizable(true)
                .decorations(true)
                .center()
                .build()
                .map_err(|e| e.to_string())?;
            }
        }
        _ => {}
    }
    
    log::info!("Notes mode: {}", new_mode);
    Ok(new_mode.to_string())
}

#[tauri::command]
fn save_notes(app: AppHandle, content: String) -> Result<(), String> {
    let state = app.state::<SettingsState>();
    let mut settings = state.0.lock().unwrap();
    settings.notes_content = content;
    save_settings_to_file(&app, &settings)?;
    Ok(())
}

#[tauri::command]
fn get_notes(app: AppHandle) -> Result<String, String> {
    let state = app.state::<SettingsState>();
    let settings = state.0.lock().unwrap();
    Ok(settings.notes_content.clone())
}

#[tauri::command]
fn show_tab_switcher(app: AppHandle) -> Result<(), String> {
    // Get active tab to position overlay
    let state = app.state::<SettingsState>();
    let settings = state.0.lock().unwrap();
    let active_tab = settings.active_tab.clone();
    drop(settings);
    
    // Get main window position for overlay placement
    let (pos_x, pos_y) = if let Some(main_window) = app.get_webview_window(&active_tab) {
        let pos = main_window.outer_position().unwrap_or_default();
        let size = main_window.outer_size().unwrap_or_default();
        // Center horizontally, position at top
        (pos.x + (size.width as i32 / 2) - 250, pos.y + 20)
    } else {
        (400, 100)
    };
    
    if let Some(overlay) = app.get_webview_window("tab_switcher") {
        overlay.set_position(tauri::PhysicalPosition::new(pos_x, pos_y)).ok();
        overlay.show().map_err(|e| e.to_string())?;
        overlay.set_focus().map_err(|e| e.to_string())?;
    } else {
        WebviewWindowBuilder::new(
            &app,
            "tab_switcher",
            WebviewUrl::App("tab-switcher.html".into())
        )
        .title("Tab Switcher")
        .inner_size(500.0, 120.0)
        .position(pos_x as f64, pos_y as f64)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .build()
        .map_err(|e: tauri::Error| e.to_string())?;
    }
    
    log::info!("Tab switcher shown");
    Ok(())
}

#[tauri::command]
fn hide_tab_switcher(app: AppHandle) -> Result<(), String> {
    if let Some(overlay) = app.get_webview_window("tab_switcher") {
        overlay.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn rebuild_menu(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<SettingsState>();
    let settings = state.0.lock().unwrap();
    
    // Build Tabs submenu
    let mut tabs_items: Vec<MenuItem<tauri::Wry>> = Vec::new();
    
    for (i, website) in settings.websites.iter().enumerate() {
        let shortcut = if i < 9 {
            format!("CmdOrCtrl+{}", i + 1)
        } else {
            String::new()
        };
        
        let item = MenuItem::with_id(
            app,
            &website.id,
            format!("{} {}", website.emoji, website.name),
            true,
            if shortcut.is_empty() { None } else { Some(shortcut.as_str()) }
        ).map_err(|e| e.to_string())?;
        
        tabs_items.push(item);
    }
    
    let tabs_submenu = Submenu::with_items(
        app,
        "Tabs",
        true,
        &tabs_items.iter().map(|i| i as &dyn tauri::menu::IsMenuItem<tauri::Wry>).collect::<Vec<_>>()
    ).map_err(|e| e.to_string())?;
    
    // Settings menu item
    let settings_item = MenuItem::with_id(
        app,
        "settings",
        "Settings...",
        true,
        Some("CmdOrCtrl+Comma")
    ).map_err(|e| e.to_string())?;
    
    // Auto-paste toggle
    let auto_paste_item = CheckMenuItem::with_id(
        app,
        "auto_paste",
        "Auto-Paste on Focus",
        true,
        settings.auto_paste_on_focus,
        Some("CmdOrCtrl+Shift+V")
    ).map_err(|e| e.to_string())?;
    
    // Cycle tabs
    let cycle_item = MenuItem::with_id(
        app,
        "cycle_tab",
        "Next Tab",
        true,
        Some("CmdOrCtrl+Tab")
    ).map_err(|e| e.to_string())?;
    
    let separator = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let separator2 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let quit = PredefinedMenuItem::quit(app, Some("Quit Peko")).map_err(|e| e.to_string())?;
    
    // App menu
    let app_menu = Submenu::with_items(
        app,
        "Peko",
        true,
        &[
            &settings_item as &dyn tauri::menu::IsMenuItem<tauri::Wry>,
            &separator,
            &quit,
        ]
    ).map_err(|e| e.to_string())?;
    
    // Navigation items
    let back_item = MenuItem::with_id(
        app,
        "go_back",
        "Back",
        true,
        Some("CmdOrCtrl+[")
    ).map_err(|e| e.to_string())?;
    
    let forward_item = MenuItem::with_id(
        app,
        "go_forward",
        "Forward",
        true,
        Some("CmdOrCtrl+]")
    ).map_err(|e| e.to_string())?;
    
    let separator3 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    
    // Notes toggle
    let notes_item = MenuItem::with_id(
        app,
        "toggle_notes",
        "Toggle Notes",
        true,
        Some("CmdOrCtrl+N")
    ).map_err(|e| e.to_string())?;
    
    let separator4 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    
    // Tab switcher
    let tab_switcher_item = MenuItem::with_id(
        app,
        "show_tab_switcher",
        "Show Tab Switcher",
        true,
        Some("CmdOrCtrl+\\")
    ).map_err(|e| e.to_string())?;
    
    // Edit menu with standard copy/paste actions
    let undo = PredefinedMenuItem::undo(app, Some("Undo")).map_err(|e| e.to_string())?;
    let redo = PredefinedMenuItem::redo(app, Some("Redo")).map_err(|e| e.to_string())?;
    let cut = PredefinedMenuItem::cut(app, Some("Cut")).map_err(|e| e.to_string())?;
    let copy = PredefinedMenuItem::copy(app, Some("Copy")).map_err(|e| e.to_string())?;
    let paste = PredefinedMenuItem::paste(app, Some("Paste")).map_err(|e| e.to_string())?;
    let select_all = PredefinedMenuItem::select_all(app, Some("Select All")).map_err(|e| e.to_string())?;
    let edit_separator = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    
    let edit_menu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &undo as &dyn tauri::menu::IsMenuItem<tauri::Wry>,
            &redo,
            &edit_separator,
            &cut,
            &copy,
            &paste,
            &select_all,
        ]
    ).map_err(|e| e.to_string())?;
    
    let view_menu = Submenu::with_items(
        app,
        "View",
        true,
        &[
            &back_item as &dyn tauri::menu::IsMenuItem<tauri::Wry>,
            &forward_item,
            &separator3,
            &notes_item,
            &tab_switcher_item,
            &separator4,
            &cycle_item,
            &separator2,
            &auto_paste_item,
        ]
    ).map_err(|e| e.to_string())?;
    
    let menu = Menu::with_items(
        app,
        &[
            &app_menu as &dyn tauri::menu::IsMenuItem<tauri::Wry>,
            &edit_menu,
            &tabs_submenu,
            &view_menu,
        ]
    ).map_err(|e| e.to_string())?;
    
    app.set_menu(menu).map_err(|e| e.to_string())?;
    
    Ok(())
}

fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    let id = event.id().as_ref();
    
    match id {
        "settings" => {
            let _ = open_settings(app.clone());
        }
        "cycle_tab" => {
            let _ = cycle_tab(app.clone());
        }
        "auto_paste" => {
            let _ = toggle_auto_paste(app.clone());
        }
        "go_back" => {
            let _ = go_back(app.clone());
        }
        "go_forward" => {
            let _ = go_forward(app.clone());
        }
        "toggle_notes" => {
            let _ = toggle_notes(app.clone());
        }
        "show_tab_switcher" => {
            let _ = show_tab_switcher(app.clone());
        }
        _ => {
            let state = app.state::<SettingsState>();
            let settings = state.0.lock().unwrap();
            if settings.websites.iter().any(|w| w.id == id) {
                drop(settings);
                let _ = switch_tab(app.clone(), id.to_string());
            }
        }
    }
}

fn create_website_windows(app: &tauri::App, settings: &AppSettings) {
    let app_data_dir = app.path().app_data_dir()
        .expect("Failed to get app data directory");
    
    for website in &settings.websites {
        let data_dir = app_data_dir.join(format!("webview_{}", website.id));
        fs::create_dir_all(&data_dir).expect("Failed to create webview data directory");
        
        // Use default_website for initial visibility, fallback to active_tab
        let default_id = settings.default_website.as_ref().unwrap_or(&settings.active_tab);
        let visible = website.id == *default_id;
        
        let _ = WebviewWindowBuilder::new(
            app,
            &website.id,
            WebviewUrl::External(website.url.parse().unwrap())
        )
        .title(format!("Peko - {}", website.name))
        .inner_size(1200.0, 800.0)
        .min_inner_size(600.0, 400.0)
        .resizable(true)
        .decorations(true)
        .visible(visible)
        .data_directory(data_dir)
        .build();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();
    
    log::info!("Peko application starting");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_websites,
            switch_tab,
            open_settings,
            cycle_tab,
            toggle_auto_paste,
            go_back,
            go_forward,
            save_default_website,
            toggle_notes,
            save_notes,
            get_notes,
            show_tab_switcher,
            hide_tab_switcher
        ])
        .setup(|app| {
            // Load settings
            let settings = load_settings(app.handle());
            log::info!("Loaded {} websites, active: {}", settings.websites.len(), settings.active_tab);
            
            // Set auto-paste state
            AUTO_PASTE_ENABLED.store(settings.auto_paste_on_focus, Ordering::SeqCst);
            
            // Create website windows
            create_website_windows(app, &settings);
            

            
            // Store settings state
            app.manage(SettingsState(Mutex::new(settings)));
            
            // Build menu
            rebuild_menu(app.handle())?;
            
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::Focused(focused) = event {
                if *focused && AUTO_PASTE_ENABLED.load(Ordering::SeqCst) {
                    let window_label = window.label().to_string();
                    // Only auto-paste for website windows, not settings
                    if window_label != "settings" {
                        log::info!("Window focused with auto-paste: {}", window_label);
                        
                        let app = window.app_handle().clone();
                        let label = window_label.clone();
                        tauri::async_runtime::spawn(async move {
                            // Small delay to ensure focus is complete
                            tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
                            // Execute paste command via JavaScript
                            if let Some(webview) = app.get_webview_window(&label) {
                                let _ = webview.eval("if(document.activeElement){document.execCommand('paste')}");
                            }
                        });
                    }
                }
            }
        })
        .on_menu_event(handle_menu_event)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Website Struct Tests =====

    #[test]
    fn test_website_creation() {
        let website = Website {
            id: "test_id".to_string(),
            name: "Test Site".to_string(),
            url: "https://example.com".to_string(),
            emoji: "🌐".to_string(),
        };

        assert_eq!(website.id, "test_id");
        assert_eq!(website.name, "Test Site");
        assert_eq!(website.url, "https://example.com");
        assert_eq!(website.emoji, "🌐");
    }

    #[test]
    fn test_website_clone() {
        let original = Website {
            id: "clone_test".to_string(),
            name: "Clone Test".to_string(),
            url: "https://clone.example.com".to_string(),
            emoji: "📋".to_string(),
        };

        let cloned = original.clone();

        assert_eq!(original.id, cloned.id);
        assert_eq!(original.name, cloned.name);
        assert_eq!(original.url, cloned.url);
        assert_eq!(original.emoji, cloned.emoji);
    }

    // ===== AppSettings Tests =====

    #[test]
    fn test_app_settings_default() {
        let settings = AppSettings::default();

        // Should have 2 default websites
        assert_eq!(settings.websites.len(), 2);
        
        // First should be Gemini
        assert_eq!(settings.websites[0].id, "gemini");
        assert_eq!(settings.websites[0].name, "Gemini");
        assert_eq!(settings.websites[0].url, "https://gemini.google.com/app");
        
        // Second should be NotebookLM
        assert_eq!(settings.websites[1].id, "notebooklm");
        assert_eq!(settings.websites[1].name, "NotebookLM");
        
        // Check other default values
        assert_eq!(settings.active_tab, "gemini");
        assert_eq!(settings.default_website, Some("gemini".to_string()));
        assert!(!settings.auto_paste_on_focus);
        assert!(settings.notes_content.is_empty());
        assert_eq!(settings.notes_mode, "hidden");
    }

    #[test]
    fn test_default_notes_mode() {
        assert_eq!(default_notes_mode(), "hidden");
    }

    // ===== Serialization Tests =====

    #[test]
    fn test_website_serialization_roundtrip() {
        let website = Website {
            id: "serial_test".to_string(),
            name: "Serialization Test".to_string(),
            url: "https://serial.example.com".to_string(),
            emoji: "🔄".to_string(),
        };

        let json = serde_json::to_string(&website).expect("Failed to serialize");
        let deserialized: Website = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(website.id, deserialized.id);
        assert_eq!(website.name, deserialized.name);
        assert_eq!(website.url, deserialized.url);
        assert_eq!(website.emoji, deserialized.emoji);
    }

    #[test]
    fn test_app_settings_serialization_roundtrip() {
        let settings = AppSettings {
            websites: vec![
                Website {
                    id: "test1".to_string(),
                    name: "Test One".to_string(),
                    url: "https://one.example.com".to_string(),
                    emoji: "1️⃣".to_string(),
                },
            ],
            active_tab: "test1".to_string(),
            default_website: Some("test1".to_string()),
            auto_paste_on_focus: true,
            notes_content: "Test notes content".to_string(),
            notes_mode: "sidebar".to_string(),
        };

        let json = serde_json::to_string_pretty(&settings).expect("Failed to serialize");
        let deserialized: AppSettings = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(settings.websites.len(), deserialized.websites.len());
        assert_eq!(settings.active_tab, deserialized.active_tab);
        assert_eq!(settings.default_website, deserialized.default_website);
        assert_eq!(settings.auto_paste_on_focus, deserialized.auto_paste_on_focus);
        assert_eq!(settings.notes_content, deserialized.notes_content);
        assert_eq!(settings.notes_mode, deserialized.notes_mode);
    }

    #[test]
    fn test_settings_deserialize_with_missing_optional_fields() {
        // Simulates loading old settings file without new fields
        let json = r#"{
            "websites": [],
            "active_tab": "gemini"
        }"#;

        let settings: AppSettings = serde_json::from_str(json).expect("Failed to deserialize");

        // Optional/default fields should have default values
        assert!(settings.default_website.is_none());
        assert!(!settings.auto_paste_on_focus);
        assert!(settings.notes_content.is_empty());
        assert_eq!(settings.notes_mode, "hidden");
    }

    // ===== Validation Logic Tests =====

    #[test]
    fn test_notes_mode_cycle() {
        // Test the notes mode cycle logic
        let modes = ["hidden", "sidebar", "window", "hidden"];
        
        for i in 0..modes.len() - 1 {
            let current = modes[i];
            let expected_next = modes[i + 1];
            
            let next = match current {
                "hidden" => "sidebar",
                "sidebar" => "window",
                "window" => "hidden",
                _ => "hidden",
            };
            
            assert_eq!(next, expected_next, "Mode cycle failed from {}", current);
        }
    }

    #[test]
    fn test_website_limit_constant() {
        // The app enforces a max of 5 websites
        // This test documents that constraint
        let max_websites = 5;
        
        let mut websites = Vec::new();
        for i in 0..max_websites {
            websites.push(Website {
                id: format!("site_{}", i),
                name: format!("Site {}", i),
                url: format!("https://site{}.example.com", i),
                emoji: "🌐".to_string(),
            });
        }
        
        assert_eq!(websites.len(), 5);
        // Adding more than 5 should be rejected by save_websites command
    }
}
