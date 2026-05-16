use async_trait::async_trait;
use russh::{client, ChannelMsg};
use russh_keys::key::KeyPair;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::traits::{Environment, ExecutionConfig, ExecutionResult, EnvError};

struct ClientHandler;

#[async_trait]
impl client::Handler for ClientHandler {
    type Error = russh::Error;
}

pub struct SshEnv {
    id: String,
    host: String,
    port: u16,
    user: String,
    key_path: String,
    session: Mutex<Option<client::Handle<ClientHandler>>>,
}

impl SshEnv {
    pub fn new(id: impl Into<String>, host: impl Into<String>, user: impl Into<String>, key_path: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            host: host.into(),
            port: 22,
            user: user.into(),
            key_path: key_path.into(),
            session: Mutex::new(None),
        }
    }
}

#[async_trait]
impl Environment for SshEnv {
    fn id(&self) -> &str {
        &self.id
    }
    
    async fn init(&self) -> Result<(), EnvError> {
        let config = client::Config {
            inactivity_timeout: Some(std::time::Duration::from_secs(30)),
            ..Default::default()
        };
        let config = Arc::new(config);
        
        let mut handle = client::connect(config, (self.host.as_str(), self.port), ClientHandler).await
            .map_err(|e| EnvError::InitFailed(e.to_string()))?;
            
        let key_pair = russh_keys::load_secret_key(&self.key_path, None)
            .map_err(|e| EnvError::InitFailed(e.to_string()))?;
            
        let auth_res = handle.authenticate_publickey(&self.user, Arc::new(key_pair)).await
            .map_err(|e| EnvError::InitFailed(e.to_string()))?;
            
        if !auth_res {
            return Err(EnvError::InitFailed("SSH Auth failed".to_string()));
        }
        
        *self.session.lock().await = Some(handle);
        Ok(())
    }
    
    async fn execute(&self, command: &str, _config: ExecutionConfig) -> Result<ExecutionResult, EnvError> {
        let mut session_guard = self.session.lock().await;
        let handle = match session_guard.as_mut() {
            Some(h) => h,
            None => return Err(EnvError::ExecutionFailed("Not connected".to_string())),
        };
        let mut channel = handle.channel_open_session().await
            .map_err(|e| EnvError::ExecutionFailed(e.to_string()))?;
            
        channel.exec(true, command).await
            .map_err(|e| EnvError::ExecutionFailed(e.to_string()))?;
            
        let mut stdout_buf = String::new();
        let mut stderr_buf = String::new();
        let mut exit_code = -1;
        
        while let Some(msg) = channel.wait().await {
            match msg {
                ChannelMsg::Data { ref data } => stdout_buf.push_str(&String::from_utf8_lossy(data)),
                ChannelMsg::ExtendedData { ref data, ext } if ext == 1 => stderr_buf.push_str(&String::from_utf8_lossy(data)),
                ChannelMsg::ExitStatus { exit_status } => exit_code = exit_status as i32,
                _ => {}
            }
        }
        
        Ok(ExecutionResult {
            exit_code,
            stdout: stdout_buf,
            stderr: stderr_buf,
        })
    }
    
    async fn write_file(&self, _path: &str, _content: &[u8]) -> Result<(), EnvError> {
        Err(EnvError::ExecutionFailed("Not implemented for SSH".to_string()))
    }
    
    async fn read_file(&self, _path: &str) -> Result<Vec<u8>, EnvError> {
        Err(EnvError::ExecutionFailed("Not implemented for SSH".to_string()))
    }
    
    async fn destroy(&self) -> Result<(), EnvError> {
        let mut session_guard = self.session.lock().await;
        if let Some(handle) = session_guard.take() {
            let _ = handle.disconnect(russh::Disconnect::ByApplication, "Done", "en").await;
        }
        Ok(())
    }
}
