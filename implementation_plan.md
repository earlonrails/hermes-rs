# Convert Hermes Agent to Rust (Phase 5)

We have successfully completed Phases 1 through 4, meaning the Rust port of Hermes is fully capable of terminal command execution, file operations, web requests, interactive local CLI, and a Telegram Bot interface.

Phase 5 will focus on reaching feature-parity with the advanced UI and tooling of the Python codebase. 

## User Review Required

> [!WARNING]
> **Workspace Restriction for Repo Move**
> You asked if we can move the project to its own repo. While I can definitely initialize a new Git repository for the `hermes-rs` directory (`git init`), I **cannot** move the folder out of the current `c:\Users\earlk\code\hermes-agent` workspace path. My tools are security-restricted to only operate within the currently open VS Code workspace. 
> 
> **How to proceed:** 
> 1. You can manually move the `hermes-rs` folder out to `c:\Users\earlk\code\hermes-rs`.
> 2. Open that new folder in your editor.
> 3. Send me a new message, and I will be able to operate in the new repo natively!
>
> If you just want me to run `git init` inside the current `hermes-rs` folder, let me know!

> [!IMPORTANT]
> **Scope of Phase 5 (Advanced Parity)**
> The Python codebase has incredibly complex agent logic (Context Compression, TUI Gateway, Prompt Caching, and 50+ specialized tools). 
> 
> I propose we focus Phase 5 on:
> 1. **Context Engine & Prompt Caching**: Porting `agent/prompt_builder.py` and `agent/context_engine.py` to ensure we aren't blowing up the token budget.
> 2. **Code Execution Environment**: Porting `code_execution_tool.py` for sandboxed AST evaluation.
> 3. **TUI Gateway Server**: Building the JSON-RPC backend (`hermes-tui-gateway`) so the existing Node.js Ink frontend can attach to our Rust agent.

## Open Questions

> [!CAUTION]
> 1. **Ink TUI vs Native Rust TUI**: The python version spins up a node.js subprocess for the Ink TUI. Instead of building the JSON-RPC bridge in Rust, would you rather just write a native Rust TUI using `ratatui`? This would drop the Node.js dependency entirely.
> 2. **Provider Adapters**: The Python version has deep support for Anthropic, Bedrock, Gemini, and Local models. Currently, we only use `async-openai`. Do you want to build custom provider adapters in Phase 5, or stick with OpenAI-compatibility for now?

---

## Proposed Changes

### [NEW] `hermes-rs/hermes-agent/src/context.rs`
- Port the token counting and context compression logic.
- Implement truncation for large tool outputs to prevent context overflow.

### [NEW] `hermes-rs/hermes-tui-gateway/` (If sticking with Node.js)
- A new crate implementing `tower-lsp` or a custom JSON-RPC server over stdio.
- Bridges Ink UI state updates with the Rust AIAgent.

### [NEW] `hermes-rs/hermes-tools/src/code_tool.rs`
- Secure evaluation of Python/JS code snippets (similar to `code_execution_tool.py`).

## Verification Plan

### Automated Tests
- Unit tests for context window token calculations.
- RPC protocol fuzzing for the TUI Gateway.

### Manual Verification
- Launch the TUI frontend and ensure it connects to the Rust backend properly.
