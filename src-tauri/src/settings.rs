use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub auto_update: bool,
    #[serde(default = "default_interval_minutes")]
    pub update_interval_minutes: u32,
    #[serde(default)]
    pub update_interval_hours: u32,
    pub autostart: bool,
    pub minimize_to_tray: bool,
    pub auto_check: bool,
    pub check_timeout_secs: u64,
    pub max_concurrency: usize,
    pub language: String,
    pub theme: String,
    pub custom_telegram_path: Option<String>,
    pub last_selected_proxy_id: Option<String>,
    pub last_update_timestamp: Option<String>,
}

fn default_interval_minutes() -> u32 {
    30
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_update: true,
            update_interval_minutes: 30,
            update_interval_hours: 0,
            autostart: false,
            minimize_to_tray: true,
            auto_check: true,
            check_timeout_secs: 4,
            max_concurrency: 30,
            language: "en".to_string(), // English default as requested
            theme: "dark".to_string(),
            custom_telegram_path: None,
            last_selected_proxy_id: None,
            last_update_timestamp: None,
        }
    }
}

/// Helper to configure Windows Run Registry key for Autostart
pub fn set_windows_autostart(enable: bool) -> Result<(), String> {
    let app_name = "MTProxyDesk";
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?;
    let exe_str = exe_path.to_string_lossy();

    if enable {
        let cmd = format!(
            r#"reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v "{}" /t REG_SZ /d "\"{}\" --minimized" /f"#,
            app_name, exe_str
        );
        let output = std::process::Command::new("cmd")
            .args(["/C", &cmd])
            .output()
            .map_err(|e| format!("Failed to execute reg command: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Registry add failed: {}", err));
        }
    } else {
        let cmd = format!(
            r#"reg delete "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v "{}" /f"#,
            app_name
        );
        let _ = std::process::Command::new("cmd")
            .args(["/C", &cmd])
            .output();
    }

    Ok(())
}
