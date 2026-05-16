#[cfg(test)]
mod tests {
    use hermes_agent::{AIAgent, AIAgentBuilder};
    
    #[test]
    fn test_agent_builder() {
        let builder = AIAgent::builder();
        assert!(builder.model("gpt-4o").model == "gpt-4o");
    }
    
    #[test]
    fn test_agent_builder_with_api_key() {
        let builder = AIAgent::builder().api_key("test-key");
        assert_eq!(builder.api_key, Some("test-key".to_string()));
    }
    
    #[test]
    fn test_agent_builder_with_max_iterations() {
        let builder = AIAgent::builder().max_iterations(10);
        assert_eq!(builder.max_iterations, 10);
    }
}