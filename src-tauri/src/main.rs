// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Tab {
    id: String,
    title: String,
    url: String,
    loading: bool,
}

#[derive(Default)]
struct AppState {
    tabs: std::sync::Mutex<HashMap<String, Tab>>,
}

#[tauri::command]
fn create_tab(state: tauri::State<AppState>) -> String {
    let tab_id = format!("tab-{}", chrono::Utc::now().timestamp_millis());
    let tab = Tab {
        id: tab_id.clone(),
        title: "New Tab".to_string(),
        url: "about:blank".to_string(),
        loading: false,
    };
    
    let mut tabs = state.tabs.lock().unwrap();
    tabs.insert(tab_id.clone(), tab);
    tab_id
}

#[tauri::command]
fn close_tab(tab_id: String, state: tauri::State<AppState>) {
    let mut tabs = state.tabs.lock().unwrap();
    tabs.remove(&tab_id);
}

#[tauri::command]
async fn navigate_tab(tab_id: String, url: String, state: tauri::State<'_, AppState>, app_handle: tauri::AppHandle) -> Result<(), String> {
    // Enhanced privacy check with more comprehensive blocking
    let blocked_domains = [
        "google-analytics.com",
        "googletagmanager.com", 
        "facebook.com",
        "doubleclick.net",
        "googlesyndication.com",
        "googleadservices.com",
        "googlesyndication.com",
        "amazon-adsystem.com",
        "adsystem.amazon.com"
    ];
    
    for domain in &blocked_domains {
        if url.contains(domain) {
            return Err("URL blocked by privacy filter".to_string());
        }
    }
    
    let mut tabs = state.tabs.lock().unwrap();
    if let Some(tab) = tabs.get_mut(&tab_id) {
        tab.url = url.clone();
        tab.loading = true;
        
        // Create a new webview window for this tab - Chrome-like behavior
        let webview_label = format!("webview_{}", tab_id);
        
        // Try to get existing webview or create new one
        if let Some(webview) = app_handle.get_webview_window(&webview_label) {
            // Navigate existing webview
            let _ = webview.navigate(url.parse().map_err(|e| format!("Invalid URL: {}", e))?);
        } else {
            // Create new webview window
            let webview = WebviewWindowBuilder::new(&app_handle, &webview_label, WebviewUrl::External(url.parse().map_err(|e| format!("Invalid URL: {}", e))?))
                .title(&tab.title)
                .inner_size(1200.0, 800.0)
                .build()
                .map_err(|e| format!("Failed to create webview: {}", e))?;
            
            // Set up privacy controls
            webview.set_user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Privacy Browser/1.0.0")
                .map_err(|e| format!("Failed to set user agent: {}", e))?;
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
fn check_url_safety(url: String) -> bool {
    let blocked_domains = [
        "google-analytics.com",
        "googletagmanager.com", 
        "facebook.com",
        "doubleclick.net",
        "googlesyndication.com"
    ];
    
    for domain in &blocked_domains {
        if url.contains(domain) {
            return false;
        }
    }
    true
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            create_tab,
            close_tab,
            navigate_tab,
            get_tabs,
            check_url_safety
        ])
        .run(tauri::generate_context!())
        .expect("Error while running Privacy Browser");
}
