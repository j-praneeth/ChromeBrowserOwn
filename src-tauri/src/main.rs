// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    Manager, Window, WindowBuilder, WindowUrl, 
    generate_context, generate_handler,
    AppHandle
};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Tab {
    id: String,
    title: String,
    url: String,
    loading: bool,
    window_label: String,
    can_go_back: bool,
    can_go_forward: bool,
    last_accessed: DateTime<Utc>,
}

#[derive(Default)]
struct AppState {
    tabs: Mutex<HashMap<String, Tab>>,
    active_tab_id: Mutex<Option<String>>,
    blocked_count: Mutex<u32>,
}

// Privacy lists - comprehensive tracking and ad domains
const BLOCKED_DOMAINS: &[&str] = &[
    "google-analytics.com",
    "googletagmanager.com",
    "doubleclick.net",
    "googlesyndication.com",
    "googleadservices.com",
    "amazon-adsystem.com",
    "adsystem.amazon.com",
    "facebook.com/tr",
    "connect.facebook.net",
    "analytics.twitter.com",
    "static.ads-twitter.com",
    "linkedin.com/analytics",
    "bizographics.com",
    "outbrain.com",
    "taboola.com",
    "criteo.com",
    "bing.com/analytics",
    "hotjar.com",
    "fullstory.com",
    "mouseflow.com",
    "crazyegg.com"
];

#[tauri::command]
async fn create_tab(app_handle: AppHandle, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let tab_id = format!("tab-{}", Utc::now().timestamp_millis());
    let window_label = format!("webview-{}", tab_id);
    
    // Create a new window for this tab using Tauri 1.x API
    let new_window = WindowBuilder::new(
        &app_handle,
        &window_label,
        WindowUrl::External("about:blank".parse().unwrap())
    )
    .title("New Tab")
    .visible(false)
    .resizable(true)
    .inner_size(800.0, 600.0)
    .build()
    .map_err(|e| format!("Failed to create window: {}", e))?;

    let tab = Tab {
        id: tab_id.clone(),
        title: "New Tab".to_string(),
        url: "about:blank".to_string(),
        loading: false,
        window_label: window_label,
        can_go_back: false,
        can_go_forward: false,
        last_accessed: Utc::now(),
    };
    
    let mut tabs = state.tabs.lock().unwrap();
    tabs.insert(tab_id.clone(), tab);
    
    // Set as active tab if it's the first one
    let mut active_tab = state.active_tab_id.lock().unwrap();
    if active_tab.is_none() {
        *active_tab = Some(tab_id.clone());
        new_window.show().map_err(|e| format!("Failed to show window: {}", e))?;
    }
    
    Ok(tab_id)
}

#[tauri::command]
async fn close_tab(tab_id: String, app_handle: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut tabs = state.tabs.lock().unwrap();
    
    if let Some(tab) = tabs.remove(&tab_id) {
        // Close the window
        if let Some(window) = app_handle.get_window(&tab.window_label) {
            window.close().map_err(|e| format!("Failed to close window: {}", e))?;
        }
        
        // Update active tab if needed
        let mut active_tab = state.active_tab_id.lock().unwrap();
        if active_tab.as_ref() == Some(&tab_id) {
            *active_tab = tabs.keys().next().cloned();
            
            // Show the new active tab if exists
            if let Some(new_active_id) = active_tab.as_ref() {
                if let Some(new_active_tab) = tabs.get(new_active_id) {
                    if let Some(window) = app_handle.get_window(&new_active_tab.window_label) {
                        window.show().map_err(|e| format!("Failed to show new active window: {}", e))?;
                    }
                }
            }
        }
    }
    
    Ok(())
}

#[tauri::command]
async fn switch_tab(tab_id: String, app_handle: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let tabs = state.tabs.lock().unwrap();
    let mut active_tab = state.active_tab_id.lock().unwrap();
    
    // Hide current active tab
    if let Some(current_active_id) = active_tab.as_ref() {
        if let Some(current_tab) = tabs.get(current_active_id) {
            if let Some(window) = app_handle.get_window(&current_tab.window_label) {
                window.hide().map_err(|e| format!("Failed to hide current window: {}", e))?;
            }
        }
    }
    
    // Show new active tab
    if let Some(new_tab) = tabs.get(&tab_id) {
        if let Some(window) = app_handle.get_window(&new_tab.window_label) {
            window.show().map_err(|e| format!("Failed to show window: {}", e))?;
            window.set_focus().map_err(|e| format!("Failed to focus window: {}", e))?;
        }
        *active_tab = Some(tab_id);
    }
    
    Ok(())
}

