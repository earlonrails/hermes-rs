use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{CreateChatCompletionRequestArgs, ChatCompletionRequestMessage},
};
use async_trait::async_trait;
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error};

use crate::base::*;
use crate::error::*;
use crate::registry::register_provider;

/// OpenAI provider profile
pub fn openai_profile() -> ProviderProfile {
    ProviderProfile {
        name: "openai".to_string(),
        api_mode: ApiMode::ChatCompletions,
        aliases: vec!["oai".to_string(), "openai-chat".to_string()],
        display_name: "OpenAI".to_string(),
        description: "OpenAI — GPT-4, GPT-3.5, and other models".to_string(),
        signup_url: "https://platform.openai.com/signup".to_string(),
        env_vars: vec!["OPENAI_API_KEY".to_string()],
        base_url: "https://api.openai.com/v1".to_string(),
        models_url: "https://api.openai.com/v1/models".to_string(),
        auth_type: AuthType::ApiKey,
        supports_health_check: true,
        fallback_models: vec![
            "gpt-4o".to_string(),
            "gpt-4o-mini".to_string(),
            "gpt-4-turbo-preview".to_string(),
            "gpt-4".to_string(),
            "gpt-3.5-turbo".to_string(),
        ],
        hostname: "api.openai.com".to_string(),
        default_headers: HashMap::new(),
        fixed_temperature: None,
        default_max_tokens: None,
        default_aux_model: "gpt-3.5-turbo".to_string(),
    }
}

/// OpenAI provider implementation
pub struct OpenAIProvider {
    profile: ProviderProfile,
    client: Client<OpenAIConfig>,
}

impl OpenAIProvider {
    pub fn new(api_key: Option<String>, base_url: Option<String>) -> Self {
        let mut config = OpenAIConfig::new();
        if let Some(key) = api_key {
            config = config.with_api_key(key);
        }
        if let Some(url) = base_url {
            config = config.with_api_base(url);
        }
        
        Self {
            profile: openai_profile(),
            client: Client::with_config(config),
        }
    }
    
