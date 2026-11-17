use crate::database::Database;
use crate::diagnostics::DiagnosticsLogger;
use crate::proxy_fetcher::ProxyFetcher;
use crate::proxy_parser::{ProxyItem, ProxyStatus};
use crate::proxy_validator::{validate_proxies_batch, validate_single_proxy};
use crate::settings::AppSettings;
use crate::telegram_launcher::{TelegramLaunchResult, TelegramLauncher};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagerStats {
    pub total: usize,
    pub tcp_reachable: usize,
    pub unreachable: usize,
    pub timeout: usize,
    pub dns_failed: usize,
    pub unchecked: usize,
    pub best_ping_ms: Option<u64>,
    pub last_update: String,
    pub is_testing: bool,
}

pub struct ProxyManager {
    pub db: Arc<Database>,
    pub logger: DiagnosticsLogger,
    pub fetcher: ProxyFetcher,
    pub launcher: TelegramLauncher,
    pub settings: Arc<Mutex<AppSettings>>,
    pub is_testing: Arc<Mutex<bool>>,
}

impl ProxyManager {
    pub fn new(
        db: Arc<Database>,
        logger: DiagnosticsLogger,
        settings: Arc<Mutex<AppSettings>>,
    ) -> Self {
        let fetcher = ProxyFetcher::new(logger.clone());
        let launcher = TelegramLauncher::new(logger.clone());

        Self {
            db,
            logger,
            fetcher,
            launcher,
            settings,
            is_testing: Arc::new(Mutex::new(false)),
        }
    }

    /// Fetches proxies from all enabled sources, merges them into the database,
    /// and falls back to existing DB proxies if offline.
    pub async fn refresh_proxies(&self) -> Result<Vec<ProxyItem>, String> {
        let sources = self.db.get_sources().map_err(|e| e.to_string())?;
        self.logger.log(
            "INFO",
            "MANAGER",
            &format!("Refreshing proxies from {} sources", sources.len()),
        );

        let (fetched_proxies, errors) = self.fetcher.fetch_all(&sources).await;

        if fetched_proxies.is_empty() {
            // Check if we have offline proxies in the database
            let existing = self.db.get_all_proxies().map_err(|e| e.to_string())?;
            if !existing.is_empty() {
                self.logger.log(
                    "WARN",
                    "MANAGER",
                    &format!(
                        "Network fetch yielded 0 items. Utilizing cached offline list ({} items). Errors: {:?}",
                        existing.len(),
                        errors
                    ),
                );
                return Ok(existing);
            } else {
                return Err(format!(
                    "Failed to fetch proxies from network and local cache is empty. Errors: {:?}",
                    errors
                ));
            }
        }

        // Save fetched proxies to SQLite (preserving user favorites and prior results)
        let count = self
            .db
            .upsert_proxies(&fetched_proxies)
            .map_err(|e| e.to_string())?;
        self.logger.log(
            "INFO",
            "MANAGER",
            &format!("Database updated with {} proxies", count),
        );

        // Update timestamp
        let now_str = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let _ = self.db.set_setting("last_update_timestamp", &now_str);
        if let Ok(mut s) = self.settings.lock() {
            s.last_update_timestamp = Some(now_str);
        }

        self.db.get_all_proxies().map_err(|e| e.to_string())
    }

    /// Validates all proxies currently stored in the database.
    pub async fn validate_all(&self) -> Result<(), String> {
        {
            let mut testing = self.is_testing.lock().unwrap();
            if *testing {
                return Err("Validation is already in progress.".to_string());
            }
            *testing = true;
        }

        self.logger.log(
            "INFO",
            "MANAGER",
            "Starting batch validation of all proxies.",
        );

        let proxies = match self.db.get_all_proxies() {
            Ok(p) => p,
            Err(e) => {
                let mut testing = self.is_testing.lock().unwrap();
                *testing = false;
                return Err(e.to_string());
            }
        };

        let (concurrency, timeout) = {
            let s = self.settings.lock().unwrap();
            (s.max_concurrency, s.check_timeout_secs)
        };

        let db_clone = self.db.clone();
        let logger_clone = self.logger.clone();

        validate_proxies_batch(proxies, concurrency, timeout, move |result| {
            let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let _ = db_clone.update_proxy_status(&result.id, &result.status, result.ping_ms, &now);
            if let Some(ms) = result.ping_ms {
                logger_clone.log(
                    "DEBUG",
                    "VALIDATOR",
                    &format!("Proxy {} reachable ({}ms)", result.id, ms),
                );
            }
        })
        .await;

        {
            let mut testing = self.is_testing.lock().unwrap();
            *testing = false;
        }

        self.logger
            .log("INFO", "MANAGER", "Batch validation completed.");
        Ok(())
    }

