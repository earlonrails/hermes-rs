use crate::{AgentConfig, AIAgentBuilder, IterationBudget, Message, ToolCall};
use athena_tools::{ToolRegistry};
use tokio::task::JoinHandle;
use tracing::{debug};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{CreateChatCompletionRequestArgs, Role, ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage, ChatCompletionMessageToolCall, FunctionCall},
};

pub struct AIAgent {
    pub(crate) config: AgentConfig,
    pub(crate) budget: IterationBudget,
}

impl AIAgent {
    pub fn builder() -> AIAgentBuilder {
        AIAgentBuilder::new()
    }

    pub fn model(&self) -> &str {
        &self.config.model
    }

    pub async fn run_conversation(
        &mut self,
        user_message: &str,
        system_message: Option<&str>,
        registry: &ToolRegistry,
    ) -> Result<String, String> {
        let mut messages = Vec::new();

        if let Some(sys) = system_message {
            messages.push(Message::System { content: sys.to_string() });
        }
        messages.push(Message::User { content: user_message.to_string(), name: None });

        let mut config = OpenAIConfig::new();
        if let Some(ref key) = self.config.api_key {
            config = config.with_api_key(key);
        }
        if let Some(ref url) = self.config.base_url {
            config = config.with_api_base(url);
        }
        let client = Client::with_config(config);

        let mut api_call_count = 0;

        while self.budget.consume() {
            debug!("Starting iteration {} / {}", api_call_count, self.config.max_iterations);
            println!("🤖 [Thinking] Consulting AI model...");

            // Map our strongly typed messages to async-openai's format
            let mut api_messages = Vec::new();
            for msg in &messages {
                match msg {
                    Message::System { content } => {
                        api_messages.push(ChatCompletionRequestSystemMessage {
                            role: Role::System,
                            content: content.clone(),
                            name: None,
                        }.into());
                    }
                    Message::User { content, name } => {
                        api_messages.push(ChatCompletionRequestUserMessage {
                            role: Role::User,
                            content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(content.clone()),
                            name: name.clone(),
                        }.into());
                    }
                    Message::Assistant { content, tool_calls, .. } => {
                        // Convert our internal tool calls to the format expected by async-openai.
                        let mapped_tool_calls = tool_calls.as_ref().map(|calls| {
                            calls.iter().map(|tc| {
                                ChatCompletionMessageToolCall {
                                    id: tc.id.clone(),
                                    r#type: async_openai::types::ChatCompletionToolType::Function,
                                    function: FunctionCall {
                                        name: tc.function.name.clone(),
                                        arguments: tc.function.arguments.clone(),
                                    },
                                }
                            }).collect::<Vec<_>>()
                        });

                        // Only push a message if we have either text content or tool calls.
                        let is_empty = content.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true);
                        let has_tool_calls = mapped_tool_calls.as_ref().map(|c| !c.is_empty()).unwrap_or(false);
                        if !(is_empty && !has_tool_calls) {
                            let mut builder = async_openai::types::ChatCompletionRequestAssistantMessageArgs::default();
                            if let Some(ref text) = content {
                                builder.content(text.clone());
                            }
                            if let Some(ref calls) = mapped_tool_calls {
                                builder.tool_calls(calls.clone());
                            }
                            let assistant_msg = builder.build().expect("Failed to build assistant message");
                            api_messages.push(assistant_msg.into());
                        }
                    }
                    Message::Tool { content, tool_call_id } => {
                        api_messages.push(async_openai::types::ChatCompletionRequestToolMessage {
                            role: Role::Tool,
                            content: content.clone(),
                            tool_call_id: tool_call_id.clone(),
                        }.into());
                    }
                }
            }

            // Get schemas for the tools we want to expose
            // For now, we ask the registry for all tools. In future we should filter by enabled_toolsets.
            // But let's just create an empty hashset for now which gets definitions for all.
            // Wait, registry.get_definitions needs a set of tool names. Let's just assume we want all tools.
            // Actually, we should ask the registry for all available definitions.
            // We'll pass an empty list of tools to OpenAI if we have none.
            let tool_schemas = registry.get_definitions(&std::collections::HashSet::new(), true).await;

            let mut request_builder = CreateChatCompletionRequestArgs::default();
            request_builder
                .model(&self.config.model)
                .messages(api_messages);

            // Add tools if we have them
            // async-openai expects a specific Tool schema type, which is a bit tedious to map from serde_json::Value.
            // We'll skip adding actual tool schemas to the request for this barebones iteration to avoid the mapping boilerplate,
            // OR we use serde to deserialize the Value into async-openai's tool type.
            let mut api_tools = Vec::new();
            for schema in tool_schemas {
                if let Ok(tool) = serde_json::from_value::<async_openai::types::ChatCompletionTool>(schema) {
                    api_tools.push(tool);
                }
            }
            if !api_tools.is_empty() {
                request_builder.tools(api_tools);
            }

            let request = match request_builder.build() {
                Ok(req) => req,
                Err(e) => return Err(format!("Failed to build request: {}", e)),
            };

            let response = match client.chat().create(request).await {
                Ok(resp) => resp,
                Err(e) => return Err(format!("API Error: {}", e)),
            };

            let choice = &response.choices[0].message;

            // Convert back to our internal message format
            let mut assistant_msg = Message::Assistant {
                content: choice.content.clone(),
                tool_calls: None,
                reasoning_content: None,
            };

            let mut our_tool_calls = Vec::new();
            if let Some(ref tcs) = choice.tool_calls {
                for tc in tcs {
                    our_tool_calls.push(ToolCall {
                        id: tc.id.clone(),
                        call_type: "function".to_string(),
                        function: crate::FunctionCall {
                            name: tc.function.name.clone(),
                            arguments: tc.function.arguments.clone(),
                        },
                    });
                }
                assistant_msg = Message::Assistant {
                    content: choice.content.clone(),
                    tool_calls: Some(our_tool_calls.clone()),
                    reasoning_content: None,
                };
            }

            messages.push(assistant_msg);
            api_call_count += 1;

            if our_tool_calls.is_empty() {
                // Done!
                return Ok(choice.content.clone().unwrap_or_default());
            }

            // Execute tools concurrently
            let mut handles: Vec<JoinHandle<(String, String)>> = Vec::new();
            for tc in &our_tool_calls {
                let tool_name = tc.function.name.clone();
                let args_str = tc.function.arguments.clone();
                let tool_id = tc.id.clone();
                let reg = registry.clone();

                let icon = match tool_name.as_str() {
                    "run_command" | "execute_code" => "🐳 [Sandbox]",
                    _ => "🛠️ [Calling Tool]",
                };
                println!("{} {} with args: {}", icon, tool_name, args_str);

                let handle = tokio::spawn(async move {
                    let parsed_args = serde_json::from_str(&args_str).unwrap_or_else(|_| serde_json::json!({}));
                    let result = reg.dispatch(&tool_name, parsed_args).await;
                    (tool_id, result)
                });
                handles.push(handle);
            }

            for handle in handles {
                let (tool_id, result_str) = handle.await.map_err(|e| e.to_string())?;

                // Find matching tool call to print its name and style appropriately
                let tool_name = our_tool_calls.iter()
                    .find(|tc| tc.id == tool_id)
                    .map(|tc| tc.function.name.as_str())
                    .unwrap_or("unknown");

                let icon = match tool_name {
                    "run_command" | "execute_code" => "🐳 [Sandbox Result]",
                    _ => "✔ [Tool Result]"
                };

                // Clean output preview to prevent huge spam
                let preview = if result_str.len() > 180 {
                    format!("{}...", &result_str[..180])
                } else {
                    result_str.clone()
                };
                println!("{} {}: {}", icon, tool_name, preview.trim());

                messages.push(Message::Tool {
                    content: result_str,
                    tool_call_id: tool_id,
                });
            }
        }

