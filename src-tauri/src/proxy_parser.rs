use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProxyStatus {
    Unchecked,         // لم يتم الفحص
    TcpReachable,      // قابل للوصول عبر TCP
    Unreachable,       // غير قابل للوصول
    Timeout,           // انتهت مهلة الاتصال
    DnsFailed,         // فشل DNS
    MtprotoUnverified, // لم يتم التحقق من MTProto
}

impl std::str::FromStr for ProxyStatus {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "TcpReachable" => ProxyStatus::TcpReachable,
            "Unreachable" => ProxyStatus::Unreachable,
            "Timeout" => ProxyStatus::Timeout,
            "DnsFailed" => ProxyStatus::DnsFailed,
            "MtprotoUnverified" => ProxyStatus::MtprotoUnverified,
            _ => ProxyStatus::Unchecked,
        })
    }
}

impl ProxyStatus {
    pub fn parse_status(s: &str) -> Self {
        match s {
            "TcpReachable" => ProxyStatus::TcpReachable,
            "Unreachable" => ProxyStatus::Unreachable,
            "Timeout" => ProxyStatus::Timeout,
            "DnsFailed" => ProxyStatus::DnsFailed,
            "MtprotoUnverified" => ProxyStatus::MtprotoUnverified,
            _ => ProxyStatus::Unchecked,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ProxyStatus::Unchecked => "Unchecked",
            ProxyStatus::TcpReachable => "TcpReachable",
            ProxyStatus::Unreachable => "Unreachable",
            ProxyStatus::Timeout => "Timeout",
            ProxyStatus::DnsFailed => "DnsFailed",
            ProxyStatus::MtprotoUnverified => "MtprotoUnverified",
        }
    }

    pub fn label_ar(&self) -> &'static str {
        match self {
            ProxyStatus::Unchecked => "لم يتم الفحص",
            ProxyStatus::TcpReachable => "قابل للوصول عبر TCP",
            ProxyStatus::Unreachable => "غير قابل للوصول",
            ProxyStatus::Timeout => "انتهت مهلة الاتصال",
            ProxyStatus::DnsFailed => "فشل DNS",
            ProxyStatus::MtprotoUnverified => "لم يتم التحقق من MTProto",
        }
    }

    pub fn label_en(&self) -> &'static str {
        match self {
            ProxyStatus::Unchecked => "Unchecked",
            ProxyStatus::TcpReachable => "TCP Reachable",
            ProxyStatus::Unreachable => "Unreachable",
            ProxyStatus::Timeout => "Connection Timeout",
            ProxyStatus::DnsFailed => "DNS Resolution Failed",
            ProxyStatus::MtprotoUnverified => "MTProto Not Verified",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProxyItem {
    pub id: String,
    pub server: String,
    pub port: u16,
    pub secret: String,
    pub status: ProxyStatus,
    pub ping_ms: Option<u64>,
    pub last_check: Option<String>,
    pub is_favorite: bool,
    pub source_url: String,
}

impl ProxyItem {
    pub fn canonical_id(server: &str, port: u16, secret: &str) -> String {
        let clean_server = clean_server_host(server).to_lowercase();
        let clean_secret = secret.trim();
        format!("{}:{}:{}", clean_server, port, clean_secret)
    }

    pub fn tg_link(&self) -> String {
        format!(
            "tg://proxy?server={}&port={}&secret={}",
            self.server, self.port, self.secret
        )
    }

    pub fn https_link(&self) -> String {
        format!(
            "https://t.me/proxy?server={}&port={}&secret={}",
            self.server, self.port, self.secret
        )
    }
}

/// Cleans domain or IP host string, removing trailing dots, brackets, and leading/trailing whitespace.
pub fn clean_server_host(raw_server: &str) -> String {
    let mut s = raw_server.trim();
    // Trim URL prefix if accidentally passed
    if let Some(stripped) = s.strip_prefix("http://") {
        s = stripped;
    } else if let Some(stripped) = s.strip_prefix("https://") {
        s = stripped;
    }
    // Remove port if server string had a port attached
    if let Some((host, _)) = s.split_once(':')
        && !s.contains('[')
    {
        // Not an IPv6 literal
        s = host;
    }
    // Trim brackets from IPv6
    s = s.trim_matches('[').trim_matches(']');
    // Trim trailing dot commonly found in DNS domains like worldmaster.varomax.co.uk.
    s = s.trim_end_matches('.');
    s.to_string()
}

/// Parses a single MTProto proxy link.
/// Supported schemes:
/// - tg://proxy?server=...&port=...&secret=...
/// - https://t.me/proxy?server=...&port=...&secret=...
/// - http://t.me/proxy?server=...&port=...&secret=...
/// - t.me/proxy?server=...&port=...&secret=...
pub fn parse_proxy_line(line: &str, source_url: &str) -> Option<ProxyItem> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    let url_str = if trimmed.starts_with("t.me/") {
        format!("https://{}", trimmed)
    } else {
        trimmed.to_string()
    };

    let url = Url::parse(&url_str).ok()?;

    let mut server: Option<String> = None;
    let mut port: Option<u16> = None;
    let mut secret: Option<String> = None;

    for (k, v) in url.query_pairs() {
        match k.to_ascii_lowercase().as_str() {
            "server" => {
                let cleaned = clean_server_host(&v);
                if !cleaned.is_empty() {
                    server = Some(cleaned);
                }
            }
            "port" => {
                if let Ok(p) = v.trim().parse::<u16>()
                    && p > 0
                {
                    port = Some(p);
                }
            }
            "secret" => {
                let sec = v.trim().to_string();
                if !sec.is_empty() {
                    secret = Some(sec);
                }
            }
            _ => {}
        }
    }

    if let (Some(server), Some(port), Some(secret)) = (server, port, secret) {
        let id = ProxyItem::canonical_id(&server, port, &secret);
        Some(ProxyItem {
            id,
            server,
            port,
            secret,
            status: ProxyStatus::Unchecked,
            ping_ms: None,
            last_check: None,
            is_favorite: false,
            source_url: source_url.to_string(),
        })
    } else {
        None
    }
}

