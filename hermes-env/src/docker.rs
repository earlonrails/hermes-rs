use async_trait::async_trait;
use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, RemoveContainerOptions, LogOutput};
use bollard::exec::{CreateExecOptions, StartExecResults, StartExecOptions};
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;

use crate::traits::{Environment, ExecutionConfig, ExecutionResult, EnvError};

pub struct DockerEnv {
    id: String,
    image: String,
    docker: Arc<Docker>,
}

impl DockerEnv {
    pub fn new(id: impl Into<String>, image: impl Into<String>) -> Result<Self, EnvError> {
        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| EnvError::InitFailed(e.to_string()))?;
            
        Ok(Self {
            id: id.into(),
            image: image.into(),
            docker: Arc::new(docker),
        })
    }
}

#[async_trait]
impl Environment for DockerEnv {
    fn id(&self) -> &str {
        &self.id
    }
    
    async fn init(&self) -> Result<(), EnvError> {
        let options = Some(CreateContainerOptions {
            name: self.id.clone(),
            platform: None,
        });
        
        let config = Config {
            image: Some(self.image.clone()),
            tty: Some(true), // Keep container alive
            cmd: Some(vec!["tail".to_string(), "-f".to_string(), "/dev/null".to_string()]),
            ..Default::default()
        };
        
        self.docker.create_container(options, config).await
            .map_err(|e| EnvError::InitFailed(e.to_string()))?;
            
        self.docker.start_container(&self.id, None::<StartContainerOptions<String>>).await
            .map_err(|e| EnvError::InitFailed(format!("Failed to start: {}", e)))?;
            
        Ok(())
    }
    
    async fn execute(&self, command: &str, config: ExecutionConfig) -> Result<ExecutionResult, EnvError> {
        let mut env_vec = Vec::new();
        for (k, v) in config.env_vars {
            env_vec.push(format!("{}={}", k, v));
        }
        
        let exec_options = CreateExecOptions {
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            cmd: Some(vec!["sh".to_string(), "-c".to_string(), command.to_string()]),
            env: if env_vec.is_empty() { None } else { Some(env_vec) },
            working_dir: config.working_dir,
            ..Default::default()
        };
        
        let exec = self.docker.create_exec(&self.id, exec_options).await
            .map_err(|e| EnvError::ExecutionFailed(e.to_string()))?;
            
        let start_options = StartExecOptions { detach: false };
        let mut stdout_buf = String::new();
        let mut stderr_buf = String::new();
        
        match self.docker.start_exec(&exec.id, Some(start_options)).await
            .map_err(|e| EnvError::ExecutionFailed(e.to_string()))? 
        {
            StartExecResults::Attached { mut output, .. } => {
                while let Some(msg_res) = output.next().await {
                    match msg_res {
                        Ok(LogOutput::StdOut { message }) => stdout_buf.push_str(&String::from_utf8_lossy(&message)),
                        Ok(LogOutput::StdErr { message }) => stderr_buf.push_str(&String::from_utf8_lossy(&message)),
                        _ => {}
                    }
                }
            }
            StartExecResults::Detached => {
                return Err(EnvError::ExecutionFailed("Exec detached unexpectedly".to_string()));
            }
        }
        
        // Inspect exec to get exit code
        let inspect = self.docker.inspect_exec(&exec.id).await
            .map_err(|e| EnvError::ExecutionFailed(e.to_string()))?;
            
        Ok(ExecutionResult {
            exit_code: inspect.exit_code.unwrap_or(-1) as i32,
            stdout: stdout_buf,
            stderr: stderr_buf,
        })
    }
    
    async fn write_file(&self, path: &str, content: &[u8]) -> Result<(), EnvError> {
        // Implement via execute or tar stream
        let encoded = base64::encode(content);
        let cmd = format!("echo '{}' | base64 -d > {}", encoded, path);
        self.execute(&cmd, ExecutionConfig::default()).await?;
        Ok(())
    }
    
    async fn read_file(&self, path: &str) -> Result<Vec<u8>, EnvError> {
        let cmd = format!("cat {}", path);
        let res = self.execute(&cmd, ExecutionConfig::default()).await?;
        if res.exit_code != 0 {
            return Err(EnvError::NotFound(path.to_string()));
        }
        Ok(res.stdout.into_bytes())
    }
    
    async fn destroy(&self) -> Result<(), EnvError> {
        let options = Some(RemoveContainerOptions {
            v: true,
            force: true,
            link: false,
        });
        self.docker.remove_container(&self.id, options).await
            .map_err(|e| EnvError::ExecutionFailed(e.to_string()))?;
        Ok(())
    }
}
