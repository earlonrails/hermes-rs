# Hermes-RS 🦀

A blazing fast, memory-safe, and highly concurrent Rust port of the Hermes Agent framework.

Hermes-RS provides a robust foundation for building advanced, tool-capable AI agents with an emphasis on safe execution, extensibility, and extreme performance.

## 🌟 Features

- **Multi-Provider Support:** Seamless integration with OpenAI, Anthropic, Mistral, Gemini, xAI, and OpenRouter. Complete with custom translation layers for provider-specific streaming (SSE) and tool-calling schemas.
- **Sandboxed Execution Environments:** Safely execute code and tools inside Docker containers, Modal serverless endpoints, or remote SSH targets.
- **WebAssembly Plugins:** Dynamically load highly secure, sandboxed `.wasm` plugins using the `wasmtime` engine, strictly enforcing gas/execution budgets.
- **Model Context Protocol (MCP):** Expose your native Rust tools to external systems (like Claude Desktop) via our built-in MCP Server, or consume external MCP tools natively.
- **Skills Ecosystem:** Persistent, declarative semantic memory for agents. Built on `rusqlite` and `fastembed` (local ONNX models) for lightning-fast, zero-cost cosine similarity skill retrieval.
- **Browser & Computer Use:** Headless browser automation via WebDriver (`thirtyfour`), alongside cross-platform programmatic mouse/keyboard simulation (`enigo`) and screen capture (`xcap`).
- **Multimedia Processing:** Native handlers for Vision processing, Speech-to-Text (Whisper), and Text-to-Speech integration.

## 🚀 Installation

You can easily install Hermes-RS directly from source using our install script:

```bash
curl -sSL https://raw.githubusercontent.com/earlonrails/hermes-rs/main/install.sh | bash
```

Alternatively, clone the repository and install it manually:

```bash
git clone https://github.com/earlonrails/hermes-rs.git
cd hermes-rs
cargo install --path hermes-cli
```

## 🏗️ Architecture

Hermes-RS is built as a highly modular Cargo workspace:

- `hermes-core`: Core configurations, constants, and paths.
- `hermes-state`: SQLite-backed state and session management.
- `hermes-tools`: Foundational tool trait and registry system.
- `hermes-providers`: Provider API clients, streaming, and tool-calling translation.
- `hermes-env`: Sandboxed execution backends (Docker, Modal, SSH).
- `hermes-mcp`: Model Context Protocol server and client adapters.
- `hermes-plugins`: WebAssembly (`wasmtime`) dynamic plugin manager.
- `hermes-skills`: Local ONNX embeddings and persistent skill vector storage.
- `hermes-browser`: WebDriver and Computer Use interactions.
- `hermes-multimedia`: Vision formatting and Audio API integrations.
- `hermes-agent`: The core conversational loop and Context Engine.
- `hermes-cli` / `hermes-gateway`: Frontend I/O and messaging integrations.

## 🙏 Acknowledgements

**Hermes-RS is heavily inspired by and ported from the original [Hermes Agent](https://github.com/earlonrails/hermes) project.**

We extend our deepest gratitude to the original authors of the Python-based Hermes Agent for pioneering the underlying architecture, tool-calling paradigms, and conversational loops that made this Rust port possible. Their visionary work on agentic workflows directly shaped the foundation of this project.

## 📝 License

This project is open-source and available under the MIT License.
