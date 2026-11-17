use crate::diagnostics::DiagnosticsLogger;
use crate::proxy_parser::{ProxyItem, parse_proxies_text};
use std::time::Duration;

pub const DEFAULT_SOURCE_URL: &str =
    "https://raw.githubusercontent.com/SoliSpirit/mtproto/master/all_proxies.txt";

pub struct ProxyFetcher {
    client: reqwest::Client,
    logger: DiagnosticsLogger,
}

impl ProxyFetcher {
    pub fn new(logger: DiagnosticsLogger) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("MTProxyDesk/1.0.0 (Windows NT 10.0; Win64; x64)")
            .build()
            .unwrap_or_default();

        Self { client, logger }
    }

    /// Fetches the raw text from a source URL with up to 3 retries
    pub async fn fetch_raw(&self, url: &str) -> Result<String, String> {
        self.logger.log(
            "INFO",
            "FETCHER",
            &format!("Fetching proxies from: {}", url),
        );
        let mut last_error = String::new();

        for attempt in 1..=3 {
            match self.client.get(url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.text().await {
                            Ok(text) => {
                                self.logger.log(
                                    "INFO",
                                    "FETCHER",
                                    &format!(
                                        "Successfully downloaded proxy list from {} ({} bytes)",
                                        url,
                                        text.len()
                                    ),
                                );
                                return Ok(text);
                            }
                            Err(e) => {
                                last_error = format!("Failed to read response body: {}", e);
                            }
                        }
                    } else {
                        last_error = format!("HTTP error status: {}", response.status());
                    }
                }
                Err(e) => {
                    last_error = format!("Network request failed: {}", e);
                }
            }

            self.logger.log(
                "WARN",
                "FETCHER",
                &format!("Attempt {}/3 failed for {}: {}", attempt, url, last_error),
            );

            if attempt < 3 {
                tokio::time::sleep(Duration::from_millis(1500 * attempt)).await;
            }
        }

        Err(last_error)
    }

    /// Fetches proxies from all provided sources, parsing and combining them.
    pub async fn fetch_all(&self, sources: &[String]) -> (Vec<ProxyItem>, Vec<String>) {
        let mut all_proxies = Vec::new();
        let mut errors = Vec::new();

        for source in sources {
            match self.fetch_raw(source).await {
                Ok(raw_text) => {
                    let parsed = parse_proxies_text(&raw_text, source);
                    self.logger.log(
                        "INFO",
                        "FETCHER",
                        &format!("Parsed {} valid proxies from {}", parsed.len(), source),
                    );
                    all_proxies.extend(parsed);
                }
                Err(e) => {
                    self.logger.log(
                        "ERROR",
                        "FETCHER",
                        &format!("Error fetching source {}: {}", source, e),
                    );
                    errors.push(format!("{}: {}", source, e));
                }
            }
        }

        // Deduplicate across all sources
        let mut deduplicated = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for p in all_proxies {
            if seen.insert(p.id.clone()) {
                deduplicated.push(p);
            }
        }

        (deduplicated, errors)
    }
}
