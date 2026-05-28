use std::io::{self, BufRead};

pub fn run_acp() {
    println!("\nAthena Agent Client Protocol (ACP) Server");
    println!("═══════════════════════════════════════════\n");
    println!("Launching local ACP listener server via standard JSON-RPC on stdin/stdout...");
    println!("Press Ctrl+C to stop.");
    println!();

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut line = String::new();

    let handshake_response = r#"{"jsonrpc":"2.0","result":{"protocolVersion":"0.1.0","capabilities":{"agent":true,"client":false}},"id":1}"#;

    while let Ok(n) = handle.read_line(&mut line) {
        if n == 0 {
            break;
        }

        let trimmed = line.trim();
        if trimmed.contains("\"method\":\"handshake\"") {
            let response = format!("{}\n", handshake_response);
            print!("{}", response);
            io::Write::flush(&mut io::stdout()).ok();
        }
        line.clear();
    }
}

// Rust guideline compliant 2026-02-21
