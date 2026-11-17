use crate::diagnostics::DiagnosticsLogger;
use crate::telegram_links::generate_links;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramLaunchResult {
    pub success: bool,
    pub message_ar: String,
    pub message_en: String,
    pub tg_link: String,
    pub fallback_web_link: String,
    pub official_download_url: String,
    pub is_telegram_installed: bool,
}

pub struct TelegramLauncher {
    logger: DiagnosticsLogger,
}

impl TelegramLauncher {
    pub fn new(logger: DiagnosticsLogger) -> Self {
        Self { logger }
    }

    /// Checks if Telegram Desktop is registered in Windows Registry or in typical directories.
    pub fn is_telegram_installed(&self) -> bool {
        // Method 1: Check HKCU or HKLM registry for tg URI scheme handler
        let reg_paths = [
            r"Software\Classes\tg\shell\open\command",
            r"Software\Clients\StartMenuInternet\Telegram.exe",
        ];

        // We can inspect via query to reg or known AppData path
        if let Ok(appdata) = std::env::var("APPDATA") {
            let tdesktop_exe = PathBuf::from(&appdata)
                .join("Telegram Desktop")
                .join("Telegram.exe");
            if tdesktop_exe.exists() {
                return true;
            }
        }

        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            let tdesktop_exe = PathBuf::from(&local_appdata)
                .join("Programs")
                .join("Telegram Desktop")
                .join("Telegram.exe");
            if tdesktop_exe.exists() {
                return true;
            }
        }

        // Check Windows Program Files
        let pf_paths = [
            r"C:\Program Files\Telegram Desktop\Telegram.exe",
            r"C:\Program Files (x86)\Telegram Desktop\Telegram.exe",
        ];
        for path in &pf_paths {
            if PathBuf::from(path).exists() {
                return true;
            }
        }

        // Check registry key via powershell/reg query if files not found in standard paths
        let output = std::process::Command::new("reg")
            .args(["query", reg_paths[0]])
            .output();

        if let Ok(out) = output
            && out.status.success()
        {
            return true;
        }

        false
    }

    /// Safely launches the proxy URL using the operating system's registered URI handler.
    pub fn launch_proxy(
        &self,
        server: &str,
        port: u16,
        secret: &str,
        custom_path: Option<&str>,
    ) -> TelegramLaunchResult {
        let links = match generate_links(server, port, secret) {
            Ok(l) => l,
            Err(e) => {
                self.logger.log(
                    "ERROR",
                    "LAUNCHER",
                    &format!("Failed to build proxy link: {}", e),
                );
                return TelegramLaunchResult {
                    success: false,
                    message_ar: format!("فشل إنشاء رابط Telegram: {}", e),
                    message_en: format!("Failed to generate Telegram link: {}", e),
                    tg_link: String::new(),
                    fallback_web_link: String::new(),
                    official_download_url: "https://desktop.telegram.org/".to_string(),
                    is_telegram_installed: false,
                };
            }
        };

        let masked_sec = DiagnosticsLogger::mask_secret(secret);
        self.logger.log(
            "INFO",
            "LAUNCHER",
            &format!(
                "Attempting to connect via Telegram with server: {}:{}, secret: {}",
                server, port, masked_sec
            ),
        );

        let is_installed = self.is_telegram_installed();

        // If custom path is provided and exists, launch it directly with the tg:// URL
        if let Some(cp) = custom_path {
            let custom_path_buf = PathBuf::from(cp);
            if custom_path_buf.exists() {
                self.logger.log(
                    "INFO",
                    "LAUNCHER",
                    &format!("Using custom Telegram binary at: {}", cp),
                );
                let child = std::process::Command::new(custom_path_buf)
                    .arg(&links.tg_link)
                    .spawn();

                if child.is_ok() {
                    return TelegramLaunchResult {
                        success: true,
                        message_ar: "تم فتح Telegram Desktop بنجاح. يرجى تأكيد إضافة وتفعيل البروكسي من شاشة تليجرام.".to_string(),
                        message_en: "Telegram Desktop opened successfully. Please confirm adding the proxy inside Telegram.".to_string(),
                        tg_link: links.tg_link,
                        fallback_web_link: links.https_link,
                        official_download_url: "https://desktop.telegram.org/".to_string(),
                        is_telegram_installed: true,
                    };
                }
            }
        }

        // Standard OS launch for tg:// URI handler
        match open::that_detached(&links.tg_link) {
            Ok(_) => {
                self.logger.log(
                    "INFO",
                    "LAUNCHER",
                    "Successfully handed off tg:// URI to Windows shell.",
                );
                TelegramLaunchResult {
                    success: true,
                    message_ar: "تم إرسال رابط البروكسي إلى نظام التشغيل لتشغيل Telegram Desktop. يرجى الضغط على «تمكين» أو «Enable Proxy» داخل نافذة Telegram لتأكيد الاستخدام.".to_string(),
                    message_en: "Proxy URI sent to OS. Please click 'Enable Proxy' in Telegram to confirm connection.".to_string(),
                    tg_link: links.tg_link,
                    fallback_web_link: links.https_link,
                    official_download_url: "https://desktop.telegram.org/".to_string(),
                    is_telegram_installed: is_installed,
                }
            }
            Err(err) => {
                self.logger.log(
                    "WARN",
                    "LAUNCHER",
                    &format!(
                        "Failed to open tg:// URI: {}. Trying fallback https://",
                        err
                    ),
                );
                // Fallback to opening web link in browser
                let _ = open::that_detached(&links.https_link);

                TelegramLaunchResult {
                    success: is_installed,
                    message_ar: if is_installed {
                        "تم فتح الرابط البديل في المتصفح للاتصال بـ Telegram Desktop.".to_string()
                    } else {
                        "لم يتم العثور على تطبيق Telegram Desktop مثبت على الجهاز. يمكنك تحميله من الموقع الرسمي أو نسخ الرابط يدوياً.".to_string()
                    },
                    message_en: if is_installed {
                        "Opened fallback proxy link in browser to connect to Telegram Desktop."
                            .to_string()
                    } else {
                        "Telegram Desktop was not detected on this system. You can download it from the official site or copy the proxy link.".to_string()
                    },
                    tg_link: links.tg_link,
                    fallback_web_link: links.https_link,
                    official_download_url: "https://desktop.telegram.org/".to_string(),
                    is_telegram_installed: is_installed,
                }
            }
        }
    }
}
