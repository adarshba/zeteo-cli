use anyhow::Result;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// A serializable chat message for session storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub role: String,
    pub content: String,
}

/// Conversation session info stored in Redis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationInfo {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub message_count: usize,
}

const SESSION_PREFIX: &str = "zeteo:session:";
const SESSION_LIST_KEY: &str = "zeteo:sessions";
const SESSION_TTL_SECONDS: i64 = 60 * 60 * 24 * 7; // 7 days

/// Redis-backed session store for conversation persistence
pub struct SessionStore {
    conn: Arc<Mutex<ConnectionManager>>,
    current_session_id: String,
}

impl SessionStore {
    /// Create a new session store with a Redis connection
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let conn = ConnectionManager::new(client).await?;
        let session_id = Uuid::new_v4().to_string();
        
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            current_session_id: session_id,
        })
    }

    /// Get the current session ID
    pub fn current_session_id(&self) -> &str {
        &self.current_session_id
    }

    /// Set the current session ID (for resuming sessions)
    pub fn set_current_session_id(&mut self, id: String) {
        self.current_session_id = id;
    }

    /// Save messages to the current session
    pub async fn save_messages(&self, messages: &[StoredMessage]) -> Result<()> {
        let key = format!("{}{}", SESSION_PREFIX, self.current_session_id);
        let data = serde_json::to_string(messages)?;
        
        let mut conn = self.conn.lock().await;
        conn.set_ex::<_, _, ()>(&key, &data, SESSION_TTL_SECONDS as u64).await?;
        
        // Update session list
        let title = if let Some(first_msg) = messages.iter().find(|m| m.role == "user") {
            let title = first_msg.content.chars().take(50).collect::<String>();
            if first_msg.content.len() > 50 {
                format!("{}...", title)
            } else {
                title
            }
        } else {
            "New conversation".to_string()
        };
        
        let info = ConversationInfo {
            id: self.current_session_id.clone(),
            title,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            message_count: messages.len(),
        };
        
        let info_json = serde_json::to_string(&info)?;
        conn.hset::<_, _, _, ()>(SESSION_LIST_KEY, &self.current_session_id, &info_json).await?;
        
        Ok(())
    }

    /// Load messages from a session
    pub async fn load_messages(&self, session_id: &str) -> Result<Vec<StoredMessage>> {
        let key = format!("{}{}", SESSION_PREFIX, session_id);
        
        let mut conn = self.conn.lock().await;
        let data: Option<String> = conn.get(&key).await?;
        
        match data {
            Some(json) => {
                let messages: Vec<StoredMessage> = serde_json::from_str(&json)?;
                Ok(messages)
            }
            None => Ok(Vec::new()),
        }
    }

    /// List all available sessions
    pub async fn list_sessions(&self) -> Result<Vec<ConversationInfo>> {
        let mut conn = self.conn.lock().await;
        let sessions: std::collections::HashMap<String, String> = 
            conn.hgetall(SESSION_LIST_KEY).await?;
        
        let mut result: Vec<ConversationInfo> = sessions
            .values()
            .filter_map(|json| serde_json::from_str(json).ok())
            .collect();
        
        // Sort by updated_at, most recent first
        result.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        
        // Limit to last 20 sessions
        result.truncate(20);
        
        Ok(result)
    }

    /// Delete a session
    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        let key = format!("{}{}", SESSION_PREFIX, session_id);
        
        let mut conn = self.conn.lock().await;
        conn.del::<_, ()>(&key).await?;
        conn.hdel::<_, _, ()>(SESSION_LIST_KEY, session_id).await?;
        
        Ok(())
    }

    /// Clear the current session's messages
    pub async fn clear_current_session(&self) -> Result<()> {
        let key = format!("{}{}", SESSION_PREFIX, self.current_session_id);
        
        let mut conn = self.conn.lock().await;
        conn.del::<_, ()>(&key).await?;
        conn.hdel::<_, _, ()>(SESSION_LIST_KEY, &self.current_session_id).await?;
        
        Ok(())
    }
}

const DEFAULT_REDIS_URL: &str = "redis://localhost:6379";

/// Try to create a session store, returning None if Redis is not available
pub async fn try_create_session_store() -> Option<SessionStore> {
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| DEFAULT_REDIS_URL.to_string());
    SessionStore::new(&redis_url).await.ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stored_message_serialization() {
        let msg = StoredMessage {
            role: "user".to_string(),
            content: "Hello world".to_string(),
        };
        
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: StoredMessage = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.role, "user");
        assert_eq!(deserialized.content, "Hello world");
    }

    #[test]
    fn test_conversation_info_serialization() {
        let info = ConversationInfo {
            id: "test-id".to_string(),
            title: "Test conversation".to_string(),
            created_at: 1234567890,
            updated_at: 1234567890,
            message_count: 5,
        };
        
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: ConversationInfo = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.id, "test-id");
        assert_eq!(deserialized.title, "Test conversation");
        assert_eq!(deserialized.message_count, 5);
    }
}
