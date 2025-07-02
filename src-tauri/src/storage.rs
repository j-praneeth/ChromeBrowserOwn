use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Row};
use chrono::{DateTime, Utc};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub folder: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub visit_count: i32,
    pub last_visited: DateTime<Utc>,
}

pub struct StorageManager {
    pool: Option<SqlitePool>,
    db_path: PathBuf,
}

impl Default for StorageManager {
    fn default() -> Self {
        let mut db_path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        db_path.push("PrivacyBrowser");
        std::fs::create_dir_all(&db_path).ok();
        db_path.push("browser.db");
        
        Self {
            pool: None,
            db_path,
        }
    }
}

impl StorageManager {
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let database_url = format!("sqlite:{}", self.db_path.display());
        
        let pool = SqlitePool::connect(&database_url).await?;
        
        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bookmarks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                title TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                folder TEXT
            )
            "#
        )
        .execute(&pool)
        .await?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                visit_count INTEGER DEFAULT 1,
                last_visited DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&pool)
        .await?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )
            "#
        )
        .execute(&pool)
        .await?;
        
        // Create indexes for better performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_bookmarks_url ON bookmarks(url)")
            .execute(&pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_history_url ON history(url)")
            .execute(&pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_history_last_visited ON history(last_visited)")
            .execute(&pool)
            .await?;
        
        self.pool = Some(pool);
        Ok(())
    }
    
    pub async fn add_bookmark(&mut self, url: &str, title: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(pool) = &self.pool {
            sqlx::query(
                "INSERT INTO bookmarks (url, title) VALUES (?1, ?2)"
            )
            .bind(url)
            .bind(title)
            .execute(pool)
            .await?;
        }
        Ok(())
    }
    
    pub async fn get_bookmarks(&mut self) -> Result<Vec<Bookmark>, Box<dyn std::error::Error>> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(
                "SELECT id, url, title, created_at, folder FROM bookmarks ORDER BY created_at DESC"
            )
            .fetch_all(pool)
            .await?;
            
            let mut bookmarks = Vec::new();
            for row in rows {
                let created_at_str: String = row.get("created_at");
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc);
                
                bookmarks.push(Bookmark {
                    id: row.get("id"),
                    url: row.get("url"),
                    title: row.get("title"),
                    created_at,
                    folder: row.get("folder"),
                });
            }
            return Ok(bookmarks);
        }
        Ok(vec![])
    }
    
    pub async fn remove_bookmark(&mut self, id: i64) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(pool) = &self.pool {
            sqlx::query("DELETE FROM bookmarks WHERE id = ?1")
                .bind(id)
                .execute(pool)
                .await?;
        }
        Ok(())
    }
    
    pub async fn add_history_entry(&mut self, url: &str, title: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(pool) = &self.pool {
            // Try to update existing entry first
            let updated = sqlx::query(
                "UPDATE history SET visit_count = visit_count + 1, last_visited = CURRENT_TIMESTAMP, title = ?1 WHERE url = ?2"
            )
            .bind(title)
            .bind(url)
            .execute(pool)
            .await?;
            
            // If no rows were updated, insert new entry
            if updated.rows_affected() == 0 {
                sqlx::query(
                    "INSERT INTO history (url, title) VALUES (?1, ?2)"
                )
                .bind(url)
                .bind(title)
                .execute(pool)
                .await?;
            }
        }
        Ok(())
    }
    
    pub async fn get_history(&mut self) -> Result<Vec<HistoryEntry>, Box<dyn std::error::Error>> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(
                "SELECT id, url, title, visit_count, last_visited FROM history ORDER BY last_visited DESC LIMIT 1000"
            )
            .fetch_all(pool)
            .await?;
            
            let mut history = Vec::new();
            for row in rows {
                let last_visited_str: String = row.get("last_visited");
                let last_visited = DateTime::parse_from_rfc3339(&last_visited_str)
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc);
                
                history.push(HistoryEntry {
                    id: row.get("id"),
                    url: row.get("url"),
                    title: row.get("title"),
                    visit_count: row.get("visit_count"),
                    last_visited,
                });
            }
            return Ok(history);
        }
        Ok(vec![])
    }
    
    pub async fn clear_history(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(pool) = &self.pool {
            sqlx::query("DELETE FROM history")
                .execute(pool)
                .await?;
        }
        Ok(())
    }
    
    pub async fn search_history(&mut self, query: &str) -> Result<Vec<HistoryEntry>, Box<dyn std::error::Error>> {
        if let Some(pool) = &self.pool {
            let search_term = format!("%{}%", query);
            let rows = sqlx::query(
                "SELECT id, url, title, visit_count, last_visited FROM history 
                 WHERE title LIKE ?1 OR url LIKE ?1 
                 ORDER BY visit_count DESC, last_visited DESC 
                 LIMIT 50"
            )
            .bind(&search_term)
            .fetch_all(pool)
            .await?;
            
            let mut results = Vec::new();
            for row in rows {
                let last_visited_str: String = row.get("last_visited");
                let last_visited = DateTime::parse_from_rfc3339(&last_visited_str)
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc);
                
                results.push(HistoryEntry {
                    id: row.get("id"),
                    url: row.get("url"),
                    title: row.get("title"),
                    visit_count: row.get("visit_count"),
                    last_visited,
                });
            }
            return Ok(results);
        }
        Ok(vec![])
    }
}
