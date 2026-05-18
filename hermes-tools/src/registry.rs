use async_trait::async_trait;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

/// Trait representing a Tool in the Hermes Agent.
#[async_trait]
pub trait Tool: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn toolset(&self) -> &'static str;
    fn schema(&self) -> Value;
    
    // Default implementations for optional properties
    fn description(&self) -> String {
        self.schema()
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("")
            .to_string()
    }
    fn emoji(&self) -> &'static str { "⚡" }
    fn max_result_size_chars(&self) -> Option<usize> { None }
    fn requires_env(&self) -> Vec<&'static str> { vec![] }
    
    /// Availability check function.
    fn check_fn(&self) -> bool { true }
    
    /// Optional dynamic schema overrides.
    fn dynamic_schema_overrides(&self) -> Option<Value> { None }

    /// The core execution handler.
    async fn handle(&self, args: Value) -> Result<Value, String>;
}

#[derive(Clone)]
pub struct RegisteredTool {
    pub factory: fn() -> Arc<dyn Tool>,
}

// We use inventory to allow decentralized registration of tools across crates.
inventory::collect!(RegisteredTool);

#[derive(Clone)]
pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
    toolset_aliases: Arc<RwLock<HashMap<String, String>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut tools = HashMap::new();
        // Automatically populate from the inventory
        for registered in inventory::iter::<RegisteredTool> {
            let tool = (registered.factory)();
            tools.insert(tool.name().to_string(), tool.clone());
        }

        Self {
            tools: Arc::new(RwLock::new(tools)),
            toolset_aliases: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_all_tools(&self) -> Vec<Arc<dyn Tool>> {
        let r = self.tools.read().await;
        r.values().cloned().collect()
    }

    pub async fn register(&self, tool: Arc<dyn Tool>) {
        let mut w = self.tools.write().await;
        let name = tool.name().to_string();
        
        if let Some(existing) = w.get(&name) {
            if existing.toolset() != tool.toolset() {
                error!(
                    "Tool registration REJECTED: '{}' (toolset '{}') would shadow existing tool from toolset '{}'.",
                    name, tool.toolset(), existing.toolset()
                );
                return;
            }
        }
        w.insert(name, tool);
    }

    pub async fn deregister(&self, name: &str) {
        let mut w = self.tools.write().await;
        if w.remove(name).is_some() {
            debug!("Deregistered tool: {}", name);
        }
    }

    pub async fn get_definitions(&self, tool_names: &HashSet<String>, quiet: bool) -> Vec<Value> {
        let r = self.tools.read().await;
        let mut result = Vec::new();

        let mut sorted_names: Vec<_> = tool_names.iter().collect();
        sorted_names.sort();

        for name in sorted_names {
            if let Some(tool) = r.get(name) {
                if !tool.check_fn() {
                    if !quiet {
                        debug!("Tool {} unavailable (check failed)", name);
                    }
                    continue;
                }

                let mut schema = tool.schema();
                if let Some(overrides) = tool.dynamic_schema_overrides() {
                    if let (Some(base_obj), Some(override_obj)) = (schema.as_object_mut(), overrides.as_object()) {
                        for (k, v) in override_obj {
                            base_obj.insert(k.clone(), v.clone());
                        }
                    }
                }

                // Ensure name is present
                if let Some(obj) = schema.as_object_mut() {
                    obj.insert("name".to_string(), Value::String(tool.name().to_string()));
                }

                result.push(serde_json::json!({
                    "type": "function",
                    "function": schema
                }));
            }
        }
        result
    }

    pub async fn dispatch(&self, name: &str, args: Value) -> String {
        let tool = {
            let r = self.tools.read().await;
            r.get(name).cloned()
        };

        if let Some(tool) = tool {
            match tool.handle(args).await {
                Ok(res) => res.to_string(),
                Err(err) => serde_json::json!({ "error": format!("Tool execution failed: {}", err) }).to_string(),
            }
        } else {
            serde_json::json!({ "error": format!("Unknown tool: {}", name) }).to_string()
        }
    }

    pub async fn register_toolset_alias(&self, alias: String, toolset: String) {
        let mut w = self.toolset_aliases.write().await;
        if let Some(existing) = w.get(&alias) {
            if existing != &toolset {
                warn!("Toolset alias collision: '{}' ({}) overwritten by {}", alias, existing, toolset);
            }
        }
        w.insert(alias, toolset);
    }
}

pub fn tool_error(message: &str) -> String {
    serde_json::json!({ "error": message }).to_string()
}

pub fn tool_result(data: Value) -> String {
    data.to_string()
}
