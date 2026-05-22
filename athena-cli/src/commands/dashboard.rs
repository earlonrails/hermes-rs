use std::net::TcpListener;
use std::io::{Write, Read};

pub fn run_dashboard() {
    println!("\nAthena Web GUI Dashboard");
    println!("══════════════════════════\n");
    println!("Launching local dashboard at http://localhost:8000...");
    println!("Press Ctrl+C to stop.");
    println!();

    let listener = match TcpListener::bind("127.0.0.1:8000") {
        Ok(l) => l,
        Err(e) => {
            println!("✗ Failed to bind to port 8000: {}. Is another server running?", e);
            return;
        }
    };

    let html_content = r#"<!DOCTYPE html>
<html>
<head>
    <title>Athena Agent Dashboard</title>
    <style>
        body {
            background: linear-gradient(135deg, #0f172a, #1e1b4b);
            color: #f8fafc;
            font-family: 'Outfit', -apple-system, sans-serif;
            margin: 0;
            padding: 40px;
            display: flex;
            flex-direction: column;
            align-items: center;
            min-height: 100vh;
        }
        .dashboard {
            background: rgba(255, 255, 255, 0.03);
            backdrop-filter: blur(16px);
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 24px;
            padding: 40px;
            width: 100%;
            max-width: 800px;
            box-shadow: 0 20px 50px rgba(0, 0, 0, 0.3);
        }
        h1 {
            font-size: 2.5rem;
            margin-bottom: 8px;
            background: linear-gradient(to right, #38bdf8, #818cf8);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }
        .subtitle {
            color: #94a3b8;
            margin-bottom: 40px;
        }
        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
            gap: 24px;
        }
        .card {
            background: rgba(255, 255, 255, 0.02);
            border: 1px solid rgba(255, 255, 255, 0.05);
            border-radius: 16px;
            padding: 24px;
            transition: all 0.3s ease;
        }
        .card:hover {
            transform: translateY(-4px);
            border-color: rgba(99, 102, 241, 0.3);
            background: rgba(255, 255, 255, 0.04);
        }
        .card h3 {
            margin: 0 0 8px 0;
            color: #cbd5e1;
        }
        .card p {
            margin: 0;
            font-size: 1.5rem;
            font-weight: 600;
            color: #38bdf8;
        }
    </style>
</head>
<body>
    <div class="dashboard">
        <h1>Athena Agent Dashboard</h1>
        <div class="subtitle">Multi-Agent Workspace Hub & Analytics</div>
        <div class="grid">
            <div class="card">
                <h3>Workspace Status</h3>
                <p>ONLINE</p>
            </div>
            <div class="card">
                <h3>Inference Profile</h3>
                <p>Default</p>
            </div>
            <div class="card">
                <h3>Port</h3>
                <p>8000</p>
            </div>
        </div>
    </div>
</body>
</html>"#;

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let mut buffer = [0; 1024];
            let _ = stream.read(&mut buffer);

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                html_content.len(),
                html_content
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    }
}
