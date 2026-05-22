const fs = require('fs');
const path = require('path');

// Ensure target directory exists
const webAppSrcDir = path.join(__dirname, '..', 'apps', 'web-app', 'src');
if (!fs.existsSync(webAppSrcDir)) {
  fs.mkdirSync(webAppSrcDir, { recursive: true });
}

const data = {
  appName: "Athena",
  tagline: "Blazing fast, memory-safe, and highly concurrent Rust Agent Framework",
  description: "Athena provides a robust foundation for building advanced, tool-capable AI agents with an emphasis on sandboxed execution, WebAssembly extensibility, and extreme concurrency.",
  installation: {
    oneLiner: "curl -sSL https://raw.githubusercontent.com/earlonrails/athena/main/install.sh | bash",
    manual: [
      "git clone https://github.com/earlonrails/athena.git",
      "cd athena",
      "cargo install --path athena-cli"
    ]
  },
  features: [
    {
      id: "multi-provider",
      name: "Multi-Provider LLM",
      description: "Seamless integration with OpenAI, Anthropic, Gemini, Mistral, xAI, and OpenRouter, with custom translation layers for provider-specific streaming (SSE) and tool calling.",
      icon: "🌐",
      category: "Core"
    },
    {
      id: "sandboxed-exec",
      name: "Sandboxed Execution",
      description: "Run generated code and tools safely inside local Docker containers, Modal serverless endpoints, or secure remote SSH targets.",
      icon: "📦",
      category: "Security"
    },
    {
      id: "wasm-plugins",
      name: "WebAssembly Extensibility",
      description: "Dynamically load highly secure, sandboxed .wasm plugins using the wasmtime engine, strictly enforcing gas and execution budgets.",
      icon: "⚡",
      category: "Extensibility"
    },
    {
      id: "mcp-integration",
      name: "Model Context Protocol",
      description: "Expose native Rust tools to external systems (like Claude Desktop) via our built-in MCP server, or consume external MCP tools natively.",
      icon: "🔌",
      category: "Core"
    },
    {
      id: "skills-ecosystem",
      name: "ONNX Semantic Memory",
      description: "Persistent, declarative semantic memory using local fastembed ONNX models and SQLite for high-performance vector skill retrieval.",
      icon: "🧠",
      category: "Memory"
    },
    {
      id: "browser-automation",
      name: "Headless Browser & Computer Use",
      description: "Headless web automation via WebDriver (thirtyfour), programmatic mouse/keyboard simulation, and screen capture capabilities.",
      icon: "🖥️",
      category: "Computer Use"
    }
  ],
  commands: [
    {
      input: "athena login",
      description: "Register and authenticate your AI providers using a secure interactive login wizard.",
      output: [
        "🦉 Athena Login Wizard",
        "════════════════════════════════════════",
        "Select Provider to authenticate:",
        " > [1] Anthropic (Claude)",
        "   [2] OpenAI (GPT-4o/o3-mini)",
        "   [3] Gemini (Pro/Flash)",
        "   [4] Mistral",
        "",
        "Enter Anthropic API Key: ································",
        "✔ Credential verified & successfully written to secure credential pool."
      ]
    },
    {
      input: "athena chat",
      description: "Start a persistent, real-time conversational agent session.",
      output: [
        "🦉 Athena Interactive Agent Session (v0.12.0)",
        "Active Model: claude-3-5-sonnet-latest (Anthropic)",
        "Sandbox Target: Docker (Local container active)",
        "Press Ctrl+D or type 'exit' to quit.",
        "",
        "athena> Write a script to fetch the current weather in SF and print it.",
        "────────────────────────────────────────────────────────",
        "🤖 [Thinking] Parsing prompt... Generating execution plan.",
        "🛠️ [Calling Tool] code_sandbox_exec { language: 'python', script: '...' }",
        "🐳 [Docker Sandbox] Container booted up. Running script...",
        "🐳 [Docker Output] The temperature in San Francisco is 62°F with clear skies.",
        "🤖 The current weather in San Francisco is 62°F. The code ran safely inside a Docker container."
      ]
    },
    {
      input: "athena query \"Search for Rust files and verify tests\"",
      description: "Execute a single-turn agent command directly from your shell.",
      output: [
        "🤖 Query received: 'Search for Rust files and verify tests'",
        "🔍 Scanning local workspace directories...",
        "🛠️ [Calling Tool] grep_search { query: 'test', path: './src' }",
        "✔ Found 4 tests in 2 files.",
        "🛠️ [Calling Tool] shell_exec { cmd: 'cargo test' }",
        "🧪 Running cargo test inside the sandbox...",
        "🧪 test result: ok. 4 passed; 0 failed;",
        "🤖 Verified successfully: All 4 unit tests in the workspace are passing."
      ]
    },
    {
      input: "athena dashboard",
      description: "Launch the local premium glassmorphic agent monitoring dashboard.",
      output: [
        "Athena Web GUI Dashboard",
        "══════════════════════════",
        "Launching local dashboard at http://localhost:8000...",
        "Press Ctrl+C to stop.",
        "HTTP/1.1 200 OK Connection: close (Ready)"
      ]
    }
  ],
  skills: [
    {
      name: "Docker Code Execution Sandbox",
      trigger: "When asked to run python, bash, or node code securely",
      instruction: "Locate or spin up the designated local Docker container. Copy the code into the sandbox, execute it, and return stdout/stderr. Do not run any untrusted code on the host machine directly."
    },
    {
      name: "Web Scraping with Headless Chrome",
      trigger: "When asked to fetch details from a complex, JS-rendered website",
      instruction: "Connect via WebDriver protocol using thirtyfour. Navigate to the website. Wait for elements to load, take a screenshot if requested, extract inner text, and safely close the browser session."
    },
    {
      name: "Semantic Skillcosm Retrieval",
      trigger: "When the agent needs specific domain knowledge or pre-configured commands",
      instruction: "Use fastembed to compute the vector embedding of the user's query context. Perform cosine similarity retrieval in the rusqlite SQLite database. Fetch the top 3 matching instructions and append them to the system instructions prompt."
    },
    {
      name: "WASM Dynamic Plugin Loader",
      trigger: "When performing custom mathematical logic, data conversions, or gas-restricted functions",
      instruction: "Instantiate the strictly sandboxed wasmtime engine. Load the WebAssembly pre-compiled plugin binary. Execute the function under a maximum budget of 1,000,000 gas units to ensure it cannot loop indefinitely."
    },
    {
      name: "Model Context Protocol Client Bridge",
      trigger: "When needing access to tools provided by external desktop applications or server utilities",
      instruction: "Connect via stdio or SSE transport to the external MCP server. Query listed resources, prompt templates, and tools. Translate the tool definitions into Athena's registry format, allow LLM execution, and return responses back to the server."
    }
  ]
};

const outputPath = path.join(webAppSrcDir, 'data.json');
fs.writeFileSync(outputPath, JSON.stringify(data, null, 2), 'utf-8');
console.log(`Successfully generated web app data index at: ${outputPath}`);
