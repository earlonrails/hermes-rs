use async_trait::async_trait;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

/// Trait representing a Tool in the Athena Agent.
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct DummyTool;

    #[async_trait]
    impl Tool for DummyTool {
        fn name(&self) -> &'static str {
            "dummy"
        }
        fn toolset(&self) -> &'static str {
            "test"
        }
        fn schema(&self) -> Value {
            json!({"description": "dummy test tool"})
        }
        async fn handle(&self, _args: Value) -> Result<Value, String> {
            Ok(json!({"status": "ok"}))
        }
    }

    struct FailingTool;

    #[async_trait]
    impl Tool for FailingTool {
        fn name(&self) -> &'static str {
            "failing"
        }
        fn toolset(&self) -> &'static str {
            "test"
        }
        fn schema(&self) -> Value {
            json!({})
        }
        async fn handle(&self, _args: Value) -> Result<Value, String> {
            Err("failed".to_string())
        }
    }

    struct OverrideTool;

    #[async_trait]
    impl Tool for OverrideTool {
        fn name(&self) -> &'static str {
            "override"
        }
        fn toolset(&self) -> &'static str {
            "test"
        }
        fn schema(&self) -> Value {
            json!({"base": 1})
        }
        fn dynamic_schema_overrides(&self) -> Option<Value> {
            Some(json!({"overridden": 2}))
        }
        async fn handle(&self, _args: Value) -> Result<Value, String> {
            Ok(json!(true))
        }
    }

    #[tokio::test]
    async fn test_tool_defaults() {
        let tool = DummyTool;
        assert_eq!(tool.emoji(), "⚡");
        assert_eq!(tool.max_result_size_chars(), None);
        assert!(tool.requires_env().is_empty());
        assert!(tool.check_fn());
        assert_eq!(tool.dynamic_schema_overrides(), None);
        assert_eq!(tool.description(), "dummy test tool");
        assert_eq!(tool.toolset(), "test");
        assert!(tool.handle(json!({})).await.is_ok());

        let failing = FailingTool;
        assert_eq!(failing.name(), "failing");
        assert_eq!(failing.toolset(), "test");
        assert!(failing.schema().is_object());

        let override_t = OverrideTool;
        assert_eq!(override_t.name(), "override");
        assert_eq!(override_t.toolset(), "test");
        assert!(override_t.handle(json!({})).await.is_ok());
    }

    #[tokio::test]
    async fn test_registry_basics() {
        let registry = ToolRegistry::new();
        let tool = Arc::new(DummyTool);

        // Register
        registry.register(tool.clone()).await;
        let tools = registry.get_all_tools().await;
        assert!(tools.iter().any(|t| t.name() == "dummy"));

        // Dispatch success
        let res = registry.dispatch("dummy", json!({})).await;
        assert_eq!(res, json!({"status": "ok"}).to_string());

        // Dispatch unknown
        let res = registry.dispatch("unknown", json!({})).await;
        assert!(res.contains("Unknown tool"));

        // Deregister
        registry.deregister("dummy").await;
        let tools = registry.get_all_tools().await;
        assert!(!tools.iter().any(|t| t.name() == "dummy"));
    }

    #[tokio::test]
    async fn test_registry_dispatch_failure() {
        let registry = ToolRegistry::new();
        let tool = Arc::new(FailingTool);
        registry.register(tool).await;

        let res = registry.dispatch("failing", json!({})).await;
        assert!(res.contains("Tool execution failed: failed"));
    }

    struct UnavailTool;
    #[async_trait]
    impl Tool for UnavailTool {
        fn name(&self) -> &'static str { "unavail" }
        fn toolset(&self) -> &'static str { "test" }
        fn schema(&self) -> Value { json!({}) }
        fn check_fn(&self) -> bool { false }
        async fn handle(&self, _args: Value) -> Result<Value, String> { Ok(json!(1)) }
    }

    struct NonObjectOverrideTool;
    #[async_trait]
    impl Tool for NonObjectOverrideTool {
        fn name(&self) -> &'static str { "nonobj_override" }
        fn toolset(&self) -> &'static str { "test" }
        fn schema(&self) -> Value { json!("not an object") }
        fn dynamic_schema_overrides(&self) -> Option<Value> {
            Some(json!("not an object"))
        }
        async fn handle(&self, _args: Value) -> Result<Value, String> { Ok(json!(1)) }
    }

    #[tokio::test]
    async fn test_registry_get_definitions() {
        let registry = ToolRegistry::new();
        let tool1 = Arc::new(DummyTool);
        let tool2 = Arc::new(OverrideTool);
        let tool3 = Arc::new(UnavailTool);
        let tool4 = Arc::new(NonObjectOverrideTool);
        registry.register(tool1).await;
        registry.register(tool2).await;
        registry.register(tool3).await;
        registry.register(tool4).await;

        let mut names = HashSet::new();
        names.insert("dummy".to_string());
        names.insert("override".to_string());
        names.insert("unavail".to_string());
        names.insert("nonobj_override".to_string());

        let defs = registry.get_definitions(&names, false).await;
        assert_eq!(defs.len(), 3); // UnavailTool is skipped

        // Check override took effect
        let override_def = defs.iter().find(|d| d["function"]["name"] == "override").unwrap();
        assert_eq!(override_def["function"]["overridden"], 2);
    }

    #[tokio::test]
    async fn test_registry_register_toolset_alias() {
        let registry = ToolRegistry::new();
        registry.register_toolset_alias("alias1".to_string(), "toolset1".to_string()).await;
        // Test collision warning coverage
        registry.register_toolset_alias("alias1".to_string(), "toolset2".to_string()).await;

        let aliases = registry.toolset_aliases.read().await;
        assert_eq!(aliases.get("alias1").unwrap(), "toolset2");
    }

    #[tokio::test]
    async fn test_registry_register_collision() {
        let registry = ToolRegistry::new();
        let tool1 = Arc::new(DummyTool);

        struct ShadowTool;
        #[async_trait]
        impl Tool for ShadowTool {
            fn name(&self) -> &'static str { "dummy" }
            fn toolset(&self) -> &'static str { "other_test" }
            fn schema(&self) -> Value { json!({}) }
            async fn handle(&self, _args: Value) -> Result<Value, String> { Ok(json!(1)) }
        }

        registry.register(tool1).await;
        // This should be rejected
        let shadow = Arc::new(ShadowTool);
        registry.register(shadow.clone()).await;

        assert!(shadow.schema().is_object());
        assert!(shadow.handle(json!({})).await.is_ok());

        // Ensure dummy is still test toolset
        let defs = registry.get_all_tools().await;
        let t = defs.iter().find(|t| t.name() == "dummy").unwrap();
        assert_eq!(t.toolset(), "test");
    }

    #[test]
    fn test_helpers() {
        assert_eq!(tool_error("test"), json!({"error": "test"}).to_string());
        assert_eq!(tool_result(json!(1)), "1");
    }

    #[tokio::test]
    async fn test_tool_registry_all_tools() {
        let registry = ToolRegistry::new();
        let all_tools = registry.get_all_tools().await;
        // Should have at least a few tools loaded via inventory
        assert!(!all_tools.is_empty());
    }

    #[tokio::test]
    async fn test_tool_descriptions() {
        let registry = ToolRegistry::new();
        let all_tools = registry.get_all_tools().await;

        for tool in all_tools {
            let description = tool.description();
            assert!(!description.is_empty(), "Tool {} should have a description", tool.name());

            let emoji = tool.emoji();
            assert!(!emoji.is_empty(), "Tool {} should have an emoji", tool.name());
        }
    }

    #[tokio::test]
    async fn test_tool_schemas() {
        let registry = ToolRegistry::new();
        let all_tools = registry.get_all_tools().await;

        for tool in all_tools {
            let schema = tool.schema();
            assert!(schema.get("description").is_some(), "Tool {} should have a description in schema", tool.name());
            assert!(schema.get("parameters").is_some(), "Tool {} should have parameters in schema", tool.name());
        }
    }
}

// Rust guideline compliant 2026-02-21
