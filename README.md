# Athena 🦉

[![CI](https://github.com/earlonrails/athena/actions/workflows/ci.yml/badge.svg)](https://github.com/earlonrails/athena/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/earlonrails/athena/graph/badge.svg?token=YOURTOKEN)](https://codecov.io/gh/earlonrails/athena)

A blazing fast, memory-safe, and highly concurrent Rust port of the Hermes Agent framework.

Athena provides a robust foundation for building advanced, tool-capable AI agents with an emphasis on safe execution, extensibility, and extreme performance.

---

## 🌟 Features

- **Multi-Provider Support:** Seamless integration with OpenAI, Anthropic, Mistral, Gemini, xAI, and OpenRouter. Complete with custom translation layers for provider-specific streaming (SSE) and tool-calling schemas.
- **Sandboxed Execution Environments:** Safely execute code and tools inside Docker containers, Modal serverless endpoints, or remote SSH targets.
- **WebAssembly Plugins:** Dynamically load highly secure, sandboxed `.wasm` plugins using the `wasmtime` engine, strictly enforcing gas/execution budgets.
- **Model Context Protocol (MCP):** Expose your native Rust tools to external systems (like Claude Desktop) via our built-in MCP Server, or consume external MCP tools natively.
- **Skills Ecosystem:** Persistent, declarative semantic memory for agents. Built on `rusqlite` and `fastembed` (local ONNX models) for lightning-fast, zero-cost cosine similarity skill retrieval.
- **Browser & Computer Use:** Headless browser automation via WebDriver (`thirtyfour`), alongside cross-platform programmatic mouse/keyboard simulation (`enigo`) and screen capture (`xcap`).
- **Multimedia Processing:** Native handlers for Vision processing, Speech-to-Text (Whisper), and Text-to-Speech integration.

---

## 🚀 Installation

You can easily install Athena directly from source using our install script:

```bash
curl -sSL https://raw.githubusercontent.com/earlonrails/athena/main/install.sh | bash
```

Alternatively, clone the repository and install it manually:

```bash
git clone https://github.com/earlonrails/athena.git
cd athena
cargo install --path athena-cli
```

---

## 🏗️ Architecture

Athena is built as a highly modular Cargo workspace:

- `athena-core`: Core configurations, constants, and paths.
- `athena-state`: SQLite-backed state and session management.
- `athena-tools`: Foundational tool trait and registry system.
- `athena-providers`: Provider API clients, streaming, and tool-calling translation.
- `athena-env`: Sandboxed execution backends (Docker, Modal, SSH).
- `athena-mcp`: Model Context Protocol server and client adapters.
- `athena-plugins`: WebAssembly (`wasmtime`) dynamic plugin manager.
- `athena-skills`: Local ONNX embeddings and persistent skill vector storage.
- `athena-browser`: WebDriver and Computer Use interactions.
- `athena-multimedia`: Vision formatting and Audio API integrations.
- `athena-agent`: The core conversational loop and Context Engine.
- `athena-cli` / `athena-gateway`: Frontend I/O and messaging integrations.

---

## 💻 Usage & CLI Guide

Athena exposes a comprehensive CLI for managing agents, sessions, and integrations.

### 1. Interactive Chat Mode
Start a persistent conversational chat session with the default model:
```bash
athena chat
```

### 2. Run a One-Shot Query
Execute a single query directly from the terminal:
```bash
athena query "Analyze the files in this directory and summarize the project."
```

### 3. Launch the Web GUI Dashboard
Start the gorgeous local glassmorphic dashboard at `http://localhost:8000`:
```bash
athena dashboard
```

### 4. Backup & Restore
Quickly export and import your full agent state:
```bash
# Back up to a ZIP archive (logs folder automatically skipped for portability)
athena backup

# Restore an Athena state archive
athena import
```

---

## ⚙️ Configuration Setup

Athena reads global configs from `~/.athena/config.yaml` and environment credentials from `~/.athena/.env`.

### 🖥️ 1. Environment Variables (`.env`)
Configure your keys under the local environment:
```env
# AI Provider Keys
OPENAI_API_KEY=sk-proj-...
ANTHROPIC_API_KEY=sk-ant-...
GEMINI_API_KEY=AIzaSy...
MISTRAL_API_KEY=...
XAI_API_KEY=...
OPENROUTER_API_KEY=...

# Gateway Tokens
TELEGRAM_BOT_TOKEN=123456789:ABCdefGhI...
WHATSAPP_TOKEN=...
SLACK_BOT_TOKEN=xoxb-...
```

### 📄 2. Core Configurations (`config.yaml`)
Control default model selections, fallbacks, and local tool states:
```yaml
model:
  default: claude-3-5-sonnet-latest
  provider: anthropic
  fallback:
    - gpt-4o
    - gemini-1.5-pro

agent:
  yolo_mode: false # require manual approval for terminal/file executions
  max_turns: 20

tools:
  disabled:
    - browser_automation # dynamically disables browser-use capabilities
```

You can view and modify these configuration values interactively from the CLI:
```bash
# Display setup summary
athena dump

# Edit config parameters
athena config
```

---

## 🔑 Provider Configurations

Athena supports seamless, dynamic provider configurations and masked credential pools.

### Registering/Authenticating a Provider Key:
```bash
athena login
```
Follow the interactive wizard to pick a provider (e.g. OpenAI, Anthropic, Gemini) and paste your API token. This safely writes the credentials to the global state.

### Managing Pool Credentials:
List, update, or remove active credential records with:
```bash
athena auth
```

---

## 💬 Messaging Gateway Integrations

Integrate Athena Agent natively with Telegram, WhatsApp, and Slack.

### 🔹 1. Telegram Bot Gateway Setup
1. Message **@BotFather** on Telegram to create a new bot and obtain your `TELEGRAM_BOT_TOKEN`.
2. Add the token to your `~/.athena/.env` file.
3. Start the gateway service:
   ```bash
   athena gateway --platform telegram
   ```
4. Start messaging your Telegram Bot! The agent will receive messages, call local tools inside sandbox execution environments, and respond in real-time.

### 🔹 2. WhatsApp Integration Setup
1. Toggle the WhatsApp bridge using the CLI:
   ```bash
   athena whatsapp
   ```
2. The CLI will direct you through setting up a Node.js companion script using your WhatsApp API pairing code.
3. Scan the generated WhatsApp Web QR Code in your phone application to pair Athena as a chat companion.

### 🔹 3. Slack Integration Setup
1. Auto-generate a fully compliant Slack App Manifest JSON file:
   ```bash
   athena slack
   ```
2. Navigate to [api.slack.com/apps](https://api.slack.com/apps) and click **Create New App** -> **From an App Manifest**.
3. Copy-paste the generated JSON template to instantly configure redirect URLs, Slash commands, and bot user scopes.
4. Install the app to your workspace and add the `SLACK_BOT_TOKEN` to your credentials pool.

---

## 🔄 Architectural Enhancements & Benefits (Port from Python)

Porting Hermes from Python to Rust (`Athena`) introduced major structural enhancements that drastically improve system performance, security, and developer productivity:

### 1. 🦀 Compile-Time Type Safety & Stability
* **Python**: Relied heavily on runtime dictionary lookups and dynamic keyword arguments (`**kwargs`), creating a high risk of unexpected runtime failures during long agent steps.
* **Rust**: Leverages strict, strongly-typed compile-time models (e.g. `AIAgentBuilder`, `ToolRegistry`, and explicit generic bounds). Potential type mismatches and incomplete tool definitions are caught **exclusively at compile time**.

### 2. ⚡ Blazing Fast Concurrency & High Performance
* **Python**: Hampered by the Global Interpreter Lock (GIL), making parallel tool execution, concurrent agent steps, and simultaneous web scraping difficult and resource-heavy.
* **Rust**: Employs industry-standard async concurrency (`tokio` multi-threaded executor). Agents, tools, and vector calculations execute in parallel on native threads with zero interpreter overhead.

### 3. 🔒 Safe Execution & WASM Sandboxing
* **Python**: Relied on standard subprocess sandboxing which leaves local filesystems vulnerable during untrusted dynamic tool execution.
* **Rust**: Incorporates formal dynamic **WebAssembly (`wasmtime`) sandboxing** with strict, grain-level gas limits and execution budgets. Execution limits are strictly monitored and enforced.

### 4. 🪶 Zero-Cost Semantic Retrieval & Embeddings
* **Python**: Transitive execution depended on running external server processes or importing large, heavy PyTorch/TensorFlow environments.
* **Rust**: Integrates local ONNX embeddings (`fastembed-rs`) alongside lightweight C-level databases (`rusqlite`). Retains a near-zero memory footprint and runs cosine similarity scans instantly.

### 5. 🛡️ Strict TLS Control (Rustls)
* **Python**: Managed TLS through dynamic OpenSSL bindings prone to configuration drift across host operating systems.
* **Rust**: Strictly standardizes on **Rustls** for all first-party networking stacks, ensuring modern, memory-safe, and independent TLS layers across all host targets.

---

## 🙏 Acknowledgements

**Athena is heavily inspired by and ported from the original [Hermes Agent](https://github.com/earlonrails/hermes) project.**

We extend our deepest gratitude to the original authors of the Python-based Hermes Agent for pioneering the underlying architecture, tool-calling paradigms, and conversational loops that made this Rust port possible. Their visionary work on agentic workflows directly shaped the foundation of this project.

---

## 📝 License

This project is open-source and available under the MIT License.
