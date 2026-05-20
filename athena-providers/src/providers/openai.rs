use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{CreateChatCompletionRequestArgs, ChatCompletionRequestMessage},
};
use async_trait::async_trait;
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

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
    
    fn map_messages(&self, messages: Vec<ChatMessage>) -> Vec<ChatCompletionRequestMessage> {
        let mut api_messages = Vec::new();
        for msg in messages {
            match msg.role {
                MessageRole::System => {
                    let sys_msg = async_openai::types::ChatCompletionRequestSystemMessage {
                        content: msg.content,
                        role: async_openai::types::Role::System,
                        name: msg.name,
                    };
                    api_messages.push(ChatCompletionRequestMessage::System(sys_msg));
                }
                MessageRole::User => {
                    let user_msg = async_openai::types::ChatCompletionRequestUserMessage {
                        content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(msg.content),
                        role: async_openai::types::Role::User,
                        name: msg.name,
                    };
                    api_messages.push(ChatCompletionRequestMessage::User(user_msg));
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
                    
                    let assistant_msg = async_openai::types::ChatCompletionRequestAssistantMessage {
                        content: if msg.content.is_empty() { None } else { Some(msg.content) },
                        role: async_openai::types::Role::Assistant,
                        name: msg.name,
                        tool_calls,
                        function_call: None,
                    };
                    api_messages.push(ChatCompletionRequestMessage::Assistant(assistant_msg));
                }
                MessageRole::Tool => {
                    let tool_msg = async_openai::types::ChatCompletionRequestToolMessage {
                        content: msg.content,
                        role: async_openai::types::Role::Tool,
                        tool_call_id: msg.tool_call_id.unwrap_or_default(),
                    };
                    api_messages.push(ChatCompletionRequestMessage::Tool(tool_msg));
                }
            }
        }
        api_messages
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
        _timeout: f64,
    ) -> std::result::Result<Vec<String>, ProviderError> {
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
    ) -> std::result::Result<ChatCompletionResponse, ProviderError> {
        let api_messages = self.map_messages(request.messages.clone());
        
        // Convert tools if present
        let mut api_request = CreateChatCompletionRequestArgs::default();
        api_request
            .model(&request.model)
            .messages(api_messages)
            .temperature(request.temperature.unwrap_or(0.7));
            
        if let Some(max_tokens) = request.max_tokens {
            api_request.max_tokens(max_tokens as u16);
        }
        
        if let Some(top_p) = request.top_p {
            api_request.top_p(top_p);
        }
        if let Some(stop) = request.stop {
            api_request.stop(stop);
        }
        if request.stream {
            api_request.stream(true);
        }
        
        // Handle tools - convert to async-openai format
        if let Some(tools) = request.tools {
            let api_tools: Vec<_> = tools.into_iter().map(|t| {
                async_openai::types::ChatCompletionTool {
                    r#type: async_openai::types::ChatCompletionToolType::Function,
                    function: async_openai::types::FunctionObject {
                        name: t.function.name,
                        description: t.function.description,
                        parameters: Some(t.function.parameters),
                    },
                }
            }).collect();
            api_request.tools(api_tools);
        }
        
        // Handle tool_choice
        if let Some(tool_choice) = request.tool_choice {
            let api_tool_choice = match tool_choice {
                ToolChoice::None => async_openai::types::ChatCompletionToolChoiceOption::None,
                ToolChoice::Auto => async_openai::types::ChatCompletionToolChoiceOption::Auto,
                ToolChoice::Required => async_openai::types::ChatCompletionToolChoiceOption::Auto,
                ToolChoice::Specific(name) => async_openai::types::ChatCompletionToolChoiceOption::Named(
                    async_openai::types::ChatCompletionNamedToolChoice {
                        r#type: async_openai::types::ChatCompletionToolType::Function,
                        function: async_openai::types::FunctionName {
                            name,
                        }
                    }
                ),
            };
            api_request.tool_choice(api_tool_choice);
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
            let role = match choice.message.role {
                async_openai::types::Role::System => MessageRole::System,
                async_openai::types::Role::User => MessageRole::User,
                async_openai::types::Role::Assistant => MessageRole::Assistant,
                async_openai::types::Role::Tool => MessageRole::Tool,
                async_openai::types::Role::Function => MessageRole::Tool,
            };
            
            let tool_calls = choice.message.tool_calls.map(|tcs| {
                tcs.into_iter().map(|tc| ToolCall {
                    id: tc.id,
                    r#type: match tc.r#type {
                        async_openai::types::ChatCompletionToolType::Function => "function".to_string(),
                    },
                    function: ToolFunction {
                        name: tc.function.name,
                        arguments: tc.function.arguments,
                    },
                }).collect()
            });
            
            let message = ChatMessage {
                role,
                content: choice.message.content.unwrap_or_default(),
                name: None,
                tool_calls,
                tool_call_id: None,
            };
            
            Choice {
                index: i,
                message,
                finish_reason: choice.finish_reason.map(|r| {
                    serde_json::to_value(&r)
                        .ok()
                        .and_then(|v| v.as_str().map(String::from))
                        .unwrap_or_else(|| format!("{:?}", r))
                }),
            }
        }).collect();
        
        let usage = response.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens as u64,
            completion_tokens: u.completion_tokens as u64,
            total_tokens: u.total_tokens as u64,
        });
        
        Ok(ChatCompletionResponse {
            id: response.id,
            model: response.model,
            created: response.created as u64,
            choices,
            usage,
        })
    }
    
    async fn create_chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> std::result::Result<ChatCompletionStream, ProviderError> {
        let api_messages = self.map_messages(request.messages.clone());
        
        let mut api_request = CreateChatCompletionRequestArgs::default();
        api_request
            .model(&request.model)
            .messages(api_messages)
            .temperature(request.temperature.unwrap_or(0.7))
            .stream(true);
            
        if let Some(max_tokens) = request.max_tokens {
            api_request.max_tokens(max_tokens as u16);
        }
        
        if let Some(top_p) = request.top_p {
            api_request.top_p(top_p);
        }
        if let Some(stop) = request.stop {
            api_request.stop(stop);
        }
        
        let api_request = api_request.build()
            .map_err(|e| ProviderError::ConfigurationError(e.to_string()))?;
        
        let stream = self.client.chat().create_stream(api_request).await
            .map_err(|e| ProviderError::StreamingError(e.to_string()))?;
        
        // Convert the async-openai stream to our format
        let converted_stream = stream.map(|chunk| {
            let chunk = chunk.map_err(|e| ProviderError::StreamingError(e.to_string()))?;
            
            let choices = chunk.choices.into_iter().enumerate().map(|(i, choice)| {
                let delta_content = choice.delta.content.clone().unwrap_or_default();
                
                let role = choice.delta.role.as_ref().map(|role| {
                    match role {
                        async_openai::types::Role::System => MessageRole::System,
                        async_openai::types::Role::User => MessageRole::User,
                        async_openai::types::Role::Assistant => MessageRole::Assistant,
                        async_openai::types::Role::Tool => MessageRole::Tool,
                        async_openai::types::Role::Function => MessageRole::Tool,
                    }
                });
                
                let tool_calls = choice.delta.tool_calls.as_ref().map(|tcs| {
                    tcs.iter().map(|tc| StreamToolCall {
                        id: tc.id.clone(),
                        r#type: tc.r#type.as_ref().map(|t| match t {
                            async_openai::types::ChatCompletionToolType::Function => "function".to_string(),
                        }),
                        function: tc.function.as_ref().map(|f| StreamToolFunction {
                            name: f.name.clone(),
                            arguments: f.arguments.clone(),
                        }),
                    }).collect()
                });
                
                StreamChoice {
                    index: i,
                    delta: StreamDelta {
                        role,
                        content: Some(delta_content),
                        tool_calls,
                    },
                    finish_reason: choice.finish_reason.map(|r| {
                        serde_json::to_value(&r)
                            .ok()
                            .and_then(|v| v.as_str().map(String::from))
                            .unwrap_or_else(|| format!("{:?}", r))
                    }),
                }
            }).collect();
            
            Ok(StreamChunk {
                id: chunk.id,
                model: chunk.model,
                created: Some(chunk.created as u64),
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

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn test_openai_profile() {
        let profile = openai_profile();
        assert_eq!(profile.name, "openai");
        assert_eq!(profile.api_mode, ApiMode::ChatCompletions);
        assert_eq!(profile.auth_type, AuthType::ApiKey);
    }

    #[tokio::test]
    async fn test_openai_provider_new() {
        let provider = OpenAIProvider::new(Some("test_key".to_string()), Some("http://localhost".to_string()));
        assert_eq!(provider.profile().name, "openai");
        
        let custom_profile = ProviderProfile::new("custom");
        let custom_provider = OpenAIProvider::new_with_profile(None, None, custom_profile);
        assert_eq!(custom_provider.profile().name, "custom");
    }

    #[tokio::test]
    async fn test_map_messages() {
        let provider = OpenAIProvider::new(None, None);
        let msgs = vec![
            ChatMessage {
                role: MessageRole::System,
                content: "sys".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: "usr".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: MessageRole::Assistant,
                content: "ast".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: MessageRole::Tool,
                content: "tool_res".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: Some("call_123".to_string()),
            },
        ];

        let mapped = provider.map_messages(msgs);
        assert_eq!(mapped.len(), 4);
        
        match &mapped[0] {
            ChatCompletionRequestMessage::System(sys) => {
                assert_eq!(sys.content, "sys");
            }
            _ => panic!("Expected System"),
        }

        match &mapped[1] {
            ChatCompletionRequestMessage::User(user) => {
                if let async_openai::types::ChatCompletionRequestUserMessageContent::Text(text) = &user.content {
                    assert_eq!(text, "usr");
                } else {
                    panic!("Expected Text");
                }
            }
            _ => panic!("Expected User"),
        }
        
        match &mapped[2] {
            ChatCompletionRequestMessage::Assistant(ast) => {
                assert_eq!(ast.content, Some("ast".to_string()));
            }
            _ => panic!("Expected Assistant"),
        }
        
        match &mapped[3] {
            ChatCompletionRequestMessage::Tool(tool) => {
                assert_eq!(tool.content, "tool_res");
                assert_eq!(tool.tool_call_id, "call_123");
            }
            _ => panic!("Expected Tool"),
        }
    }

    #[tokio::test]
    async fn test_fetch_models() {
        let mock_server = MockServer::start().await;
        
        let response_body = serde_json::json!({
            "object": "list",
            "data": [
                {"id": "gpt-4", "object": "model", "created": 1234, "owned_by": "openai"}
            ]
        });

        Mock::given(method("GET"))
            .and(path("/models"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(Some("test_key".to_string()), Some(mock_server.uri()));
        let models = provider.fetch_models(None, 10.0).await.unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0], "gpt-4");
    }
}

// Rust guideline compliant 2026-02-21
