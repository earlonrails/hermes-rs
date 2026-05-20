use crate::Message;
use tiktoken_rs::cl100k_base;

pub struct ContextEngine {
    max_tokens: usize,
}

impl ContextEngine {
    pub fn new(max_tokens: usize) -> Self {
        Self { max_tokens }
    }

    /// Counts the approximate number of tokens in a string using the cl100k_base encoding.
    pub fn count_tokens(&self, text: &str) -> usize {
        if let Ok(bpe) = cl100k_base() {
            bpe.encode_with_special_tokens(text).len()
        } else {
            // Fallback approximation if tokenizer fails to load
            text.chars().count() / 4
        }
    }

    /// Compresses a list of messages so that the total token count is under `max_tokens`.
    /// 
    /// Simple implementation: Drops the oldest user/assistant messages if we exceed the limit.
    /// In a real scenario, this would summarize or truncate specific tool outputs.
    pub fn compress(&self, messages: &[Message]) -> Vec<Message> {
        let mut total_tokens = 0;
        let mut compressed = Vec::new();

        // Iterate backwards to keep the most recent messages
        for msg in messages.iter().rev() {
            let tokens = match msg {
                Message::System { content } => self.count_tokens(content),
                Message::User { content, .. } => self.count_tokens(content),
                Message::Assistant { content, .. } => {
                    content.as_ref().map(|c| self.count_tokens(c)).unwrap_or(0)
                }
                Message::Tool { content, .. } => self.count_tokens(content),
            };

            if total_tokens + tokens <= self.max_tokens {
                total_tokens += tokens;
                compressed.push(msg.clone());
            } else {
                break;
            }
        }

        compressed.reverse();
        compressed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Message;

    #[test]
    fn test_context_engine_count_tokens() {
        let engine = ContextEngine::new(100);
        let tokens = engine.count_tokens("Hello world");
        assert!(tokens > 0);
    }

    #[test]
    fn test_context_engine_compress() {
        let engine = ContextEngine::new(20);
        let messages = vec![
            Message::System { content: "Sys".to_string() },
            // This long string will exceed 20 tokens easily
            Message::User { content: "This is a very long string that will definitely exceed twenty tokens because it has a lot of words and complex characters.".to_string(), name: None },
            Message::Assistant { content: Some("Short".to_string()), tool_calls: None, reasoning_content: None },
            Message::Tool { content: "Res".to_string(), tool_call_id: "call_1".to_string() }
        ];

        let compressed = engine.compress(&messages);
        
        // It iterates backwards. Tool and Assistant will fit.
        // User message is too big and will break the loop, meaning System and User are dropped.
        assert_eq!(compressed.len(), 2);
        
        match &compressed[0] {
            Message::Assistant { content, .. } => assert_eq!(content.as_deref(), Some("Short")),
            _ => panic!("Expected Assistant message"),
        }
    }
}

// Rust guideline compliant 2026-02-21
