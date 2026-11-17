use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct DiagnosticsLogger {
    log_path: PathBuf,
    recent_entries: Arc<Mutex<Vec<String>>>,
}

impl DiagnosticsLogger {
    pub fn new(app_dir: &std::path::Path) -> Self {
        let log_path = app_dir.join("MTProxyDesk_Diagnostics.log");
        Self {
            log_path,
            recent_entries: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Masks secrets in logs according to requirements
    pub fn mask_secret(secret: &str) -> String {
        let len = secret.len();
        if len <= 8 {
            "***MASKED***".to_string()
        } else {
            let start: String = secret.chars().take(4).collect();
            let end: String = secret.chars().skip(len - 4).collect();
            format!("{}...{} [MASKED]", start, end)
        }
    }

    pub fn log(&self, level: &str, category: &str, message: &str) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let line = format!("[{}] [{}] [{}] {}", now, level, category, message);

        println!("{}", line);

        if let Ok(mut entries) = self.recent_entries.lock() {
            if entries.len() >= 500 {
                entries.remove(0);
            }
            entries.push(line.clone());
        }

        if let Some(parent) = self.log_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            let _ = writeln!(file, "{}", line);
        }
    }

    pub fn get_recent_logs(&self) -> Vec<String> {
        self.recent_entries
            .lock()
            .map(|l| l.clone())
            .unwrap_or_default()
    }

    pub fn clear_logs(&self) {
        if let Ok(mut entries) = self.recent_entries.lock() {
            entries.clear();
        }
        let _ = fs::write(&self.log_path, "");
    }

    pub fn export_report(
        &self,
        total_proxies: usize,
        reachable_count: usize,
        last_update: &str,
    ) -> String {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S");
        let mut report = String::new();
        report.push_str("====================================================\n");
        report.push_str(" MTProxy Desk – Diagnostics & Health Report\n");
        report.push_str("====================================================\n");
        report.push_str(&format!("Generated at: {}\n", now));
        report.push_str("Application: MTProxy Desk v1.0.0\n");
        report.push_str("Operating System: Windows 64-bit\n");
        report.push_str(&format!("Log File Path: {}\n", self.log_path.display()));
        report.push_str(&format!("Last List Update: {}\n", last_update));
        report.push_str(&format!("Total Proxies in Local DB: {}\n", total_proxies));
        report.push_str(&format!("TCP Reachable Count: {}\n", reachable_count));
        report.push_str("----------------------------------------------------\n");
        report.push_str("Recent Log Entries (Secrets are masked for security):\n");
        report.push_str("----------------------------------------------------\n\n");

        for entry in self.get_recent_logs() {
            report.push_str(&entry);
            report.push('\n');
        }

        report.push_str("\n====================================================\n");
        report.push_str("End of Report\n");
        report
    }

    pub fn get_log_path(&self) -> PathBuf {
        self.log_path.clone()
    }
}
