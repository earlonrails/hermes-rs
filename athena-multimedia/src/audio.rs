use reqwest::Client;
use tracing::info;

pub struct AudioProcessor {
    client: Client,
    openai_api_key: String,
    endpoint: String,
}

impl AudioProcessor {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            openai_api_key: api_key,
            endpoint: "https://api.openai.com/v1".to_string(),
        }
    }

    pub fn with_endpoint(mut self, endpoint: String) -> Self {
        self.endpoint = endpoint;
        self
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

        let res = self.client.post(&format!("{}/audio/transcriptions", self.endpoint))
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

        let res = self.client.post(&format!("{}/audio/speech", self.endpoint))
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

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, header};
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use std::env;

    #[tokio::test]
    async fn test_audio_processor_transcribe_success() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/audio/transcriptions"))
            .and(header("authorization", "Bearer test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "text": "Hello world"
            })))
            .mount(&mock_server)
            .await;

        let processor = AudioProcessor::new("test-key".to_string())
            .with_endpoint(mock_server.uri());

        let temp_file = env::temp_dir().join(format!("test_audio_{}.wav", uuid::Uuid::new_v4()));
        tokio::fs::write(&temp_file, b"dummy audio data").await.unwrap();

        let result = processor.transcribe(temp_file.to_str().unwrap()).await.unwrap();
        assert_eq!(result, "Hello world");

        let _ = tokio::fs::remove_file(&temp_file).await;
    }

    #[tokio::test]
    async fn test_audio_processor_transcribe_failure_missing_file() {
        let processor = AudioProcessor::new("test-key".to_string());
        let result = processor.transcribe("/path/does/not/exist.wav").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_audio_processor_transcribe_api_error() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/audio/transcriptions"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let processor = AudioProcessor::new("test-key".to_string())
            .with_endpoint(mock_server.uri());

        let temp_file = env::temp_dir().join(format!("test_audio_err_{}.wav", uuid::Uuid::new_v4()));
        tokio::fs::write(&temp_file, b"dummy audio data").await.unwrap();

        let result = processor.transcribe(temp_file.to_str().unwrap()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API error: 500"));

        let _ = tokio::fs::remove_file(&temp_file).await;
    }

    #[tokio::test]
    async fn test_audio_processor_synthesize_success() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/audio/speech"))
            .and(header("authorization", "Bearer test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"audio content".to_vec()))
            .mount(&mock_server)
            .await;

        let processor = AudioProcessor::new("test-key".to_string())
            .with_endpoint(mock_server.uri());

        let temp_file = env::temp_dir().join(format!("test_speech_{}.wav", uuid::Uuid::new_v4()));

        processor.synthesize("Hello", temp_file.to_str().unwrap()).await.unwrap();
        
        let contents = tokio::fs::read(&temp_file).await.unwrap();
        assert_eq!(contents, b"audio content");

        let _ = tokio::fs::remove_file(&temp_file).await;
    }

    #[tokio::test]
    async fn test_audio_processor_synthesize_api_error() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/audio/speech"))
            .respond_with(ResponseTemplate::new(400))
            .mount(&mock_server)
            .await;

        let processor = AudioProcessor::new("test-key".to_string())
            .with_endpoint(mock_server.uri());

        let temp_file = env::temp_dir().join(format!("test_speech_err_{}.wav", uuid::Uuid::new_v4()));

        let result = processor.synthesize("Hello", temp_file.to_str().unwrap()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API error: 400"));
    }
}

// Rust guideline compliant 2026-02-21
