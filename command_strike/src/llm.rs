use anyhow::{Context, Result};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use serde_json;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::time::timeout;
#[allow(unused_imports)]
use futures_util::StreamExt;

// Constants for LLM configuration
const REQUEST_TIMEOUT_SECS: u64 = 120;
const DEFAULT_TEMPERATURE: f32 = 0.7;
const DEFAULT_MAX_TOKENS: u32 = 2048;

/// Configuration for the Ollama LLM service
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    /// The base URL for the Ollama API
    pub api_url: String,
    /// The model name to use (e.g., "gemma3:12b")
    pub model: String,
    /// Temperature setting for response generation (0.0-1.0)
    pub temperature: f32,
    /// Maximum tokens to generate
    pub max_tokens: u32,
    /// Request timeout in seconds
    pub timeout_secs: u64,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:11434".to_string(),
            model: "gemma3:12b".to_string(),
            temperature: DEFAULT_TEMPERATURE,
            max_tokens: DEFAULT_MAX_TOKENS,
            timeout_secs: REQUEST_TIMEOUT_SECS,
        }
    }
}

/// LLM service for interacting with Ollama
#[derive(Debug, Clone)]
pub struct OllamaClient {
    client: reqwest::Client,
    config: OllamaConfig,
}

/// History item for maintaining conversation context
#[derive(Debug, Clone)]
pub struct HistoryItem {
    pub user_input: String,
    pub command: String,
    pub result: String,
}

/// Represents a streaming response from the LLM
#[derive(Debug)]
pub struct StreamingResponse {
    pub receiver: mpsc::Receiver<String>,
    pub final_response: Arc<Mutex<Option<String>>>,
}

/// Request body for the Ollama API
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

/// Response from the Ollama API
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    #[allow(dead_code)]
    model: String,
    response: String,
    #[serde(default)]
    done: bool,
}

impl OllamaClient {
    /// Create a new Ollama client with default settings
    pub fn new() -> Result<Self> {
        Self::with_config(OllamaConfig::default())
    }