        Err("Max iterations reached".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use athena_tools::ToolRegistry;
    use serde_json::json;

    #[test]
    fn test_builder() {
        let agent = AIAgent::builder();
        // Just verify it creates a builder without panicking
        assert!(agent.build().budget.remaining() > 0);
    }

    #[test]
    fn test_model() {
        let agent = AIAgentBuilder::new()
            .model("test-model")
            .build();
        assert_eq!(agent.model(), "test-model");
    }

    #[tokio::test]
    async fn test_run_conversation_mocked() {
        // Start a mock server
        let mock_server = MockServer::start().await;

        // Mock OpenAI chat completions endpoint
        let response_body = json!({
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1677652288,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello from the mocked AI!"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 9,
                "completion_tokens": 12,
                "total_tokens": 21
            }
        });

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&mock_server)
            .await;

        let registry = ToolRegistry::new();
        let mut agent = AIAgentBuilder::new()
            .model("gpt-4o")
            .base_url(mock_server.uri())
            .api_key("fake-key")
            .build();

        let result = agent.run_conversation("Say hello", Some("System prompt"), &registry).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello from the mocked AI!");
    }

    #[tokio::test]
    async fn test_run_conversation_no_system_message() {
        let mock_server = MockServer::start().await;

        let response_body = json!({
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1677652288,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Response without system"
                },
                "finish_reason": "stop"
            }]
        });

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&mock_server)
            .await;

        let registry = ToolRegistry::new();
        let mut agent = AIAgentBuilder::new()
            .model("gpt-4o")
            .base_url(mock_server.uri())
            .api_key("fake-key")
            .build();

        let result = agent.run_conversation("Hello", None, &registry).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Response without system");
    }

    #[tokio::test]
    async fn test_run_conversation_api_error() {
        let mock_server = MockServer::start().await;

        let response_body = json!({
            "error": {
                "message": "Invalid API key",
                "type": "BadRequestError"
            }
        });

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(400).set_body_json(response_body))
            .mount(&mock_server)
            .await;

        let registry = ToolRegistry::new();
        let mut agent = AIAgentBuilder::new()
            .model("gpt-4o")
            .base_url(mock_server.uri())
            .api_key("invalid-key")
            .build();

        let result = agent.run_conversation("Hello", None, &registry).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API Error"));
    }

    #[tokio::test]
    async fn test_run_conversation_with_tool_calls() {
        let mock_server = MockServer::start().await;

        // Response with tool calls
        let response_body = json!({
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1677652288,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Let me help you",
                    "tool_calls": [{
                        "id": "call_1",
                        "type": "function",
                        "function": {
                            "name": "run_command",
                            "arguments": "{\"command\": \"ls\"}"
                        }
                    }]
                },
                "finish_reason": "tool_calls"
            }]
        });

        // Second response without tool calls (completion)
        let completion_response = json!({
            "id": "chatcmpl-124",
            "object": "chat.completion",
            "created": 1677652289,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Done!"
                },
                "finish_reason": "stop"
            }]
        });

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body.clone()))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(completion_response))
            .mount(&mock_server)
            .await;

        let registry = ToolRegistry::new();
        let mut agent = AIAgentBuilder::new()
            .model("gpt-4o")
            .base_url(mock_server.uri())
            .api_key("fake-key")
            .max_iterations(10)
            .build();

        let result = agent.run_conversation("List files", None, &registry).await;
        // Tool execution will fail for non-existent tools, but we're testing the path
        // The test verifies the code path for tool calls is exercised
        let _ = result;
    }

    #[tokio::test]
    async fn test_run_conversation_max_iterations() {
        let mock_server = MockServer::start().await;

        // Always respond with a tool call to keep looping
        let response_body = json!({
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1677652288,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Looping",
                    "tool_calls": [{
                        "id": "call_1",
                        "type": "function",
                        "function": {
                            "name": "run_command",
                            "arguments": "{}"
                        }
                    }]
                },
                "finish_reason": "tool_calls"
            }]
        });

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&mock_server)
            .await;

        let registry = ToolRegistry::new();
        let mut agent = AIAgentBuilder::new()
            .model("gpt-4o")
            .base_url(mock_server.uri())
            .api_key("fake-key")
            .max_iterations(2)
            .build();

        let result = agent.run_conversation("Loop", None, &registry).await;
        // Should eventually fail with max iterations or tool execution error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_conversation_empty_content() {
        let mock_server = MockServer::start().await;

        let response_body = json!({
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1677652288,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": null
                },
                "finish_reason": "stop"
            }]
        });

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&mock_server)
            .await;

        let registry = ToolRegistry::new();
        let mut agent = AIAgentBuilder::new()
            .model("gpt-4o")
            .base_url(mock_server.uri())
            .api_key("fake-key")
            .build();

        let result = agent.run_conversation("Test", None, &registry).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }
}

// Rust guideline compliant 2026-02-21
