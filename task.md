# Phase 1: Foundation & State Management (Completed)
- `[x]` Initialize the Cargo workspace.
    - `[x]` Create root `Cargo.toml`
    - `[x]` Create `athena-core` crate
    - `[x]` Create `athena-state` crate
- `[x]` Port `hermes_constants.py` to `athena-core/src/paths.rs`
- `[x]` Port configuration data structures (`athena_cli/config.py`) to `athena-core/src/config.rs`
- `[x]` Port `athena_state.py` (SessionDB) to `athena-state/src/db.rs` using `rusqlite`
- `[x]` Port `hermes_logging.py` using `tracing` (can be deferred or done in `athena-core`)

# Phase 2: Core Agent & Tool Registry
- `[x]` Create `athena-tools` crate
- `[x]` Port `tools/registry.py` to define the Tool Trait and macro system
- `[x]` Port foundational LLM loop from `run_agent.py`
    - `[x]` Add `athena-agent` to workspace `Cargo.toml`
    - `[x]` Create `athena-agent/Cargo.toml` and `src/lib.rs`
    - `[x]` Implement `budget.rs` (IterationBudget)
    - `[x]` Implement `messages.rs` (Strongly-typed LLM messages)
    - `[x]` Implement `config.rs` and `builder.rs` (AIAgentBuilder)
    - `[x]` Implement `agent.rs` (AIAgent core loop)

# Phase 3: Core Tool Implementations
- `[x]` File Tools (`athena-tools/src/file_tools.rs`)
    - `[x]` Implement `read_file`
    - `[x]` Implement `write_file`
    - `[x]` Implement `list_dir`
    - `[x]` Implement `search_files`
- `[x]` Patch Tool (`athena-tools/src/patch_tool.rs`)
    - `[x]` Implement basic fuzzy-matching patch application
- `[x]` Terminal Tool (`athena-tools/src/terminal_tool.rs`)
    - `[x]` Implement `run_command` with timeout and output capture
- `[x]` Web Tools (`athena-tools/src/web_tools.rs`)
    - `[x]` Implement `web_search` and `read_url`

# Phase 4: CLI Frontend & Gateway
- `[x]` Create `athena-cli` crate
- `[x]` Implement CLI entry point and setup logging/agent builder
- `[x]` Implement persistent chat loop (`interactive.rs`)
- `[x]` Create `athena-gateway` crate for messaging platforms
    - `[x]` Implement Telegram Bot platform integration

# Phase 5: Advanced Parity
- `[x]` Context Engine (`athena-agent/src/context.rs`)
    - `[x]` Implement token counting logic
    - `[x]` Implement message truncation and compression
- `[ ]` Code Execution Tool (`athena-tools/src/code_tool.rs`) (Minimal stub)
- `[ ]` TUI Gateway (`athena-tui-gateway`) (Minimal JSON-RPC stub)
    - `[ ]` Create `athena-tui-gateway` crate
    - `[ ]` Implement JSON-RPC server over stdio

# Phase 6: Provider Parity
- `[x]` Implement robust LLM provider traits
- `[x]` Port OpenAI, Anthropic, Gemini, OpenRouter, Mistral, xAI providers
- `[x]` Handle provider-specific streaming and tool-calling formats

# Phase 7: Environments & Sandboxing
- `[x]` Design environment execution traits
- `[x]` Implement Docker container backend
- `[x]` Implement Modal/Serverless backend
- `[x]` Implement SSH and other remote backends

# Phase 8: Model Context Protocol (MCP)
- `[x]` Implement MCP Server capabilities (`mcp_serve`)
- `[x]` Implement ACP Adapter/Registry
- `[x]` Support consuming external MCP tools

# Phase 9: Plugins
- `[x]` Design dynamic plugin loader architecture
- `[x]` Port core plugins and extensions

# Phase 10: Skills Ecosystem
- `[x]` Implement Skill Manager and dynamic skill loading
- `[x]` Port Skills Hub and Skills Guard
- `[x]` Implement persistent skill storage and retrieval

# Phase 11: Browser Automation & Computer Use
- `[x]` Implement Computer Use API (VNC/UI interactions)
- `[x]` Integrate Browser Providers (Browserbase, Firecrawl, browser-use)

# Phase 12: Multimedia Tools
- `[x]` Port Vision Tools
- `[x]` Port Voice & TTS integration
- `[x]` Port Video generation capabilities

# Phase 13: CLI Subcommands Implementation
- `[x]` Implement unimplemented CLI subcommands:
    - `[x]` Chat
    - `[x]` Model
    - `[x]` Fallback
    - `[x]` Gateway (Telegram only)
    - `[ ]` Lsp (Minimal stub)
    - `[x]` Setup
    - `[ ]` Whatsapp (Config/Manifest only)
    - `[ ]` Slack (Config/Manifest only)
    - `[x]` Login
    - `[x]` Logout
    - `[x]` Auth
    - `[x]` Status
    - `[x]` Cron
    - `[x]` Webhook
    - `[x]` Kanban
    - `[x]` Hooks
    - `[x]` Doctor
    - `[x]` Dump
    - `[x]` Debug
    - `[x]` Backup
    - `[x]` Checkpoints
    - `[x]` Import
    - `[x]` Config
    - `[x]` Pairing
    - `[x]` Skills
    - `[x]` Plugins
    - `[x]` Curator
    - `[x]` Memory
    - `[x]` Tools
    - `[x]` ComputerUse
    - `[x]` Mcp
    - `[x]` Sessions
    - `[x]` Insights
    - `[x]` Claw
    - `[x]` Version
    - `[x]` Update
    - `[x]` Uninstall
    - `[x]` Acp
    - `[x]` Profile
    - `[x]` Completion
    - `[ ]` Dashboard (Static HTML stub)
    - `[x]` Logs
    - `[x]` ListTools (Implemented 2026-05-28)
    - `[x]` ListToolsets (Implemented 2026-05-28)

# Phase 14: Athena Rebranding 🦉
- `[x]` Rebrand primary package and binary name from `hermes` to `athena` in Cargo files
- `[x]` Rebrand environment variable namespaces and home directory configurations from `HERMES` / `.hermes` to `ATHENA` / `.athena`
- `[x]` Rebrand documentation guides in README to Athena

# Phase 15: Unit Test Coverage
- `[x]` Achieve 100% test coverage for `athena-providers`
- `[x]` Achieve >94% test coverage for `athena-tools`
- `[x]` Achieve >95% test coverage for `athena-env`
- `[x]` Achieve high test coverage for `athena-skills`
- `[x]` Achieve >95% test coverage for `athena-multimedia`
- `[x]` Achieve high test coverage for `athena-plugins`
- `[x]` Achieve high test coverage for `athena-mcp`
- `[x]` Achieve high test coverage for Phase 5 components (`athena-agent`, `athena-tui-gateway`)
