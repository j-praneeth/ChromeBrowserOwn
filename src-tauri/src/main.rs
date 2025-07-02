// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod browser;
mod privacy;
mod storage;

use tauri::{Manager, State};
use std::sync::{Arc, Mutex};
use browser::{BrowserState, Tab};
use storage::StorageManager;
use privacy::PrivacyEngine;

struct AppState {
    browser: Arc<Mutex<BrowserState>>,
    storage: Arc<Mutex<StorageManager>>,
    privacy: Arc<Mutex<PrivacyEngine>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            browser: Arc::new(Mutex::new(BrowserState::default())),
            storage: Arc::new(Mutex::new(StorageManager::default())),
            privacy: Arc::new(Mutex::new(PrivacyEngine::default())),
        }
    }
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
    // Check if URL should be blocked first
    let should_block = {
        let privacy = state.privacy.lock().unwrap();
        privacy.should_block_url(&url)
    };
    
    if should_block {
        return Err("URL blocked by privacy filter".to_string());
    }
    
    // Navigate the tab
    {
        let mut browser = state.browser.lock().unwrap();
        browser.navigate_tab(&tab_id, &url);
    }
    
    // Save to history
    let storage = state.storage.clone();
    {
        let mut storage_lock = storage.lock().unwrap();
        storage_lock.add_history_entry(&url, &format!("Page at {}", url)).await.map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
async fn get_tabs(state: State<'_, AppState>) -> Result<Vec<Tab>, String> {
    let browser = state.browser.lock().unwrap();
    Ok(browser.get_tabs())
}

#[tauri::command]
async fn add_bookmark(url: String, title: String, state: State<'_, AppState>) -> Result<(), String> {
    let storage = state.storage.clone();
    let mut storage_lock = storage.lock().unwrap();
    storage_lock.add_bookmark(&url, &title).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_bookmarks(state: State<'_, AppState>) -> Result<Vec<storage::Bookmark>, String> {
    let storage = state.storage.clone();
    let mut storage_lock = storage.lock().unwrap();
    storage_lock.get_bookmarks().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_history(state: State<'_, AppState>) -> Result<Vec<storage::HistoryEntry>, String> {
    let storage = state.storage.clone();
    let mut storage_lock = storage.lock().unwrap();
    storage_lock.get_history().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    let storage = state.storage.clone();
    let mut storage_lock = storage.lock().unwrap();
    storage_lock.clear_history().await.map_err(|e| e.to_string())
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
            let storage = state.storage.clone();
            tokio::spawn(async move {
                let mut storage_lock = storage.lock().unwrap();
                if let Err(e) = storage_lock.initialize().await {
                    eprintln!("Failed to initialize storage: {}", e);
                }
            });
            
            // Initialize privacy engine
            let privacy = state.privacy.clone();
            tokio::spawn(async move {
                let mut privacy_lock = privacy.lock().unwrap();
                if let Err(e) = privacy_lock.initialize().await {
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
