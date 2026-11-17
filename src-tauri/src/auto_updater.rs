use crate::proxy_manager::ProxyManager;
use std::sync::Arc;
use std::time::Duration;

pub struct AutoUpdater {
    manager: Arc<ProxyManager>,
}

impl AutoUpdater {
    pub fn new(manager: Arc<ProxyManager>) -> Self {
        Self { manager }
    }

    /// Spawns the background updater loop
    pub fn start_background_task(&self) {
        let manager = self.manager.clone();

        tauri::async_runtime::spawn(async move {
            manager.logger.log(
                "INFO",
                "AUTO_UPDATER",
                "Background auto-updater service started (30-minute interval default).",
            );

            // Wait 2 seconds after startup before initial refresh if DB is empty
            tokio::time::sleep(Duration::from_secs(2)).await;

            let should_initial_fetch = {
                let proxies = manager.db.get_all_proxies().unwrap_or_default();
                proxies.is_empty()
            };

            if should_initial_fetch {
                manager
                    .logger
                    .log("INFO", "AUTO_UPDATER", "Initial startup fetch triggered.");
                let _ = manager.refresh_proxies().await;
                let auto_check = manager
                    .settings
                    .lock()
                    .map(|s| s.auto_check)
                    .unwrap_or(true);
                if auto_check {
                    let _ = manager.validate_all().await;
                }
            }

            loop {
                // Determine update interval from settings (default 30 minutes)
                let (auto_update, interval_mins, auto_check) = {
                    let s = manager.settings.lock().unwrap();
                    let mins = if s.update_interval_minutes > 0 {
                        s.update_interval_minutes
                    } else if s.update_interval_hours > 0 {
                        s.update_interval_hours * 60
                    } else {
                        30
                    };
                    (s.auto_update, mins.max(5), s.auto_check)
                };

                let interval_secs = (interval_mins as u64) * 60;
                tokio::time::sleep(Duration::from_secs(interval_secs)).await;

                if auto_update {
                    manager.logger.log(
                        "INFO",
                        "AUTO_UPDATER",
                        &format!(
                            "Periodic auto-update triggered (every {} minutes).",
                            interval_mins
                        ),
                    );

                    match manager.refresh_proxies().await {
                        Ok(proxies) => {
                            manager.logger.log(
                                "INFO",
                                "AUTO_UPDATER",
                                &format!(
                                    "Periodic auto-update successful. Loaded {} proxies.",
                                    proxies.len()
                                ),
                            );

                            if auto_check {
                                let _ = manager.validate_all().await;
                            }
                        }
                        Err(err) => {
                            manager.logger.log(
                                "WARN",
                                "AUTO_UPDATER",
                                &format!("Periodic auto-update failed: {}. Retrying later.", err),
                            );
                        }
                    }
                }
            }
        });
    }
}
