use regex::Regex;
use std::collections::HashSet;
use tokio::time::{Duration, interval};
use reqwest;

pub struct PrivacyEngine {
    blocked_domains: HashSet<String>,
    blocked_patterns: Vec<Regex>,
    tracking_domains: HashSet<String>,
    ad_domains: HashSet<String>,
    initialized: bool,
}

impl Default for PrivacyEngine {
    fn default() -> Self {
        Self {
            blocked_domains: HashSet::new(),
            blocked_patterns: Vec::new(),
            tracking_domains: HashSet::new(),
            ad_domains: HashSet::new(),
            initialized: false,
        }
    }
}

impl PrivacyEngine {
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.initialized {
            return Ok(());
        }
        
        // Load default block lists
        self.load_default_filters();
        
        // Attempt to update from remote sources
        let _ = self.update_block_lists().await;
        
        self.initialized = true;
        Ok(())
    }
    
    fn load_default_filters(&mut self) {
        // Default tracking domains to block
        let tracking_domains = vec![
            "google-analytics.com",
            "googletagmanager.com",
            "facebook.com",
            "doubleclick.net",
            "googlesyndication.com",
            "amazon-adsystem.com",
            "twitter.com",
            "instagram.com",
            "linkedin.com",
            "pinterest.com",
            "snapchat.com",
            "tiktok.com",
            "outbrain.com",
            "taboola.com",
            "criteo.com",
            "adsystem.com",
            "adsymptotic.com",
            "adservicing.com",
            "scorecardresearch.com",
            "quantserve.com",
            "hotjar.com",
            "mouseflow.com",
            "crazyegg.com",
            "fullstory.com",
        ];
        
        for domain in tracking_domains {
            self.tracking_domains.insert(domain.to_string());
            self.blocked_domains.insert(domain.to_string());
        }
        
        // Default ad domains to block
        let ad_domains = vec![
            "googlesyndication.com",
            "googleadservices.com",
            "amazon-adsystem.com",
            "adsystem.com",
            "doubleclick.net",
            "media.net",
            "adsymptotic.com",
            "adservicing.com",
            "outbrain.com",
            "taboola.com",
            "revcontent.com",
            "criteo.com",
            "bidswitch.net",
            "adswifit.com",
            "casalemedia.com",
            "contextweb.com",
            "rubiconproject.com",
            "openx.net",
            "pubmatic.com",
        ];
        
        for domain in ad_domains {
            self.ad_domains.insert(domain.to_string());
            self.blocked_domains.insert(domain.to_string());
        }
        
        // Compile regex patterns for common tracking and ad patterns
        let patterns = vec![
            r".*\.ads\.",
            r".*\.advertising\.",
            r".*\.tracker\.",
            r".*\.analytics\.",
            r".*\.metrics\.",
            r".*\.telemetry\.",
            r".*\.beacon\.",
            r".*\.pixel\.",
            r".*googletagmanager.*",
            r".*google-analytics.*",
            r".*facebook\.com/tr.*",
            r".*\.doubleclick\.net.*",
            r".*googlesyndication.*",
        ];
        
        for pattern in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                self.blocked_patterns.push(regex);
            }
        }
    }
    
    pub async fn update_block_lists(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Try to fetch updated block lists from common sources
        let block_list_urls = vec![
            "https://easylist.to/easylist/easylist.txt",
            "https://easylist.to/easylist/easyprivacy.txt",
        ];
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;
        
        for url in block_list_urls {
            match client.get(url).send().await {
                Ok(response) => {
                    if let Ok(text) = response.text().await {
                        self.parse_easylist_format(&text);
                    }
                }
                Err(_) => {
                    // Silently continue if we can't fetch remote lists
                    continue;
                }
            }
        }
        
        Ok(())
    }
    
    fn parse_easylist_format(&mut self, content: &str) {
        for line in content.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('!') || line.starts_with('[') {
                continue;
            }
            
            // Extract domain from various EasyList formats
            if line.starts_with("||") {
                if let Some(domain) = self.extract_domain_from_filter(line) {
                    self.blocked_domains.insert(domain);
                }
            }
        }
    }
    
    fn extract_domain_from_filter(&self, filter: &str) -> Option<String> {
        // Simple extraction for ||domain.com^ format
        if filter.starts_with("||") && filter.contains("^") {
            let domain_part = &filter[2..];
            if let Some(end) = domain_part.find('^') {
                let domain = &domain_part[..end];
                if self.is_valid_domain(domain) {
                    return Some(domain.to_string());
                }
            }
        }
        None
    }
    
    fn is_valid_domain(&self, domain: &str) -> bool {
        // Basic domain validation
        !domain.is_empty() 
            && domain.contains('.') 
            && !domain.contains('/')
            && !domain.contains('*')
    }
    
    pub fn should_block_url(&self, url: &str) -> bool {
        if !self.initialized {
            return false;
        }
        
        // Parse URL to get domain
        if let Ok(parsed_url) = url::Url::parse(url) {
            if let Some(host) = parsed_url.host_str() {
                // Check if domain is in blocked list
                if self.blocked_domains.contains(host) {
                    return true;
                }
                
                // Check subdomains
                for blocked_domain in &self.blocked_domains {
                    if host.ends_with(blocked_domain) {
                        return true;
                    }
                }
            }
        }
        
        // Check against regex patterns
        for pattern in &self.blocked_patterns {
            if pattern.is_match(url) {
                return true;
            }
        }
        
        false
    }
    
    pub fn is_tracking_domain(&self, domain: &str) -> bool {
        self.tracking_domains.contains(domain) ||
        self.tracking_domains.iter().any(|blocked| domain.ends_with(blocked))
    }
    
    pub fn is_ad_domain(&self, domain: &str) -> bool {
        self.ad_domains.contains(domain) ||
        self.ad_domains.iter().any(|blocked| domain.ends_with(blocked))
    }
    
    pub fn get_blocked_count(&self) -> usize {
        self.blocked_domains.len()
    }
}

// Background task to periodically update block lists
pub async fn start_privacy_updater(privacy_engine: std::sync::Arc<tokio::sync::Mutex<PrivacyEngine>>) {
    let mut interval = interval(Duration::from_secs(3600 * 24)); // Update daily
    
    loop {
        interval.tick().await;
        
        let mut engine = privacy_engine.lock().await;
        if let Err(e) = engine.update_block_lists().await {
            eprintln!("Failed to update privacy lists: {}", e);
        }
    }
}
