#[cfg(test)]
mod tests {
    use hermes_tools::ToolRegistry;
    
    #[test]
    fn test_tool_registry_creation() {
        let registry = ToolRegistry::new();
        assert!(registry.clone().is_some());
    }
    
    // Note: We cannot test list_tools/list_toolsets since they are not implemented in the current version
    // but we can at least test that the registry can be instantiated and used
}