    /// Validates a single proxy by ID
    pub async fn validate_single(&self, id: &str) -> Result<ProxyItem, String> {
        let proxies = self.db.get_all_proxies().map_err(|e| e.to_string())?;
        let proxy = proxies
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| "Proxy not found".to_string())?;

        let timeout = self.settings.lock().unwrap().check_timeout_secs;
        let res = validate_single_proxy(&proxy, timeout).await;

        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.db
            .update_proxy_status(&res.id, &res.status, res.ping_ms, &now)
            .map_err(|e| e.to_string())?;

        let mut updated = proxy;
        updated.status = res.status;
        updated.ping_ms = res.ping_ms;
        updated.last_check = Some(now);

        Ok(updated)
    }

    /// Finds the fastest reachable proxy
    pub fn get_fastest_proxy(&self) -> Option<ProxyItem> {
        let proxies = self.db.get_all_proxies().ok()?;
        let mut reachable: Vec<ProxyItem> = proxies
            .into_iter()
            .filter(|p| p.status == ProxyStatus::TcpReachable && p.ping_ms.is_some())
            .collect();

        reachable.sort_by_key(|p| p.ping_ms.unwrap_or(u64::MAX));
        reachable.into_iter().next()
    }

    /// Connects to a proxy via Telegram
    pub fn launch_telegram_proxy(&self, id: &str) -> TelegramLaunchResult {
        let proxies = match self.db.get_all_proxies() {
            Ok(p) => p,
            Err(e) => {
                return TelegramLaunchResult {
                    success: false,
                    message_ar: format!("خطأ في قاعدة البيانات: {}", e),
                    message_en: format!("Database error: {}", e),
                    tg_link: String::new(),
                    fallback_web_link: String::new(),
                    official_download_url: "https://desktop.telegram.org/".to_string(),
                    is_telegram_installed: false,
                };
            }
        };

        let proxy = match proxies.into_iter().find(|p| p.id == id) {
            Some(p) => p,
            None => {
                return TelegramLaunchResult {
                    success: false,
                    message_ar: "البروكسي المحدد غير موجود.".to_string(),
                    message_en: "Selected proxy not found.".to_string(),
                    tg_link: String::new(),
                    fallback_web_link: String::new(),
                    official_download_url: "https://desktop.telegram.org/".to_string(),
                    is_telegram_installed: false,
                };
            }
        };

        // Remember as last selected proxy
        if let Ok(mut s) = self.settings.lock() {
            s.last_selected_proxy_id = Some(proxy.id.clone());
        }
        let _ = self.db.set_setting("last_selected_proxy_id", &proxy.id);

        let custom_path = self
            .settings
            .lock()
            .ok()
            .and_then(|s| s.custom_telegram_path.clone());
        self.launcher.launch_proxy(
            &proxy.server,
            proxy.port,
            &proxy.secret,
            custom_path.as_deref(),
        )
    }

    /// Calculates statistics for the dashboard
    pub fn get_stats(&self) -> ManagerStats {
        let proxies = self.db.get_all_proxies().unwrap_or_default();
        let total = proxies.len();
        let mut tcp_reachable = 0;
        let mut unreachable = 0;
        let mut timeout = 0;
        let mut dns_failed = 0;
        let mut unchecked = 0;
        let mut best_ping: Option<u64> = None;

        for p in &proxies {
            match p.status {
                ProxyStatus::TcpReachable => {
                    tcp_reachable += 1;
                    if let Some(ping) = p.ping_ms {
                        best_ping = Some(best_ping.map_or(ping, |current| current.min(ping)));
                    }
                }
                ProxyStatus::Unreachable => unreachable += 1,
                ProxyStatus::Timeout => timeout += 1,
                ProxyStatus::DnsFailed => dns_failed += 1,
                ProxyStatus::Unchecked | ProxyStatus::MtprotoUnverified => unchecked += 1,
            }
        }

        let last_update = self
            .db
            .get_setting("last_update_timestamp")
            .ok()
            .flatten()
            .unwrap_or_else(|| "لم يتم التحديث بعد".to_string());

        let is_testing = *self.is_testing.lock().unwrap();

        ManagerStats {
            total,
            tcp_reachable,
            unreachable,
            timeout,
            dns_failed,
            unchecked,
            best_ping_ms: best_ping,
            last_update,
            is_testing,
        }
    }
}
