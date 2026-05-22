use std::io::{self, BufRead};

pub fn run_lsp() {
    eprintln!("Starting Athena LSP Server on stdin/stdout...");

    let capabilities_json = r#"{"jsonrpc":"2.0","result":{"capabilities":{"textDocumentSync":1,"completionProvider":{"resolveProvider":true},"hoverProvider":true,"codeActionProvider":true}},"id":1}"#;

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut line = String::new();

    while let Ok(n) = handle.read_line(&mut line) {
        if n == 0 {
            break;
        }

        if line.starts_with("Content-Length:") {
            line.clear();
            let _ = handle.read_line(&mut line);

            line.clear();
            let _ = handle.read_line(&mut line);

            if line.contains("initialize") {
                let response = format!("Content-Length: {}\r\n\r\n{}", capabilities_json.len(), capabilities_json);
                print!("{}", response);
                io::Write::flush(&mut io::stdout()).ok();
            }
        }
        line.clear();
    }
}