    /// Create a new Ollama client with custom configuration
    pub fn with_config(config: OllamaConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client, config })
    }

    /// Set the model to use (allows changing model without recreating client)
    pub fn set_model(&mut self, model: &str) {
        self.config.model = model.to_string();
        info!("Model set to: {}", model);
    }

    /// Set the temperature for generation
    pub fn set_temperature(&mut self, temperature: f32) {
        // Clamp temperature to valid range
        let temp = temperature.max(0.0).min(1.0);
        self.config.temperature = temp;
        debug!("Temperature set to: {}", temp);
    }

    /// Check if the Ollama service is available
    pub async fn check_available(&self) -> bool {
        match self.client.get(format!("{}/api/tags", self.config.api_url)).send().await {
            Ok(response) => response.status().is_success(),
            Err(e) => {
                warn!("Ollama service check failed: {}", e);
                false
            }
        }
    }

    /// Generate a command based on user input and conversation history
    pub async fn generate_command(&self, user_input: &str, history: &[HistoryItem]) -> Result<String> {
        let system_prompt = format!(
            "You are CommandStrike, a CTF assistant. Your task is to translate natural language security \
            requests into shell commands. Output ONLY the exact command to run, nothing else. \
            Make sure commands are safe, efficient, and well-formed. Optimize for Unix/Linux environments."
        );

        // Prepare context from history
        let history_prompt = if !history.is_empty() {
            let mut context = String::new();
            context.push_str("\nPrevious commands and results:\n");
            
            // Include up to 3 most recent commands for context
            for item in history.iter().rev().take(3) {
                context.push_str(&format!(
                    "Request: {}\nCommand: {}\nResult: {}\n\n",
                    item.user_input, item.command, item.result
                ));
            }
            context
        } else {
            String::new()
        };

        let full_prompt = format!(
            "{}\n\nUser request: {}\n\nGenerate a single shell command to fulfill this request:",
            history_prompt, user_input
        );

        debug!("Sending command generation prompt to Ollama");

        // Send the request to Ollama
        let response = self.generate_with_timeout(&full_prompt, Some(&system_prompt)).await?;
        
        // Clean and validate the response
        let command = self.clean_command_response(&response);
        
        Ok(command)
    }

    /// Interpret the results of a command execution
    pub async fn interpret_result(&self, result: &str, history: &[HistoryItem]) -> Result<String> {
        let system_prompt = String::from(
            "You are CommandStrike, a CTF assistant specialized in security and penetration testing. \
            Analyze command outputs and provide clear, concise explanations focusing on security implications. \
            Highlight important findings, vulnerabilities, and suggest next steps for further investigation. \
            Be thorough but prioritize the most critical information first."
        );

        // Include the last command for context
        let history_context = if !history.is_empty() {
            let last = history.last().unwrap();
            format!(
                "User request: {}\nCommand executed: {}\n\n",
                last.user_input, last.command
            )
        } else {
            String::new()
        };

        let prompt = format!(
            "{}Here is the output of the command:\n\n{}\n\nAnalyze these results, \
            explaining what they mean from a security perspective and suggesting valuable next steps:",
            history_context, result
        );

        debug!("Sending interpretation prompt to Ollama");
        
        self.generate_with_timeout(&prompt, Some(&system_prompt)).await
    }

    /// Stream a response from the Ollama API
    pub async fn stream_response(&self, 
                                prompt: &str, 
                                system: Option<&str>) -> Result<StreamingResponse> {
        let request = OllamaRequest {
            model: self.config.model.clone(),
            prompt: prompt.to_string(),
            system: system.map(ToString::to_string),
            stream: Some(true),
            options: Some(OllamaOptions {
                temperature: self.config.temperature,
                top_p: Some(0.9),
                top_k: None,
                max_tokens: Some(self.config.max_tokens),
            }),
        };

        let url = format!("{}/api/generate", self.config.api_url);
        
        // Create a channel for streaming responses
        let (tx, rx) = mpsc::channel(100);
        let final_response = Arc::new(Mutex::new(None));
        let final_response_clone = final_response.clone();
        
        // Create a client that won't timeout during streaming
        let streaming_client = reqwest::Client::new();
        
        // Clone what we need for the task to avoid lifetime issues
        let url = url.clone();
        let request_json = serde_json::to_string(&request)
            .context("Failed to serialize request to JSON")?;
        
        // Spawn a task to handle the streaming response
        tokio::spawn(async move {
            let resp = match streaming_client.post(url)
                .header("Content-Type", "application/json")
                .body(request_json)
                .send()
                .await {
                    Ok(r) => r,
                    Err(e) => {
                        let _ = tx.send(format!("Error: {}", e)).await;
                        return;
                    }
                };
            
            if !resp.status().is_success() {
                let error_text = match resp.text().await {
                    Ok(t) => t,
                    Err(e) => format!("Failed to read error response: {}", e),
                };
                let _ = tx.send(format!("API Error: {}", error_text)).await;
                return;
            }
            
            let mut stream = resp.bytes_stream();
            let mut full_response = String::new();
            
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        if let Ok(text) = String::from_utf8(chunk.to_vec()) {
                            // Each line is a separate JSON object
                            for line in text.lines() {
                                if let Ok(response) = serde_json::from_str::<OllamaResponse>(line) {
                                    let _ = tx.send(response.response.clone()).await;
                                    full_response.push_str(&response.response);
                                    
                                    if response.done {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(format!("Stream error: {}", e)).await;
                        break;
                    }
                }
            }
            
            // Store the full response
            if let Ok(mut guard) = final_response_clone.lock() {
                *guard = Some(full_response);
            }
        });
        
        Ok(StreamingResponse {
            receiver: rx,
            final_response,
        })
    }

    /// Generate a response with a timeout
    async fn generate_with_timeout(&self, prompt: &str, system: Option<&str>) -> Result<String> {
        let request = OllamaRequest {
            model: self.config.model.clone(),
            prompt: prompt.to_string(),
            system: system.map(ToString::to_string),
            // Explicitly set stream to false to get a complete response
            stream: Some(false),
            options: Some(OllamaOptions {
                temperature: self.config.temperature,
                top_p: Some(0.9),
                top_k: None,
                max_tokens: Some(self.config.max_tokens),
            }),
        };

        let url = format!("{}/api/generate", self.config.api_url);
        debug!("Sending request to Ollama API: {}", url);
        
        // Execute with timeout
        let timeout_duration = Duration::from_secs(self.config.timeout_secs);
        let response_future = self.client
            .post(&url)
            .json(&request)
            .send();
            
        let response = timeout(timeout_duration, response_future)
            .await
            .context("Request to Ollama API timed out")?
            .context("Failed to send request to Ollama API")?;
            
        if !response.status().is_success() {
            let error_text = response.text().await
                .context("Failed to read error response from Ollama API")?;
            anyhow::bail!("Ollama API error: {}", error_text);
        }

        // Get the response text
        let response_text = response.text().await
            .context("Failed to read response from Ollama API")?;
        
        debug!("Received response from Ollama API: {}", response_text);
        
        // Parse the response
        let ollama_response: OllamaResponse = serde_json::from_str(&response_text)
            .context("Failed to parse response from Ollama API")?;

        Ok(ollama_response.response.trim().to_string())
    }
    
    /// Clean and format command response from LLM
    fn clean_command_response(&self, response: &str) -> String {
        // Remove code block markers and leading/trailing whitespace
        let mut cleaned = response.trim().to_string();
        
        // Remove markdown code formatting if present
        if cleaned.starts_with("```") && cleaned.ends_with("```") {
            // Find the first newline
            if let Some(start_pos) = cleaned.find('\n') {
                // Find the last newline before the final ```
                if let Some(end_pos) = cleaned.rfind('\n') {
                    cleaned = cleaned[start_pos+1..end_pos].trim().to_string();
                }
            }
        } else if cleaned.starts_with('`') && cleaned.ends_with('`') {
            cleaned = cleaned[1..cleaned.len()-1].trim().to_string();
        }
        
        // Remove any "sh", "bash", or "shell" language specifiers at the beginning
        let language_prefixes = ["sh ", "bash ", "shell "];
        for prefix in language_prefixes.iter() {
            if cleaned.starts_with(prefix) {
                cleaned = cleaned[prefix.len()..].to_string();
                break;
            }
        }
        
        cleaned.trim().to_string()
    }
}

