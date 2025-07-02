use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: String,
    pub url: String,
    pub title: String,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

impl Default for Tab {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            url: "about:blank".to_string(),
            title: "New Tab".to_string(),
            loading: false,
            can_go_back: false,
            can_go_forward: false,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
        }
    }
}

#[derive(Default)]
pub struct BrowserState {
    tabs: HashMap<String, Tab>,
    active_tab_id: Option<String>,
    tab_order: Vec<String>,
}

impl BrowserState {
    pub fn create_tab(&mut self) -> String {
        let tab = Tab::default();
        let tab_id = tab.id.clone();
        
        self.tabs.insert(tab_id.clone(), tab);
        self.tab_order.push(tab_id.clone());
        self.active_tab_id = Some(tab_id.clone());
        
        tab_id
    }
    
    pub fn close_tab(&mut self, tab_id: &str) {
        self.tabs.remove(tab_id);
        self.tab_order.retain(|id| id != tab_id);
        
        // If we closed the active tab, switch to another tab
        if let Some(active_id) = &self.active_tab_id {
            if active_id == tab_id {
                self.active_tab_id = self.tab_order.last().cloned();
            }
        }
        
        // If no tabs left, create a new one
        if self.tabs.is_empty() {
            self.create_tab();
        }
    }
    
    pub fn navigate_tab(&mut self, tab_id: &str, url: &str) {
        if let Some(tab) = self.tabs.get_mut(tab_id) {
            tab.url = url.to_string();
            tab.title = self.extract_title_from_url(url);
            tab.loading = true;
            tab.last_accessed = Utc::now();
            
            // Update navigation state (simplified)
            tab.can_go_back = tab.url != "about:blank";
            tab.can_go_forward = false;
        }
    }
    
    pub fn get_tabs(&self) -> Vec<Tab> {
        self.tab_order
            .iter()
            .filter_map(|id| self.tabs.get(id))
            .cloned()
            .collect()
    }
    
    pub fn get_active_tab(&self) -> Option<&Tab> {
        self.active_tab_id
            .as_ref()
            .and_then(|id| self.tabs.get(id))
    }
    
    pub fn set_active_tab(&mut self, tab_id: &str) {
        if self.tabs.contains_key(tab_id) {
            self.active_tab_id = Some(tab_id.to_string());
            
            // Update last accessed time
            if let Some(tab) = self.tabs.get_mut(tab_id) {
                tab.last_accessed = Utc::now();
            }
        }
    }
    
    fn extract_title_from_url(&self, url: &str) -> String {
        if url == "about:blank" {
            return "New Tab".to_string();
        }
        
        if let Ok(parsed_url) = url::Url::parse(url) {
            if let Some(host) = parsed_url.host_str() {
                return host.to_string();
            }
        }
        
        url.to_string()
    }
    
    pub fn update_tab_title(&mut self, tab_id: &str, title: &str) {
        if let Some(tab) = self.tabs.get_mut(tab_id) {
            tab.title = title.to_string();
            tab.loading = false;
        }
    }
    
    pub fn set_tab_loading(&mut self, tab_id: &str, loading: bool) {
        if let Some(tab) = self.tabs.get_mut(tab_id) {
            tab.loading = loading;
        }
    }
}
