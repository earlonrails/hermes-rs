use reqwest::Client;
use tracing::info;

pub struct AudioProcessor {
    client: Client,
    openai_api_key: String,
}

impl AudioProcessor {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            openai_api_key: api_key,
        }
    }

    /// Transcribe an audio file using OpenAI Whisper API
    pub async fn transcribe(&self, file_path: &str) -> Result<String, String> {
        info!("Transcribing audio file: {}", file_path);
        
        let file_bytes = tokio::fs::read(file_path).await.map_err(|e| e.to_string())?;
        
        let part = reqwest::multipart::Part::bytes(file_bytes)
            .file_name(file_path.to_string());
            
        let form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("model", "whisper-1");

        let res = self.client.post("https://api.openai.com/v1/audio/transcriptions")
            .bearer_auth(&self.openai_api_key)
            .multipart(form)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(format!("API error: {}", res.status()));
        }

        let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
        Ok(json["text"].as_str().unwrap_or_default().to_string())
    }

    /// Generate Text-to-Speech using OpenAI TTS API
    pub async fn synthesize(&self, text: &str, output_path: &str) -> Result<(), String> {
        info!("Synthesizing audio for text to: {}", output_path);
        
        let body = serde_json::json!({
            "model": "tts-1",
            "input": text,
            "voice": "alloy"
        });

        let res = self.client.post("https://api.openai.com/v1/audio/speech")
            .bearer_auth(&self.openai_api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(format!("API error: {}", res.status()));
        }

        let bytes = res.bytes().await.map_err(|e| e.to_string())?;
        tokio::fs::write(output_path, bytes).await.map_err(|e| e.to_string())?;

        Ok(())
    }
}
