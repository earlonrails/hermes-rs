use serde_json::Value;

/// This struct acts as a placeholder or builder for vision-related contexts.
/// In the actual agent implementation, vision processing (extracting image URLs or base64)
/// should be injected directly into the LLMProvider's message payloads.
pub struct VisionProcessor;

impl VisionProcessor {
    pub fn new() -> Self {
        Self
    }

    /// Formats an image URL into the proper payload for GPT-4V/Claude 3
    pub fn format_image_url(url: &str) -> Value {
        serde_json::json!({
            "type": "image_url",
            "image_url": {
                "url": url
            }
        })
    }

    /// Formats a base64 string into the proper payload
    pub fn format_base64(mime_type: &str, base64_data: &str) -> Value {
        serde_json::json!({
            "type": "image_url",
            "image_url": {
                "url": format!("data:{};base64,{}", mime_type, base64_data)
            }
        })
    }
}