/// Parses multiple lines of raw proxy links, deduplicating them.
pub fn parse_proxies_text(content: &str, source_url: &str) -> Vec<ProxyItem> {
    let mut items = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for line in content.lines() {
        if let Some(proxy) = parse_proxy_line(line, source_url)
            && seen_ids.insert(proxy.id.clone())
        {
            items.push(proxy);
        }
    }

    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tme_proxy_with_trailing_dot() {
        let line = "https://t.me/proxy?server=worldmaster.varomax.co.uk.&port=5500&secret=eeRigzNJvXrFGRMCIMJdEA";
        let parsed = parse_proxy_line(line, "test").expect("Should parse");
        assert_eq!(parsed.server, "worldmaster.varomax.co.uk");
        assert_eq!(parsed.port, 5500);
        assert_eq!(parsed.secret, "eeRigzNJvXrFGRMCIMJdEA");
    }

    #[test]
    fn test_parse_tg_proxy() {
        let line =
            "tg://proxy?server=192.168.1.100&port=443&secret=dd10400103324995b07c030386e886e7f1";
        let parsed = parse_proxy_line(line, "test").expect("Should parse");
        assert_eq!(parsed.server, "192.168.1.100");
        assert_eq!(parsed.port, 443);
        assert_eq!(parsed.secret, "dd10400103324995b07c030386e886e7f1");
    }

    #[test]
    fn test_deduplication() {
        let text = r#"
https://t.me/proxy?server=example.com&port=443&secret=ee0102030405
tg://proxy?server=example.com.&port=443&secret=ee0102030405
https://t.me/proxy?server=example2.com&port=8443&secret=7HQighJPBNMYVRNB6tdkVw
"#;
        let list = parse_proxies_text(text, "source1");
        assert_eq!(list.len(), 2);
    }
}
