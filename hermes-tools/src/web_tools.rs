use crate::registry::Tool;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::net::IpAddr;
use reqwest::Url;

fn is_ip_safe(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            if ipv4.is_loopback() || ipv4.is_link_local() || ipv4.is_private() || ipv4.is_unspecified() {
                return false;
            }
        }
        IpAddr::V6(ipv6) => {
            if ipv6.is_loopback() || ipv6.is_unspecified() {
                return false;
            }
            if let Some(ipv4) = to_ipv4_mapped(&ipv6) {
                return is_ip_safe(IpAddr::V4(ipv4));
            }
            if (ipv6.segments()[0] & 0xffc0) == 0xfe80 {
                return false;
            }
            if (ipv6.segments()[0] & 0xfe00) == 0xfc00 {
                return false;
            }
        }
    }
    true
}

fn to_ipv4_mapped(ipv6: &std::net::Ipv6Addr) -> Option<std::net::Ipv4Addr> {
    let octets = ipv6.octets();
    if octets[0..10] == [0; 10] && octets[10] == 0xff && octets[11] == 0xff {
        Some(std::net::Ipv4Addr::new(octets[12], octets[13], octets[14], octets[15]))
    } else {
        None
    }
}

async fn is_url_safe(url_str: &str) -> bool {
    let parsed = match Url::parse(url_str) {
        Ok(u) => u,
        Err(_) => return false,
    };

    let host = match parsed.host_str() {
        Some(h) => h,
        None => return false,
    };

    if let Ok(ip) = host.parse::<IpAddr>() {
        return is_ip_safe(ip);
    }

    if let Ok(addrs) = tokio::net::lookup_host(format!("{}:80", host)).await {
        for addr in addrs {
            if !is_ip_safe(addr.ip()) {
                return false;
            }
        }
    }

    true
}

pub struct ReadUrlTool;

#[async_trait]
impl Tool for ReadUrlTool {
    fn name(&self) -> &'static str { "read_url" }
    fn toolset(&self) -> &'static str { "web" }
    fn schema(&self) -> Value {
        json!({
            "description": "Fetch content from a URL via HTTP request.",
            "parameters": {
                "type": "object",
                "properties": {
                    "url": { "type": "string", "description": "The URL to fetch." }
                },
                "required": ["url"]
            }
        })
    }
    async fn handle(&self, args: Value) -> Result<Value, String> {
        let url = match args.get("url").and_then(|v| v.as_str()) {
            Some(u) => u,
            None => return Ok(json!({ "error": "Missing or invalid 'url' argument" })),
        };

        if !is_url_safe(url).await {
            return Ok(json!({ "error": "Access denied: Requesting this target is restricted for security (SSRF prevention)." }));
        }

        match reqwest::get(url).await {
            Ok(resp) => {
                match resp.text().await {
                    Ok(text) => Ok(json!({ "success": true, "content": text })),
                    Err(e) => Ok(json!({ "error": format!("Failed to read response body: {}", e) })),
                }
            }
            Err(e) => Ok(json!({ "error": format!("Failed to execute request: {}", e) })),
        }
    }
}

inventory::submit!(crate::registry::RegisteredTool { factory: || std::sync::Arc::new(ReadUrlTool) });
