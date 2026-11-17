use crate::proxy_parser::{ProxyItem, ProxyStatus};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpStream, lookup_host};
use tokio::sync::Semaphore;
use tokio::time::Instant;

pub struct ValidationResult {
    pub id: String,
    pub status: ProxyStatus,
    pub ping_ms: Option<u64>,
}

/// Validates a single proxy via DNS lookup and TCP handshake.
/// Distinguishes between TCP port reachability and actual MTProto protocol handshake.
pub async fn validate_single_proxy(proxy: &ProxyItem, timeout_secs: u64) -> ValidationResult {
    let timeout_duration = Duration::from_secs(timeout_secs);
    let host_port = format!("{}:{}", proxy.server, proxy.port);

    // Step 1: DNS Lookup with timeout
    let dns_result = tokio::time::timeout(timeout_duration, lookup_host(&host_port)).await;
    let addrs = match dns_result {
        Ok(Ok(mut iter)) => {
            if let Some(addr) = iter.next() {
                addr
            } else {
                return ValidationResult {
                    id: proxy.id.clone(),
                    status: ProxyStatus::DnsFailed,
                    ping_ms: None,
                };
            }
        }
        Ok(Err(_)) => {
            return ValidationResult {
                id: proxy.id.clone(),
                status: ProxyStatus::DnsFailed,
                ping_ms: None,
            };
        }
        Err(_) => {
            return ValidationResult {
                id: proxy.id.clone(),
                status: ProxyStatus::Timeout,
                ping_ms: None,
            };
        }
    };

    // Step 2: TCP Connect and RTT Latency measurement
    let start = Instant::now();
    let connect_result = tokio::time::timeout(timeout_duration, TcpStream::connect(addrs)).await;

    match connect_result {
        Ok(Ok(_stream)) => {
            let elapsed_ms = start.elapsed().as_millis() as u64;
            // Successfully connected via TCP:
            // Marked as TcpReachable (NOT falsely claiming MTProto verified)
            ValidationResult {
                id: proxy.id.clone(),
                status: ProxyStatus::TcpReachable,
                ping_ms: Some(elapsed_ms),
            }
        }
        Ok(Err(_err)) => ValidationResult {
            id: proxy.id.clone(),
            status: ProxyStatus::Unreachable,
            ping_ms: None,
        },
        Err(_) => ValidationResult {
            id: proxy.id.clone(),
            status: ProxyStatus::Timeout,
            ping_ms: None,
        },
    }
}

/// Runs validation across multiple proxies concurrently with a semaphore limit.
pub async fn validate_proxies_batch<F>(
    proxies: Vec<ProxyItem>,
    concurrency_limit: usize,
    timeout_secs: u64,
    on_result: F,
) where
    F: Fn(ValidationResult) + Send + Sync + 'static,
{
    let semaphore = Arc::new(Semaphore::new(concurrency_limit.clamp(1, 100)));
    let on_result = Arc::new(on_result);
    let mut tasks = Vec::new();

    for proxy in proxies {
        let sem = semaphore.clone();
        let callback = on_result.clone();

        let task = tauri::async_runtime::spawn(async move {
            let _permit = sem.acquire().await;
            let res = validate_single_proxy(&proxy, timeout_secs).await;
            callback(res);
        });

        tasks.push(task);
    }

    for task in tasks {
        let _ = task.await;
    }
}
