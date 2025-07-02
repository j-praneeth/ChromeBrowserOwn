// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
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
fn navigate_tab(tab_id: String, url: String, state: tauri::State<AppState>, window: tauri::Window) -> Result<(), String> {
    // Enhanced privacy check with more comprehensive blocking
    let blocked_domains = [
        "google-analytics.com",
        "googletagmanager.com", 
        "facebook.com",
        "doubleclick.net",
        "googlesyndication.com",
        "googleadservices.com",
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
        
        // For Tauri 1.x, we'll navigate the main window
        // In a real Chrome-like implementation, each tab would have its own webview
        let _ = window.eval(&format!("window.location.href = '{}'", url));
        
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
