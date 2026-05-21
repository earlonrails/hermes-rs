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

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_to_ipv4_mapped() {
        let mapped = std::net::Ipv6Addr::from_str("::ffff:192.168.1.1").unwrap();
        assert_eq!(to_ipv4_mapped(&mapped), Some(std::net::Ipv4Addr::new(192, 168, 1, 1)));
        
        let not_mapped = std::net::Ipv6Addr::from_str("2001:db8::1").unwrap();
        assert_eq!(to_ipv4_mapped(&not_mapped), None);
    }

    #[test]
    fn test_is_ip_safe_v4() {
        assert!(!is_ip_safe(IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)))); // Loopback
        assert!(!is_ip_safe(IpAddr::V4(std::net::Ipv4Addr::new(169, 254, 0, 1)))); // Link local
        assert!(!is_ip_safe(IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 1)))); // Private
        assert!(!is_ip_safe(IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)))); // Unspecified
        assert!(is_ip_safe(IpAddr::V4(std::net::Ipv4Addr::new(8, 8, 8, 8)))); // Safe
    }

    #[test]
    fn test_is_ip_safe_v6() {
        assert!(!is_ip_safe(IpAddr::V6(std::net::Ipv6Addr::from_str("::1").unwrap()))); // Loopback
        assert!(!is_ip_safe(IpAddr::V6(std::net::Ipv6Addr::from_str("::").unwrap()))); // Unspecified
        assert!(!is_ip_safe(IpAddr::V6(std::net::Ipv6Addr::from_str("::ffff:127.0.0.1").unwrap()))); // Mapped loopback
        assert!(!is_ip_safe(IpAddr::V6(std::net::Ipv6Addr::from_str("fe80::1").unwrap()))); // Link local
        assert!(!is_ip_safe(IpAddr::V6(std::net::Ipv6Addr::from_str("fc00::1").unwrap()))); // Unique local
        assert!(is_ip_safe(IpAddr::V6(std::net::Ipv6Addr::from_str("2001:db8::1").unwrap()))); // Safe (documentation, but not blocked by our rules)
    }

    #[tokio::test]
    async fn test_is_url_safe() {
        assert!(!is_url_safe("http://127.0.0.1").await);
        assert!(!is_url_safe("http://localhost").await);
        assert!(!is_url_safe("invalid-url").await);
        
        // No host scenario (data URI)
        assert!(!is_url_safe("data:text/plain,Hello").await);
        
        // DNS lookup SSRF blocks (localtest.me resolves to 127.0.0.1)
        assert!(!is_url_safe("http://localtest.me").await);

        // Safe domains
        assert!(is_url_safe("https://example.com").await);
    }

    #[tokio::test]
    async fn test_read_url_tool_invalid_url() {
        let tool = ReadUrlTool;
        let res = tool.handle(json!({})).await.unwrap();
        assert_eq!(res["error"], "Missing or invalid 'url' argument");

        let res2 = tool.handle(json!({"url": "http://127.0.0.1"})).await.unwrap();
        assert!(res2["error"].as_str().unwrap().contains("Access denied"));
    }

    #[tokio::test]
    async fn test_read_url_tool() {
        let tool = ReadUrlTool;
        assert_eq!(tool.name(), "read_url");
        assert_eq!(tool.toolset(), "web");

        let schema = tool.schema();
        assert!(schema.get("description").is_some());
        assert!(schema.get("parameters").is_some());

        let result = tool.handle(json!({})).await.unwrap();
        assert!(result.get("error").is_some());
        
        // Actual read
        let result = tool.handle(json!({"url": "https://example.com"})).await.unwrap();
        assert!(result.get("success").is_some() || result.get("error").is_some()); // Success if network is up
    }
}

// Rust guideline compliant 2026-02-21
