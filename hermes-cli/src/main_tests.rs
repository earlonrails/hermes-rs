#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_args_parsing() {
        // Test that CLI arguments parse correctly
        let args = Args::parse_from(["hermes", "--help"]);
        // Just testing that parsing doesn't panic
        assert!(args.help);
    }
    
    #[test]
    fn test_model_arg_parsing() {
        let args = Args::parse_from(["hermes", "--model", "gpt-4-turbo"]);
        assert_eq!(args.model, Some("gpt-4-turbo".to_string()));
    }
    
    #[test]
    fn test_toolsets_arg_parsing() {
        let args = Args::parse_from(["hermes", "--toolsets", "web,terminal"]);
        assert_eq!(args.toolsets, Some("web,terminal".to_string()));
    }
    
    #[test]
    fn test_skills_arg_parsing() {
        let args = Args::parse_from(["hermes", "--skills", "github-auth,dev-tools"]);
        assert_eq!(args.skills, Some("github-auth,dev-tools".to_string()));
    }
    
    #[test]
    fn test_max_turns_arg_parsing() {
        let args = Args::parse_from(["hermes", "--max-turns", "30"]);
        assert_eq!(args.max_turns, Some(30));
    }
    
    #[test]
    fn test_single_query_arg_parsing() {
        let args = Args::parse_from(["hermes", "-q", "hello world"]);
        assert_eq!(args.query, Some("hello world".to_string()));
    }
}