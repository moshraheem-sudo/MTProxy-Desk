use crate::diagnostics::DiagnosticsLogger;
use crate::proxy_parser::{parse_proxies_text, ProxyItem};
use std::time::Duration;

pub const DEFAULT_PRIMARY_SOURCE: &str =
    "https://raw.githubusercontent.com/SoliSpirit/mtproto/master/all_proxies.txt";
pub const DEFAULT_SECONDARY_SOURCE: &str =
    "https://raw.githubusercontent.com/moshraheem-sudo/mtproto-auto-proxies/main/proxies.txt";

// Alias for backwards compatibility
pub const DEFAULT_SOURCE_URL: &str = DEFAULT_PRIMARY_SOURCE;

pub struct ProxyFetcher {
    client: reqwest::Client,
    logger: DiagnosticsLogger,
}

impl ProxyFetcher {
    pub fn new(logger: DiagnosticsLogger) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(12))
            .user_agent("MTProxyDesk/1.0.1 (Windows NT 10.0; Win64; x64)")
            .build()
            .unwrap_or_default();

        Self { client, logger }
    }

    /// Fetches the raw text from a source URL with up to 3 retries and exponential backoff
    pub async fn fetch_raw(&self, url: &str) -> Result<String, String> {
        fetch_raw_endpoint(&self.client, &self.logger, url).await
    }

    /// Fetches proxies from all provided sources concurrently and independently.
    /// Fault tolerance: If one source fails, times out, or becomes unavailable,
    /// the app seamlessly continues working using proxies fetched from the other source.
    /// Strict deduplication: Canonical key (server + port + secret) ensures no duplicates.
    pub async fn fetch_all(&self, sources: &[String]) -> (Vec<ProxyItem>, Vec<String>) {
        let mut source_list = sources.to_vec();

        // Ensure both Primary (SoliSpirit) and Secondary (mtproto-auto-proxies) are always included
        if !source_list.iter().any(|s| s == DEFAULT_PRIMARY_SOURCE) {
            source_list.push(DEFAULT_PRIMARY_SOURCE.to_string());
        }
        if !source_list.iter().any(|s| s == DEFAULT_SECONDARY_SOURCE) {
            source_list.push(DEFAULT_SECONDARY_SOURCE.to_string());
        }

        self.logger.log(
            "INFO",
            "FETCHER",
            &format!(
                "Starting concurrent dual-source proxy synchronization across {} sources...",
                source_list.len()
            ),
        );

        // Spawn concurrent async tasks for each source for true parallel fetching and fault isolation
        let mut tasks = Vec::new();
        for source_url in source_list {
            let client_clone = self.client.clone();
            let logger_clone = self.logger.clone();
            tasks.push(tokio::spawn(async move {
                let res = fetch_raw_endpoint(&client_clone, &logger_clone, &source_url).await;
                (source_url, res)
            }));
        }

        let mut all_proxies = Vec::new();
        let mut errors = Vec::new();
        let mut successful_sources = 0;

        for task in tasks {
            match task.await {
                Ok((source_url, Ok(raw_text))) => {
                    let parsed = parse_proxies_text(&raw_text, &source_url);
                    self.logger.log(
                        "INFO",
                        "FETCHER",
                        &format!(
                            "Source SUCCESS [{}]: parsed {} valid proxies",
                            source_url,
                            parsed.len()
                        ),
                    );
                    all_proxies.extend(parsed);
                    successful_sources += 1;
                }
                Ok((source_url, Err(err_msg))) => {
                    self.logger.log(
                        "WARN",
                        "FETCHER",
                        &format!(
                            "Source FAILOVER [{}]: {} (Secondary/other sources will continue)",
                            source_url, err_msg
                        ),
                    );
                    errors.push(format!("{}: {}", source_url, err_msg));
                }
                Err(join_err) => {
                    let err = format!("Worker task panicked or failed: {}", join_err);
                    self.logger.log("ERROR", "FETCHER", &err);
                    errors.push(err);
                }
            }
        }

        // Strict Canonical Deduplication:
        // Canonical ID is lowercase(clean_server):port:clean_secret
        // This unifies tg://proxy and https://t.me/proxy across both feeds.
        let mut deduplicated = Vec::new();
        let mut seen_canonical_keys = std::collections::HashSet::new();

        for p in all_proxies {
            if seen_canonical_keys.insert(p.id.clone()) {
                deduplicated.push(p);
            }
        }

        self.logger.log(
            "INFO",
            "FETCHER",
            &format!(
                "Dual-source sync finished. Successfully queried {} sources. Unified deduplicated proxies: {}. (Errors: {})",
                successful_sources,
                deduplicated.len(),
                errors.len()
            ),
        );

        (deduplicated, errors)
    }
}

/// Helper function to fetch from a single endpoint with retries
async fn fetch_raw_endpoint(
    client: &reqwest::Client,
    logger: &DiagnosticsLogger,
    url: &str,
) -> Result<String, String> {
    logger.log("INFO", "FETCHER", &format!("Fetching proxies from: {}", url));
    let mut last_error = String::new();

    for attempt in 1..=3 {
        match client.get(url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.text().await {
                        Ok(text) => {
                            logger.log(
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

        logger.log(
            "WARN",
            "FETCHER",
            &format!("Attempt {}/3 failed for {}: {}", attempt, url, last_error),
        );

        if attempt < 3 {
            tokio::time::sleep(Duration::from_millis(1000 * attempt)).await;
        }
    }

    Err(last_error)
}
