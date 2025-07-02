// Simplified main.rs to get the app running
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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
fn navigate_tab(tab_id: String, url: String, state: tauri::State<AppState>) -> Result<(), String> {
    let mut tabs = state.tabs.lock().unwrap();
    if let Some(tab) = tabs.get_mut(&tab_id) {
        tab.url = url;
        tab.loading = true;
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
    // Simple privacy check - block known tracker domains
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