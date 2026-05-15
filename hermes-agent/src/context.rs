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
