use super::error::ApiError;
use super::stats::{LiveRequest, RequestStats, StatsCollector};
use super::types::ThreadMessage;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    threads: Arc<RwLock<HashMap<String, ThreadState>>>,
    stats: StatsCollector,
    qwen_tokens: Arc<RwLock<Vec<String>>>,
    qwen_index: Arc<tokio::sync::Mutex<usize>>,
    qwen_models: Arc<RwLock<Option<Vec<reverse_api::qwen::models::Model>>>>,
    uploaded_files: Arc<RwLock<HashMap<String, reverse_api::qwen::models::QwenFile>>>,
}

pub struct ThreadState {
    pub created_at: u64,
    pub metadata: Option<serde_json::Value>,
    pub messages: Vec<ThreadMessage>,
    pub model: String,
    pub qwen_chat_id: Option<String>,
    pub qwen_parent_id: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        // Load tokens from .qwen_token file if present, each line is a token
        let mut tokens: Vec<String> = vec![];
        if let Ok(contents) = std::fs::read_to_string(".qwen_token") {
            tokens = contents
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();
        }

        Self {
            threads: Arc::new(RwLock::new(HashMap::new())),
            stats: StatsCollector::new(),
            qwen_tokens: Arc::new(RwLock::new(tokens)),
            qwen_index: Arc::new(tokio::sync::Mutex::new(0)),
            qwen_models: Arc::new(RwLock::new(None)),
            uploaded_files: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn set_qwen_token(&self, token: String) {
        let mut tokens = self.qwen_tokens.write().await;
        tokens.clear();
        tokens.push(token);
    }

    // Get next token in round-robin fashion. Returns None if no tokens configured
    pub async fn next_qwen_token(&self) -> Option<String> {
        let tokens = self.qwen_tokens.read().await;
        if tokens.is_empty() {
            return None;
        }
        let mut idx = self.qwen_index.lock().await;
        let token = tokens.get(*idx % tokens.len()).cloned();
        *idx = (*idx + 1) % tokens.len();
        token
    }

    pub async fn get_qwen_models(&self) -> Option<Vec<reverse_api::qwen::models::Model>> {
        self.qwen_models.read().await.clone()
    }

    pub async fn set_qwen_models(&self, models: Vec<reverse_api::qwen::models::Model>) {
        let mut qw_models = self.qwen_models.write().await;
        *qw_models = Some(models);
    }

    pub async fn store_uploaded_file(&self, file: reverse_api::qwen::models::QwenFile) -> String {
        let file_id = file.id.clone();
        let mut files = self.uploaded_files.write().await;
        files.insert(file_id.clone(), file);
        file_id
    }

    pub async fn get_uploaded_file(
        &self,
        file_id: &str,
    ) -> Option<reverse_api::qwen::models::QwenFile> {
        let files = self.uploaded_files.read().await;
        files.get(file_id).cloned()
    }

    pub async fn get_uploaded_files(
        &self,
        file_ids: &[String],
    ) -> Vec<reverse_api::qwen::models::QwenFile> {
        let files = self.uploaded_files.read().await;
        file_ids
            .iter()
            .filter_map(|id| files.get(id).cloned())
            .collect()
    }

    pub async fn record_request(
        &self,
        method: &str,
        path: &str,
        status: u16,
        duration: std::time::Duration,
        user_agent: &str,
    ) {
        self.stats
            .record_request(method, path, status, duration, user_agent)
            .await;
    }

    pub async fn get_stats(&self) -> RequestStats {
        self.stats.get_stats().await
    }

    pub async fn get_live_requests(&self) -> Vec<LiveRequest> {
        self.stats.get_live_requests().await
    }

    pub async fn create_thread(
        &self,
        messages: Vec<ThreadMessage>,
        metadata: Option<serde_json::Value>,
        model: &str,
    ) -> Result<(String, ThreadState), ApiError> {
        let thread_id = uuid::Uuid::new_v4().to_string();
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let thread_state = ThreadState {
            created_at,
            metadata,
            messages,
            model: model.to_string(),
            qwen_chat_id: None,
            qwen_parent_id: None,
        };

        let mut threads = self.threads.write().await;
        threads.insert(thread_id.clone(), thread_state.clone());

        Ok((thread_id, thread_state))
    }

    pub async fn get_thread(&self, thread_id: &str) -> Result<ThreadState, ApiError> {
        let threads = self.threads.read().await;
        threads
            .get(thread_id)
            .cloned()
            .ok_or_else(|| ApiError::not_found("Thread not found"))
    }

    pub async fn list_threads(&self) -> Vec<(String, ThreadState)> {
        let threads = self.threads.read().await;
        threads
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub async fn delete_thread(&self, thread_id: &str) -> Result<(), ApiError> {
        let mut threads = self.threads.write().await;
        threads
            .remove(thread_id)
            .ok_or_else(|| ApiError::not_found("Thread not found"))?;
        Ok(())
    }

    pub async fn add_message_to_thread(
        &self,
        thread_id: &str,
        role: String,
        content: String,
    ) -> Result<(), ApiError> {
        let mut threads = self.threads.write().await;
        let thread = threads
            .get_mut(thread_id)
            .ok_or_else(|| ApiError::not_found("Thread not found"))?;

        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        thread.messages.push(ThreadMessage {
            role,
            content,
            created_at: Some(created_at),
        });

        Ok(())
    }

    pub async fn update_thread(&self, thread_id: &str, state: ThreadState) -> Result<(), ApiError> {
        let mut threads = self.threads.write().await;
        threads.insert(thread_id.to_string(), state);
        Ok(())
    }
}

impl ThreadState {
    pub fn get_messages(&self) -> &[ThreadMessage] {
        &self.messages
    }

    pub fn add_message(&mut self, role: String, content: String) {
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.messages.push(ThreadMessage {
            role,
            content,
            created_at: Some(created_at),
        });
    }
}

impl Clone for ThreadState {
    fn clone(&self) -> Self {
        Self {
            created_at: self.created_at,
            metadata: self.metadata.clone(),
            messages: self.messages.clone(),
            model: self.model.clone(),
            qwen_chat_id: self.qwen_chat_id.clone(),
            qwen_parent_id: self.qwen_parent_id.clone(),
        }
    }
}
