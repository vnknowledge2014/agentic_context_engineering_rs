// ACE Types - Functional Type Definitions
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Result type for Railway-Oriented Programming
pub type Result<T> = std::result::Result<T, String>;

// ACE Domain Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBullet {
    pub id: String,
    pub content: String,
    pub helpful_count: i32,
    pub harmful_count: i32,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ReasoningStep {
    pub description: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Trajectory {
    pub query: String,
    pub steps: Vec<ReasoningStep>,
    pub outcome: String,
    pub success: bool,
    pub used_bullets: Vec<String>,
    pub feedback: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Insight {
    pub content: String,
    pub insight_type: String,
    pub confidence: f64,
    pub source_id: String,
}

#[derive(Debug, Clone)]
pub struct DeltaUpdate {
    pub bullets: Vec<ContextBullet>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ContextState {
    pub bullets: HashMap<String, ContextBullet>,
    pub version: i32,
}

#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub url: String,
    pub model: String,
    pub temperature: f64,
    pub max_tokens: i32,
    pub context_window: i32,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:11434".to_string(),
            model: "qwen2.5-coder:1.5b".to_string(),
            temperature: 0.7,
            max_tokens: 256,
            context_window: 2048,
        }
    }
}

impl ContextState {
    pub fn new() -> Self {
        Self {
            bullets: HashMap::new(),
            version: 0,
        }
    }
}