    pub fn new_with_profile(api_key: Option<String>, base_url: Option<String>, profile: ProviderProfile) -> Self {
        let mut config = OpenAIConfig::new();
        if let Some(key) = api_key {
            config = config.with_api_key(key);
        }
        if let Some(url) = base_url {
            config = config.with_api_base(url);
        }
        
        Self { profile, client: Client::with_config(config) }
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    fn profile(&self) -> &ProviderProfile {
        &self.profile
    }
    
    async fn fetch_models(
        &self,
        api_key: Option<&str>,
        timeout: f64,
    ) -> Result<Vec<String>, ProviderError> {
        let client = if let Some(key) = api_key {
            let config = OpenAIConfig::new().with_api_key(key.to_string());
            Client::with_config(config)
        } else {
            self.client.clone()
        };
        
        let models = client.models().list().await
            .map_err(|e| ProviderError::ApiRequestFailed(e.to_string()))?;
        
        Ok(models.data.into_iter().map(|m| m.id).collect())
    }
    
    async fn create_chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ProviderError> {
        // Convert our request format to async-openai's format
        let mut api_messages: Vec<ChatCompletionRequestMessage> = Vec::new();
        
        for msg in request.messages {
            match msg.role {
                MessageRole::System => {
                    api_messages.push(ChatCompletionRequestMessage::System {
                        role: async_openai::types::Role::System,
                        content: msg.content,
                        name: msg.name,
                    });
                }
                MessageRole::User => {
                    api_messages.push(ChatCompletionRequestMessage::User {
                        role: async_openai::types::Role::User,
                        content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(msg.content),
                        name: msg.name,
                    });
                }
                MessageRole::Assistant => {
                    let tool_calls = msg.tool_calls.map(|tcs| {
                        tcs.into_iter().map(|tc| {
                            async_openai::types::ChatCompletionMessageToolCall {
                                id: tc.id,
                                r#type: async_openai::types::ChatCompletionToolType::Function,
                                function: async_openai::types::FunctionCall {
                                    name: tc.function.name,
                                    arguments: tc.function.arguments,
                                },
                            }
                        }).collect()
                    });
                    
                    api_messages.push(ChatCompletionRequestMessage::Assistant {
                        role: async_openai::types::Role::Assistant,
                        content: msg.content,
                        tool_calls,
                        name: msg.name,
                        function_call: None,
                    });
                }
                MessageRole::Tool => {
                    api_messages.push(ChatCompletionRequestMessage::Tool {
                        role: async_openai::types::Role::Tool,
                        content: msg.content,
                        tool_call_id: msg.tool_call_id,
                    });
                }
            }
        }
        
        // Convert tools if present
        let mut api_request = CreateChatCompletionRequestArgs::default()
            .model(&request.model)
            .messages(api_messages)
            .temperature(request.temperature.unwrap_or(0.7))
            .max_tokens(request.max_tokens.map(|t| t as i64));
        
        if let Some(top_p) = request.top_p {
            api_request = api_request.top_p(top_p as f64);
        }
        if let Some(stop) = request.stop {
            api_request = api_request.stop(stop);
        }
        if request.stream {
            api_request = api_request.stream(true);
        }
        
        // Handle tools - convert to async-openai format
        if let Some(tools) = request.tools {
            let api_tools: Vec<_> = tools.into_iter().map(|t| {
                async_openai::types::ChatCompletionTool {
                    r#type: async_openai::types::ChatCompletionToolType::Function,
                    function: async_openai::types::FunctionObject {
                        name: t.function.name,
                        description: t.function.description,
                        parameters: t.function.parameters,
                        strict: false,
                    },
                }
            }).collect();
            api_request = api_request.tools(api_tools);
        }
        
        // Handle tool_choice
        if let Some(tool_choice) = request.tool_choice {
            let api_tool_choice = match tool_choice {
                ToolChoice::None => async_openai::types::ToolChoice::None,
                ToolChoice::Auto => async_openai::types::ToolChoice::Auto,
                ToolChoice::Required => async_openai::types::ToolChoice::Required,
                ToolChoice::Specific(name) => async_openai::types::ToolChoice::Function(
                    async_openai::types::FunctionCall { name, arguments: "{}".to_string() }
                ),
            };
            api_request = api_request.tool_choice(api_tool_choice);
        }
        
        // Add extra body fields
        for (key, value) in request.extra_body {
            // For OpenAI, extra_body fields can be added to the request
            // This is a simplified approach - we'd need to handle specific fields
            debug!("Extra body field {}: {:?}", key, value);
        }
        
        let api_request = api_request.build()
            .map_err(|e| ProviderError::ConfigurationError(e.to_string()))?;
        
        let response = self.client.chat().create(api_request).await
            .map_err(|e| ProviderError::ApiRequestFailed(e.to_string()))?;
        
        // Convert response back to our format
        let choices = response.choices.into_iter().enumerate().map(|(i, choice)| {
            let message = match choice.message {
                async_openai::types::ChatCompletionMessage::System(s) => ChatMessage {
                    role: MessageRole::System,
                    content: s.content.unwrap_or_default(),
                    name: s.name,
                    tool_calls: None,
                    tool_call_id: None,
                },
                async_openai::types::ChatCompletionMessage::User(u) => ChatMessage {
                    role: MessageRole::User,
                    content: match u.content {
                        async_openai::types::ChatCompletionRequestUserMessageContent::Text(t) => t,
                        _ => String::new(),
                    },
                    name: u.name,
                    tool_calls: None,
                    tool_call_id: None,
                },
                async_openai::types::ChatCompletionMessage::Assistant(a) => {
                    let tool_calls = a.tool_calls.map(|tcs| {
                        tcs.into_iter().map(|tc| ToolCall {
                            id: tc.id,
                            r#type: tc.r#type.to_string(),
                            function: ToolFunction {
                                name: tc.function.name,
                                arguments: tc.function.arguments,
                            },
                        }).collect()
                    });
                    
                    ChatMessage {
                        role: MessageRole::Assistant,
                        content: a.content.unwrap_or_default(),
                        name: a.name,
                        tool_calls,
                        tool_call_id: None,
                    }
                }
                async_openai::types::ChatCompletionMessage::Tool(t) => ChatMessage {
                    role: MessageRole::Tool,
                    content: t.content,
                    name: None,
                    tool_calls: None,
                    tool_call_id: t.tool_call_id,
                },
            };
            
            Choice {
                index: i,
                message,
                finish_reason: choice.finish_reason,
            }
        }).collect();
        
        let usage = response.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });
        
