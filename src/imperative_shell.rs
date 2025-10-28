// ACE Imperative Shell - Side Effects Layer
#![allow(dead_code)]
use crate::types::*;
use futures::stream::StreamExt;
use reqwest::Client;
use serde_json::json;

pub struct OllamaClient {
    config: OllamaConfig,
    client: Client,
}

impl OllamaClient {
    pub fn new(config: OllamaConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    pub async fn initialize(&self) -> Result<bool> {
        let url = format!("{}/api/tags", self.config.url);
        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => Ok(true),
            Ok(resp) => Err(format!("Ollama not available: {}", resp.status())),
            Err(e) => Err(format!("Connection failed: {}", e)),
        }
    }

    pub async fn generate(&self, prompt: &str) -> Result<String> {
        self.generate_with_thinking(prompt, false).await
    }

    pub async fn generate_with_thinking(&self, prompt: &str, enable_thinking: bool) -> Result<String> {
        let url = format!("{}/api/generate", self.config.url);
        let mut options = json!({
            "temperature": self.config.temperature,
            "num_predict": self.config.max_tokens.min(128),
            "num_ctx": self.config.context_window.min(512)
        });
        
        if enable_thinking {
            options["enable_thinking"] = json!(true);
        }
        
        let payload = json!({
            "model": self.config.model,
            "prompt": prompt,
            "stream": false,
            "options": options
        });

        let timeout = if enable_thinking { 
            std::time::Duration::from_secs(300) 
        } else { 
            std::time::Duration::from_secs(120) 
        };

        match self.client.post(&url).json(&payload).timeout(timeout).send().await {
            Ok(resp) if resp.status().is_success() => {
                let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
                Ok(json["response"].as_str().unwrap_or("").trim().to_string())
            }
            Ok(resp) => Err(format!("API error: {}", resp.status())),
            Err(e) => Err(format!("Generation failed: {}", e)),
        }
    }

    pub async fn generate_stream(
        &self,
        prompt: &str,
    ) -> Result<impl futures::Stream<Item = Result<String>>> {
        self.generate_stream_with_thinking(prompt, false).await
    }

    pub async fn generate_stream_with_thinking(
        &self,
        prompt: &str,
        enable_thinking: bool,
    ) -> Result<impl futures::Stream<Item = Result<String>>> {
        let url = format!("{}/api/generate", self.config.url);
        let mut options = json!({
            "temperature": self.config.temperature,
            "num_predict": self.config.max_tokens.min(128),
            "num_ctx": self.config.context_window.min(512)
        });
        
        if enable_thinking {
            options["enable_thinking"] = json!(true);
        }
        
        let payload = json!({
            "model": self.config.model,
            "prompt": prompt,
            "stream": true,
            "options": options
        });

        let timeout = if enable_thinking { 
            std::time::Duration::from_secs(300) 
        } else { 
            std::time::Duration::from_secs(120) 
        };

        let resp = self
            .client
            .post(&url)
            .json(&payload)
            .timeout(timeout)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("API error: {}", resp.status()));
        }

        let stream = resp.bytes_stream().map(|result| match result {
            Ok(bytes) => {
                let text = String::from_utf8_lossy(&bytes);
                for line in text.lines() {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                        if let Some(response) = json["response"].as_str() {
                            return Ok(response.to_string());
                        }
                    }
                }
                Ok(String::new())
            }
            Err(e) => Err(e.to_string()),
        });

        Ok(stream)
    }
}

// Logging functions
pub fn log_info(message: &str) {
    println!("ℹ️  {}", message);
}

pub fn log_success(message: &str) {
    println!("✅ {}", message);
}

pub fn log_error(message: &str) {
    println!("❌ {}", message);
}
