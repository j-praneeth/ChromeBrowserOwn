// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod browser;
mod privacy;
mod storage;

use tauri::{Manager, State};
use std::sync::Mutex;
use browser::{BrowserState, Tab};
use storage::StorageManager;
use privacy::PrivacyEngine;

#[derive(Default)]
struct AppState {
    browser: Mutex<BrowserState>,
    storage: Mutex<StorageManager>,
    privacy: Mutex<PrivacyEngine>,
}

#[tauri::command]
async fn create_tab(state: State<'_, AppState>) -> Result<String, String> {
    let mut browser = state.browser.lock().unwrap();
    let tab_id = browser.create_tab();
    Ok(tab_id)
}

#[tauri::command]
async fn close_tab(tab_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut browser = state.browser.lock().unwrap();
    browser.close_tab(&tab_id);
    Ok(())
}

#[tauri::command]
async fn navigate_tab(tab_id: String, url: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut browser = state.browser.lock().unwrap();
    let privacy = state.privacy.lock().unwrap();
    
    // Check if URL should be blocked
    if privacy.should_block_url(&url) {
        return Err("URL blocked by privacy filter".to_string());
    }
    
    browser.navigate_tab(&tab_id, &url);
    
    // Save to history
    let mut storage = state.storage.lock().unwrap();
    storage.add_history_entry(&url, &format!("Page at {}", url)).await.map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
async fn get_tabs(state: State<'_, AppState>) -> Result<Vec<Tab>, String> {
    let browser = state.browser.lock().unwrap();
    Ok(browser.get_tabs())
}

#[tauri::command]
async fn add_bookmark(url: String, title: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut storage = state.storage.lock().unwrap();
    storage.add_bookmark(&url, &title).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_bookmarks(state: State<'_, AppState>) -> Result<Vec<storage::Bookmark>, String> {
    let mut storage = state.storage.lock().unwrap();
    storage.get_bookmarks().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_history(state: State<'_, AppState>) -> Result<Vec<storage::HistoryEntry>, String> {
    let mut storage = state.storage.lock().unwrap();
    storage.get_history().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    let mut storage = state.storage.lock().unwrap();
    storage.clear_history().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_privacy_lists(state: State<'_, AppState>) -> Result<(), String> {
    let mut privacy = state.privacy.lock().unwrap();
    privacy.update_block_lists().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_url_safety(url: String, state: State<'_, AppState>) -> Result<bool, String> {
    let privacy = state.privacy.lock().unwrap();
    Ok(!privacy.should_block_url(&url))
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .setup(|app| {
            let app_handle = app.handle();
            let state: State<AppState> = app_handle.state();
            
            // Initialize storage
            tokio::spawn(async move {
                let mut storage = state.storage.lock().unwrap();
                if let Err(e) = storage.initialize().await {
                    eprintln!("Failed to initialize storage: {}", e);
                }
            });
            
            // Initialize privacy engine
            tokio::spawn(async move {
                let mut privacy = state.privacy.lock().unwrap();
                if let Err(e) = privacy.initialize().await {
                    eprintln!("Failed to initialize privacy engine: {}", e);
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_tab,
            close_tab,
            navigate_tab,
            get_tabs,
            add_bookmark,
            get_bookmarks,
            get_history,
            clear_history,
            update_privacy_lists,
            check_url_safety
        ])
        .run(tauri::generate_context!())
        .expect("Error while running Privacy Browser");
}