#[tauri::command]
async fn navigate_tab(tab_id: String, url: String, app_handle: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    // Check privacy filter first
    for domain in BLOCKED_DOMAINS {
        if url.contains(domain) {
            let mut blocked_count = state.blocked_count.lock().unwrap();
            *blocked_count += 1;
            return Err(format!("URL blocked by privacy filter: {}", domain));
        }
    }
    
    let mut tabs = state.tabs.lock().unwrap();
    if let Some(tab) = tabs.get_mut(&tab_id) {
        tab.url = url.clone();
        tab.loading = true;
        tab.last_accessed = Utc::now();
        
        // Navigate the window
        if let Some(window) = app_handle.get_window(&tab.window_label) {
            let parsed_url = if url.starts_with("http://") || url.starts_with("https://") {
                url
            } else if url == "about:blank" {
                url
            } else {
                format!("https://{}", url)
            };
            
            // Use JavaScript to navigate the window
            let nav_script = if parsed_url == "about:blank" {
                "window.location.href = 'about:blank';".to_string()
            } else {
                // Try to parse as URL first
                match parsed_url.parse::<url::Url>() {
                    Ok(_) => {
                        format!("window.location.href = '{}';", parsed_url.replace("'", "\\'"))
                    }
                    Err(_) => {
                        // If not a valid URL, treat as search query
                        let search_url = format!("https://duckduckgo.com/?q={}", urlencoding::encode(&url));
                        format!("window.location.href = '{}';", search_url.replace("'", "\\'"))
                    }
                }
            };
            
            window.eval(&nav_script).map_err(|e| format!("Failed to navigate: {}", e))?;
            
            // Update title from URL if it's a proper domain
            if let Ok(parsed) = parsed_url.parse::<url::Url>() {
                if let Some(host) = parsed.host_str() {
                    tab.title = host.to_string();
                }
            }
        } else {
            return Err("Webview window not found".to_string());
        }
        
        Ok(())
    } else {
        Err("Tab not found".to_string())
    }
}

#[tauri::command]
fn get_tabs(state: tauri::State<AppState>) -> Vec<Tab> {
    let tabs = state.tabs.lock().unwrap();
    tabs.values().cloned().collect()
}

#[tauri::command]
fn get_active_tab_id(state: tauri::State<AppState>) -> Option<String> {
    let active_tab = state.active_tab_id.lock().unwrap();
    active_tab.clone()
}

#[tauri::command]
fn check_url_safety(url: String) -> bool {
    for domain in BLOCKED_DOMAINS {
        if url.contains(domain) {
            return false;
        }
    }
    true
}

#[tauri::command]
fn get_blocked_count(state: tauri::State<AppState>) -> u32 {
    let blocked_count = state.blocked_count.lock().unwrap();
    *blocked_count
}

#[tauri::command]
async fn webview_go_back(tab_id: String, app_handle: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let tabs = state.tabs.lock().unwrap();
    if let Some(tab) = tabs.get(&tab_id) {
        if let Some(window) = app_handle.get_window(&tab.window_label) {
            window.eval("history.back()").map_err(|e| format!("Failed to go back: {}", e))?;
        }
    }
    Ok(())
}

#[tauri::command]
async fn webview_go_forward(tab_id: String, app_handle: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let tabs = state.tabs.lock().unwrap();
    if let Some(tab) = tabs.get(&tab_id) {
        if let Some(window) = app_handle.get_window(&tab.window_label) {
            window.eval("history.forward()").map_err(|e| format!("Failed to go forward: {}", e))?;
        }
    }
    Ok(())
}

#[tauri::command]
async fn webview_reload(tab_id: String, app_handle: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let tabs = state.tabs.lock().unwrap();
    if let Some(tab) = tabs.get(&tab_id) {
        if let Some(window) = app_handle.get_window(&tab.window_label) {
            window.eval("location.reload()").map_err(|e| format!("Failed to reload: {}", e))?;
        }
    }
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(generate_handler![
            create_tab,
            close_tab,
            switch_tab,
            navigate_tab,
            get_tabs,
            get_active_tab_id,
            check_url_safety,
            get_blocked_count,
            webview_go_back,
            webview_go_forward,
            webview_reload
        ])
        .run(generate_context!())
        .expect("Error while running Privacy Browser");
}
