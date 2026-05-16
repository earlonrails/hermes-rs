use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error};
use futures::StreamExt;
use eventsource_stream::Eventsource;

use crate::base::*;
use crate::error::*;
use crate::registry::register_provider;

/// Anthropic provider profile
pub fn anthropic_profile() -> ProviderProfile {
    let mut default_headers = HashMap::new();
    default_headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());
    
    ProviderProfile {
        name: "anthropic".to_string(),
        api_mode: ApiMode::AnthropicMessages,
        aliases: vec![
            "claude".to_string(),
            "claude-oauth".to_string(),
            "claude-code".to_string(),
        ],
        display_name: "Anthropic".to_string(),
        description: "Anthropic — Claude models via Messages API".to_string(),
        signup_url: "https://www.anthropic.com/".to_string(),
        env_vars: vec![
            "ANTHROPIC_API_KEY".to_string(),
            "ANTHROPIC_TOKEN".to_string(),
            "CLAUDE_CODE_OAUTH_TOKEN".to_string(),
        ],
        base_url: "https://api.anthropic.com".to_string(),
        models_url: "https://api.anthropic.com/v1/models".to_string(),
        auth_type: AuthType::ApiKey,
        supports_health_check: true,
        fallback_models: vec![
            "claude-3-5-sonnet-20251022".to_string(),
            "claude-3-7-sonnet-20250219".to_string(),
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-haiku-20240307".to_string(),
        ],
        hostname: "api.anthropic.com".to_string(),
        default_headers,
        fixed_temperature: None,
        default_max_tokens: None,
        default_aux_model: "claude-3-haiku-20240307".to_string(),
    }
}

/// Anthropic provider implementation
pub struct AnthropicProvider {
    profile: ProviderProfile,
}

impl AnthropicProvider {
    pub fn new() -> Self {
        Self {
            profile: anthropic_profile(),
        }
    }
    
    pub fn new_with_profile(profile: ProviderProfile) -> Self {
        Self { profile }
    }

