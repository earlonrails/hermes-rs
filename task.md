# Phase 1: Foundation & State Management (Completed)
- `[x]` Initialize the Cargo workspace.
    - `[x]` Create root `Cargo.toml`
    - `[x]` Create `hermes-core` crate
    - `[x]` Create `hermes-state` crate
- `[x]` Port `hermes_constants.py` to `hermes-core/src/paths.rs`
- `[x]` Port configuration data structures (`hermes_cli/config.py`) to `hermes-core/src/config.rs`
- `[x]` Port `hermes_state.py` (SessionDB) to `hermes-state/src/db.rs` using `rusqlite`
- `[x]` Port `hermes_logging.py` using `tracing` (can be deferred or done in `hermes-core`)

# Phase 2: Core Agent & Tool Registry
- `[x]` Create `hermes-tools` crate
- `[x]` Port `tools/registry.py` to define the Tool Trait and macro system
- `[x]` Port foundational LLM loop from `run_agent.py`
    - `[x]` Add `hermes-agent` to workspace `Cargo.toml`
    - `[x]` Create `hermes-agent/Cargo.toml` and `src/lib.rs`
    - `[x]` Implement `budget.rs` (IterationBudget)
    - `[x]` Implement `messages.rs` (Strongly-typed LLM messages)
    - `[x]` Implement `config.rs` and `builder.rs` (AIAgentBuilder)
    - `[x]` Implement `agent.rs` (AIAgent core loop)

# Phase 3: Core Tool Implementations
- `[x]` File Tools (`hermes-tools/src/file_tools.rs`)
    - `[x]` Implement `read_file`
    - `[x]` Implement `write_file`
    - `[x]` Implement `list_dir`
    - `[x]` Implement `search_files`
- `[x]` Patch Tool (`hermes-tools/src/patch_tool.rs`)
    - `[x]` Implement basic fuzzy-matching patch application
- `[x]` Terminal Tool (`hermes-tools/src/terminal_tool.rs`)
    - `[x]` Implement `run_command` with timeout and output capture
- `[x]` Web Tools (`hermes-tools/src/web_tools.rs`)
    - `[x]` Implement `web_search` and `read_url`

# Phase 4: CLI Frontend & Gateway
- `[x]` Create `hermes-cli` crate
- `[x]` Implement CLI entry point and setup logging/agent builder
- `[x]` Implement persistent chat loop (`interactive.rs`)
- `[x]` Create `hermes-gateway` crate for messaging platforms
    - `[x]` Implement Telegram Bot platform integration

# Phase 5: Advanced Parity
- `[x]` Context Engine (`hermes-agent/src/context.rs`)
    - `[x]` Implement token counting logic
    - `[x]` Implement message truncation and compression
- `[x]` Code Execution Tool (`hermes-tools/src/code_tool.rs`)
    - `[x]` Implement basic AST evaluation or script runner
- `[x]` TUI Gateway (`hermes-tui-gateway`)
    - `[x]` Create `hermes-tui-gateway` crate
    - `[x]` Implement JSON-RPC server over stdio

# Phase 6: Provider Parity
- `[ ]` Implement robust LLM provider traits
- `[ ]` Port OpenAI, Anthropic, Gemini, OpenRouter, Mistral, xAI providers
- `[ ]` Handle provider-specific streaming and tool-calling formats

# Phase 7: Environments & Sandboxing
- `[ ]` Design environment execution traits
- `[ ]` Implement Docker container backend
- `[ ]` Implement Modal/Serverless backend
- `[ ]` Implement SSH and other remote backends

# Phase 8: Model Context Protocol (MCP)
- `[ ]` Implement MCP Server capabilities (`mcp_serve`)
- `[ ]` Implement ACP Adapter/Registry
- `[ ]` Support consuming external MCP tools

# Phase 9: Plugins
- `[ ]` Design dynamic plugin loader architecture
- `[ ]` Port core plugins and extensions

# Phase 10: Skills Ecosystem
- `[ ]` Implement Skill Manager and dynamic skill loading
- `[ ]` Port Skills Hub and Skills Guard
- `[ ]` Implement persistent skill storage and retrieval

# Phase 11: Browser Automation & Computer Use
- `[ ]` Implement Computer Use API (VNC/UI interactions)
- `[ ]` Integrate Browser Providers (Browserbase, Firecrawl, browser-use)

# Phase 12: Multimedia Tools
- `[ ]` Port Vision Tools
- `[ ]` Port Voice & TTS integration
- `[ ]` Port Video generation capabilities