/// Helper function to test if Ollama is running
pub async fn check_ollama_running() -> bool {
    match reqwest::Client::new()
        .get("http://localhost:11434/api/tags")
        .timeout(Duration::from_secs(2))
        .send()
        .await
    {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

/// Helper function to validate the model exists
pub async fn validate_model(model: &str) -> Result<bool> {
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:11434/api/tags")
        .send()
        .await
        .context("Failed to connect to Ollama API")?;
        
    if !response.status().is_success() {
        return Ok(false);
    }
    
    #[derive(Deserialize)]
    struct ModelsResponse {
        models: Vec<ModelInfo>,
    }
    
    #[derive(Deserialize)]
    struct ModelInfo {
        name: String,
    }
    
    let models: ModelsResponse = response
        .json()
        .await
        .context("Failed to parse models response")?;
        
    Ok(models.models.iter().any(|m| m.name == model))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_clean_command_response() {
        let client = OllamaClient::new().unwrap();
        
        // Test code block cleaning
        assert_eq!(client.clean_command_response("```bash\nls -la\n```"), "ls -la");
        
        // Test inline code cleaning
        assert_eq!(client.clean_command_response("`ls -la`"), "ls -la");
        
        // Test language prefix removal
        assert_eq!(client.clean_command_response("sh echo hello"), "echo hello");
        assert_eq!(client.clean_command_response("bash echo hello"), "echo hello");
    }
}