    fn build_anthropic_request(&self, request: &ChatCompletionRequest) -> serde_json::Value {
        let mut body = serde_json::json!({
            "model": request.model,
            "max_tokens": request.max_tokens.unwrap_or(4096),
        });

        let mut messages = Vec::new();
        let mut system_prompt = String::new();

        for msg in &request.messages {
            match msg.role {
                MessageRole::System => {
                    system_prompt.push_str(&msg.content);
                    system_prompt.push('\n');
                }
                MessageRole::User => {
                    messages.push(serde_json::json!({
                        "role": "user",
                        "content": msg.content
                    }));
                }
                MessageRole::Assistant => {
                    let mut content = Vec::new();
                    if !msg.content.is_empty() {
                        content.push(serde_json::json!({
                            "type": "text",
                            "text": msg.content
                        }));
                    }
                    if let Some(tool_calls) = &msg.tool_calls {
                        for tc in tool_calls {
                            let input: serde_json::Value = serde_json::from_str(&tc.function.arguments)
                                .unwrap_or_else(|_| serde_json::json!({}));
                            content.push(serde_json::json!({
                                "type": "tool_use",
                                "id": tc.id,
                                "name": tc.function.name,
                                "input": input
                            }));
                        }
                    }
                    if content.is_empty() {
                        content.push(serde_json::json!({
                            "type": "text",
                            "text": ""
                        }));
                    }
                    messages.push(serde_json::json!({
                        "role": "assistant",
                        "content": content
                    }));
                }
                MessageRole::Tool => {
                    messages.push(serde_json::json!({
                        "role": "user",
                        "content": [{
                            "type": "tool_result",
                            "tool_use_id": msg.tool_call_id.clone().unwrap_or_default(),
                            "content": msg.content
                        }]
                    }));
                }
            }
        }
        
        if !system_prompt.is_empty() {
            body["system"] = serde_json::Value::String(system_prompt.trim().to_string());
        }

        // Merge adjacent messages of the same role (Anthropic requires alternating roles)
        let mut merged_messages = Vec::new();
        for msg in messages {
            if let Some(last) = merged_messages.last_mut() {
                if last["role"] == msg["role"] {
                    let mut last_content = if last["content"].is_array() {
                        if let Some(arr) = last["content"].as_array() {
                            arr.clone()
                        } else {
                            vec![serde_json::json!({"type": "text", "text": last["content"].as_str().unwrap_or_default()})]
                        }
                    } else {
                        vec![serde_json::json!({"type": "text", "text": last["content"].as_str().unwrap_or_default()})]
                    };
                    
                    let msg_content = if msg["content"].is_array() {
                        if let Some(arr) = msg["content"].as_array() {
                            arr.clone()
                        } else {
                            vec![serde_json::json!({"type": "text", "text": msg["content"].as_str().unwrap_or_default()})]
                        }
                    } else {
                        vec![serde_json::json!({"type": "text", "text": msg["content"].as_str().unwrap_or_default()})]
                    };
                    
                    last_content.extend(msg_content);
                    *last = serde_json::json!({
                        "role": last["role"].clone(),
                        "content": last_content
                    });
                    continue;
                }
            }
            merged_messages.push(msg);
        }

        body["messages"] = serde_json::Value::Array(merged_messages);

        if let Some(temp) = request.temperature {
            if let Some(n) = serde_json::Number::from_f64(temp as f64) {
                body["temperature"] = serde_json::Value::Number(n);
            }
        }
        if let Some(top_p) = request.top_p {
            if let Some(n) = serde_json::Number::from_f64(top_p as f64) {
                body["top_p"] = serde_json::Value::Number(n);
            }
        }
        if let Some(stop) = &request.stop {
            body["stop_sequences"] = serde_json::Value::Array(
                stop.iter().map(|s| serde_json::Value::String(s.clone())).collect()
            );
        }

        if let Some(tools) = &request.tools {
            let anthropic_tools: Vec<_> = tools.iter().map(|t| {
                let mut tool_obj = serde_json::json!({
                    "name": t.function.name,
                    "input_schema": t.function.parameters
                });
                if let Some(desc) = &t.function.description {
                    tool_obj["description"] = serde_json::Value::String(desc.clone());
                }
                tool_obj
            }).collect();
            if !anthropic_tools.is_empty() {
                body["tools"] = serde_json::Value::Array(anthropic_tools);
            }
        }

        if let Some(tool_choice) = &request.tool_choice {
            match tool_choice {
                ToolChoice::Auto => {
                    body["tool_choice"] = serde_json::json!({"type": "auto"});
                }
                ToolChoice::Required => {
                    body["tool_choice"] = serde_json::json!({"type": "any"});
                }
                ToolChoice::Specific(name) => {
                    body["tool_choice"] = serde_json::json!({"type": "tool", "name": name});
                }
                _ => {}
            }
        }

        for (key, value) in &request.extra_body {
            body[key] = value.clone();
        }

        body
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    fn profile(&self) -> &ProviderProfile {
        &self.profile
    }
    
    async fn fetch_models(
        &self,
        api_key: Option<&str>,
        _timeout: f64,
    ) -> Result<Vec<String>, ProviderError> {
        let client = reqwest::Client::new();
        let url = "https://api.anthropic.com/v1/models";
        
        let mut request = client.request(reqwest::Method::GET, url);
        
        if let Some(key) = api_key {
            request = request.header("x-api-key", key);
        }
        
        request = request
            .header("anthropic-version", "2023-06-01")
            .header("Accept", "application/json");
        
        for (key, value) in &self.profile.default_headers {
            request = request.header(key, value);
        }
        
        let req = request.build().map_err(|e| ProviderError::ApiRequestFailed(e.to_string()))?;
        let response = client.execute(req)
            .await
            .map_err(|e| ProviderError::ApiRequestFailed(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(ProviderError::ApiRequestFailed(format!(
                "HTTP {}: {}", 
                response.status(), 
                response.text().await.unwrap_or_default()
            )));
        }
        
        let data: serde_json::Value = response.json().await
            .map_err(|e| ProviderError::InvalidResponseFormat(e.to_string()))?;
        
        let models = data.get("data").and_then(|d| d.as_array()).unwrap_or(&Vec::new());
        
        Ok(models.iter()
            .filter_map(|m| m.get("id").and_then(|id| id.as_str()).map(|s| s.to_string()))
            .collect())
    }
    
    async fn create_chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ProviderError> {
        let body = self.build_anthropic_request(&request);
        
        let client = reqwest::Client::new();
        let url = format!("{}/v1/messages", self.profile.base_url);
        
        let mut request_builder = client.request(reqwest::Method::POST, &url);
        
        if let Some(api_key) = std::env::var("ANTHROPIC_API_KEY").ok() {
            request_builder = request_builder.header("x-api-key", api_key);
        }
        
        request_builder = request_builder
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");
        
        for (key, value) in &self.profile.default_headers {
            request_builder = request_builder.header(key, value);
        }
        
        let response = request_builder
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::ApiRequestFailed(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(ProviderError::ApiRequestFailed(format!(
                "HTTP {}: {}", 
                response.status(), 
                response.text().await.unwrap_or_default()
            )));
        }
        
        let data: serde_json::Value = response.json().await
            .map_err(|e| ProviderError::InvalidResponseFormat(e.to_string()))?;
        
        let content_blocks = data.get("content").and_then(|c| c.as_array()).unwrap_or(&Vec::new());
        let mut text_content = String::new();
        let mut tool_calls = Vec::new();

        for block in content_blocks {
            if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                if let Some(t) = block.get("text").and_then(|t| t.as_str()) {
                    text_content.push_str(t);
                }
            } else if block.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                if let (Some(id), Some(name), Some(input)) = (
                    block.get("id").and_then(|i| i.as_str()),
                    block.get("name").and_then(|n| n.as_str()),
                    block.get("input")
                ) {
                    tool_calls.push(ToolCall {
                        id: id.to_string(),
                        r#type: "function".to_string(),
                        function: ToolFunction {
                            name: name.to_string(),
                            arguments: serde_json::to_string(input).unwrap_or_default(),
                        }
                    });
                }
            }
        }
        
        let choice = Choice {
            index: 0,
            message: ChatMessage {
                role: MessageRole::Assistant,
                content: text_content,
                name: None,
                tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls) },
                tool_call_id: None,
            },
            finish_reason: data.get("stop_reason").and_then(|s| s.as_str()).map(|s| s.to_string()),
        };
        
        let usage = data.get("usage").and_then(|u| {
            Some(Usage {
                prompt_tokens: u.get("input_tokens").and_then(|t| t.as_u64()).unwrap_or(0),
                completion_tokens: u.get("output_tokens").and_then(|t| t.as_u64()).unwrap_or(0),
                total_tokens: u.get("input_tokens").and_then(|t| t.as_u64()).unwrap_or(0) +
                             u.get("output_tokens").and_then(|t| t.as_u64()).unwrap_or(0),
            })
        });
        
        Ok(ChatCompletionResponse {
            id: data.get("id").and_then(|id| id.as_str()).unwrap_or_default().to_string(),
            model: data.get("model").and_then(|m| m.as_str()).unwrap_or_default().to_string(),
            created: 0,
            choices: vec![choice],
            usage,
        })
    }
    
    async fn create_chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionStream, ProviderError> {
        let mut body = self.build_anthropic_request(&request);
        body["stream"] = serde_json::Value::Bool(true);
        
        let client = reqwest::Client::new();
        let url = format!("{}/v1/messages", self.profile.base_url);
        
        let mut request_builder = client.request(reqwest::Method::POST, &url);
        
        if let Some(api_key) = std::env::var("ANTHROPIC_API_KEY").ok() {
            request_builder = request_builder.header("x-api-key", api_key);
        }
        
        request_builder = request_builder
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");
        
        for (key, value) in &self.profile.default_headers {
            request_builder = request_builder.header(key, value);
        }
        
        let response = request_builder
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::ApiRequestFailed(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(ProviderError::ApiRequestFailed(format!(
                "HTTP {}: {}", 
                response.status(), 
                response.text().await.unwrap_or_default()
            )));
        }

        let stream = response.bytes_stream().eventsource();
        
        let mapped_stream = stream.map(|event_res| {
            let event = event_res.map_err(|e| ProviderError::StreamingError(e.to_string()))?;
            let data: serde_json::Value = serde_json::from_str(&event.data)
                .map_err(|e| ProviderError::StreamingError(e.to_string()))?;
                
            let event_type = event.event;
            let mut choices = Vec::new();
            
            match event_type.as_str() {
                "message_start" => {
                    let role = data["message"]["role"].as_str().map(|r| match r {
                        "user" => MessageRole::User,
                        "assistant" => MessageRole::Assistant,
                        _ => MessageRole::Assistant,
                    });
                    choices.push(StreamChoice {
                        index: 0,
                        delta: StreamDelta {
                            role,
                            content: None,
                            tool_calls: None,
                        },
                        finish_reason: None,
                    });
                }
                "content_block_start" => {
                    if data["content_block"]["type"] == "tool_use" {
                        let id = data["content_block"]["id"].as_str().map(|s| s.to_string());
                        let name = data["content_block"]["name"].as_str().map(|s| s.to_string());
                        choices.push(StreamChoice {
                            index: 0,
                            delta: StreamDelta {
                                role: None,
                                content: None,
                                tool_calls: Some(vec![StreamToolCall {
                                    id,
                                    r#type: Some("function".to_string()),
                                    function: Some(StreamToolFunction {
                                        name,
                                        arguments: Some(String::new()),
                                    }),
                                }]),
                            },
                            finish_reason: None,
                        });
                    }
                }
                "content_block_delta" => {
                    if data["delta"]["type"] == "text_delta" {
                        if let Some(text) = data["delta"]["text"].as_str() {
                            choices.push(StreamChoice {
                                index: 0,
                                delta: StreamDelta {
                                    role: None,
                                    content: Some(text.to_string()),
                                    tool_calls: None,
                                },
                                finish_reason: None,
                            });
                        }
                    } else if data["delta"]["type"] == "input_json_delta" {
                        if let Some(partial_json) = data["delta"]["partial_json"].as_str() {
                            choices.push(StreamChoice {
                                index: 0,
                                delta: StreamDelta {
                                    role: None,
                                    content: None,
                                    tool_calls: Some(vec![StreamToolCall {
                                        id: None,
                                        r#type: None,
                                        function: Some(StreamToolFunction {
                                            name: None,
                                            arguments: Some(partial_json.to_string()),
                                        }),
                                    }]),
                                },
                                finish_reason: None,
                            });
                        }
                    }
                }
                "message_delta" => {
                    if let Some(stop_reason) = data["delta"]["stop_reason"].as_str() {
                        choices.push(StreamChoice {
                            index: 0,
                            delta: StreamDelta {
                                role: None,
                                content: None,
                                tool_calls: None,
                            },
                            finish_reason: Some(stop_reason.to_string()),
                        });
                    }
                }
                _ => {} // Ignore other event types
            }
            
            Ok(StreamChunk {
                id: data["message"]["id"].as_str().unwrap_or_default().to_string(),
                model: String::new(),
                created: None,
                choices,
            })
        });

        Ok(ChatCompletionStream {
            response: Box::new(mapped_stream),
        })
    }
}

/// Register the Anthropic provider
pub fn register() {
    register_provider(Arc::new(AnthropicProvider::new()));
}