        Ok(ChatCompletionResponse {
            id: response.id,
            model: response.model,
            created: response.created,
            choices,
            usage,
        })
    }
    
    async fn create_chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionStream, ProviderError> {
        // Similar to create_chat_completion but returns a stream
        let mut api_messages: Vec<ChatCompletionRequestMessage> = Vec::new();
        
        for msg in request.messages {
            match msg.role {
                MessageRole::System => {
                    api_messages.push(ChatCompletionRequestMessage::System {
                        role: async_openai::types::Role::System,
                        content: msg.content,
                        name: msg.name,
                    });
                }
                MessageRole::User => {
                    api_messages.push(ChatCompletionRequestMessage::User {
                        role: async_openai::types::Role::User,
                        content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(msg.content),
                        name: msg.name,
                    });
                }
                MessageRole::Assistant => {
                    let tool_calls = msg.tool_calls.map(|tcs| {
                        tcs.into_iter().map(|tc| {
                            async_openai::types::ChatCompletionMessageToolCall {
                                id: tc.id,
                                r#type: async_openai::types::ChatCompletionToolType::Function,
                                function: async_openai::types::FunctionCall {
                                    name: tc.function.name,
                                    arguments: tc.function.arguments,
                                },
                            }
                        }).collect()
                    });
                    
                    api_messages.push(ChatCompletionRequestMessage::Assistant {
                        role: async_openai::types::Role::Assistant,
                        content: msg.content,
                        tool_calls,
                        name: msg.name,
                        function_call: None,
                    });
                }
                MessageRole::Tool => {
                    api_messages.push(ChatCompletionRequestMessage::Tool {
                        role: async_openai::types::Role::Tool,
                        content: msg.content,
                        tool_call_id: msg.tool_call_id,
                    });
                }
            }
        }
        
        let mut api_request = CreateChatCompletionRequestArgs::default()
            .model(&request.model)
            .messages(api_messages)
            .temperature(request.temperature.unwrap_or(0.7))
            .max_tokens(request.max_tokens.map(|t| t as i64))
            .stream(true);
        
        if let Some(top_p) = request.top_p {
            api_request = api_request.top_p(top_p as f64);
        }
        if let Some(stop) = request.stop {
            api_request = api_request.stop(stop);
        }
        
        let api_request = api_request.build()
            .map_err(|e| ProviderError::ConfigurationError(e.to_string()))?;
        
        let stream = self.client.chat().create_stream(api_request).await
            .map_err(|e| ProviderError::StreamingError(e.to_string()))?;
        
        // Convert the async-openai stream to our format
        let converted_stream = futures::stream::try_map(stream, |chunk| {
            let chunk = chunk.map_err(|e| ProviderError::StreamingError(e.to_string()))?;
            
            let choices = chunk.choices.into_iter().enumerate().map(|(i, choice)| {
                let delta_content = match &choice.delta {
                    async_openai::types::ChatCompletionStreamDelta::Content { content, .. } => {
                        content.clone()
                    }
                    _ => String::new(),
                };
                
                let role = match &choice.delta {
                    async_openai::types::ChatCompletionStreamDelta::Role { role, .. } => {
                        Some(match role {
                            async_openai::types::Role::System => MessageRole::System,
                            async_openai::types::Role::User => MessageRole::User,
                            async_openai::types::Role::Assistant => MessageRole::Assistant,
                            async_openai::types::Role::Tool => MessageRole::Tool,
                        })
                    }
                    _ => None,
                };
                
                let tool_calls = match &choice.delta {
                    async_openai::types::ChatCompletionStreamDelta::ToolCall { tool_calls, .. } => {
                        tool_calls.as_ref().map(|tcs| {
                            tcs.iter().map(|tc| StreamToolCall {
                                id: tc.id.clone(),
                                r#type: Some(tc.r#type.to_string()),
                                function: tc.function.as_ref().map(|f| StreamToolFunction {
                                    name: Some(f.name.clone()),
                                    arguments: f.arguments.clone(),
                                }),
                            }).collect()
                        })
                    }
                    _ => None,
                };
                
                StreamChoice {
                    index: i,
                    delta: StreamDelta {
                        role,
                        content: Some(delta_content),
                        tool_calls,
                    },
                    finish_reason: choice.finish_reason,
                }
            }).collect();
            
            Ok(StreamChunk {
                id: chunk.id,
                model: chunk.model,
                created: Some(chunk.created),
                choices,
            })
        });
        
        Ok(ChatCompletionStream {
            response: Box::new(converted_stream),
        })
    }
}

/// Register the OpenAI provider
pub fn register() {
    register_provider(Arc::new(OpenAIProvider::new(None, None)));
}
