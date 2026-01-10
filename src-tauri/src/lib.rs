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
        Some("CmdOrCtrl+,")
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
    
    // View menu
    let view_menu = Submenu::with_items(
        app,
        "View",
        true,
        &[
            &back_item as &dyn tauri::menu::IsMenuItem<tauri::Wry>,
            &forward_item,
            &separator3,
            &cycle_item,
            &separator2,
            &auto_paste_item,
        ]
    ).map_err(|e| e.to_string())?;
    
    let menu = Menu::with_items(
        app,
        &[
            &app_menu as &dyn tauri::menu::IsMenuItem<tauri::Wry>,
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
            save_default_website
        ])
        .setup(|app| {
            // Load settings
            let settings = load_settings(&app.handle());
            log::info!("Loaded {} websites, active: {}", settings.websites.len(), settings.active_tab);
            
            // Set auto-paste state
            AUTO_PASTE_ENABLED.store(settings.auto_paste_on_focus, Ordering::SeqCst);
            
            // Create website windows
            create_website_windows(app, &settings);
            

            
            // Store settings state
            app.manage(SettingsState(Mutex::new(settings)));
            
            // Build menu
            rebuild_menu(&app.handle())?;
            
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
