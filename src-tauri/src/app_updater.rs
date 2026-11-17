use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use tauri::{AppHandle, Emitter};

pub const CURRENT_VERSION: &str = "1.0.0";
pub const GITHUB_REPO: &str = "moshraheem-sudo/MTProxy-Desk";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub has_update: bool,
    pub current_version: String,
    pub latest_version: String,
    pub release_name: String,
    pub release_notes: String,
    pub download_url: String,
    pub file_name: String,
    pub file_size_bytes: u64,
    pub file_size_mb: f64,
    pub release_html_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgressPayload {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub downloaded_mb: f64,
    pub total_mb: f64,
    pub percentage: f64,
    pub status: String,
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    html_url: String,
    #[serde(default)]
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    size: u64,
    browser_download_url: String,
}

/// Checks GitHub Releases API for a newer version
pub async fn check_for_updates() -> Result<UpdateCheckResult, String> {
    let client = reqwest::Client::builder()
        .user_agent("MTProxyDesk/1.0.0 (Windows NT)")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let api_url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );

    let res = client
        .get(&api_url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("Network request failed: {}", e))?;

    if res.status().as_u16() == 404 {
        // No releases published yet on the repository
        return Ok(UpdateCheckResult {
            has_update: false,
            current_version: CURRENT_VERSION.to_string(),
            latest_version: CURRENT_VERSION.to_string(),
            release_name: "No releases published yet".to_string(),
            release_notes: format!(
                "You are running v{}. Releases will appear at https://github.com/{}/releases",
                CURRENT_VERSION, GITHUB_REPO
            ),
            download_url: String::new(),
            file_name: String::new(),
            file_size_bytes: 0,
            file_size_mb: 0.0,
            release_html_url: format!("https://github.com/{}/releases", GITHUB_REPO),
        });
    }

    if !res.status().is_success() {
        return Err(format!("GitHub API returned status: {}", res.status()));
    }

    let release: GitHubRelease = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse GitHub release JSON: {}", e))?;

    let latest_tag = release.tag_name.trim_start_matches('v').trim();
    let is_newer = is_version_newer(CURRENT_VERSION, latest_tag);

    // Look for installer asset (.exe preferred, then .msi)
    let asset = release
        .assets
        .iter()
        .find(|a| a.name.ends_with(".exe") && a.name.contains("Setup"))
        .or_else(|| release.assets.iter().find(|a| a.name.ends_with(".exe")))
        .or_else(|| release.assets.iter().find(|a| a.name.ends_with(".msi")));

    let (download_url, file_name, file_size_bytes, file_size_mb) = match asset {
        Some(a) => {
            let mb = (a.size as f64) / (1024.0 * 1024.0);
            (
                a.browser_download_url.clone(),
                a.name.clone(),
                a.size,
                (mb * 100.0).round() / 100.0,
            )
        }
        None => (
            String::new(),
            String::new(),
            0,
            0.0,
        ),
    };

    Ok(UpdateCheckResult {
        has_update: is_newer,
        current_version: CURRENT_VERSION.to_string(),
        latest_version: latest_tag.to_string(),
        release_name: release.name.unwrap_or_else(|| release.tag_name.clone()),
        release_notes: release.body.unwrap_or_default(),
        download_url,
        file_name,
        file_size_bytes,
        file_size_mb,
        release_html_url: release.html_url,
    })
}

/// Downloads the update installer with live progress in MB, cleans up previous versions,
/// and executes the installer.
pub async fn download_and_install_update(
    app: AppHandle,
    download_url: String,
    file_name: String,
) -> Result<String, String> {
    if download_url.is_empty() {
        return Err("No download URL available for this update.".to_string());
    }

    let client = reqwest::Client::builder()
        .user_agent("MTProxyDesk/1.0.0 (Windows NT)")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Create a dedicated directory for update downloads
    let update_dir = std::env::temp_dir().join("mtproxy_desk_updates");
    
    // Always clean up previous update versions as requested
    if update_dir.exists() {
        let _ = fs::remove_dir_all(&update_dir);
    }
    fs::create_dir_all(&update_dir)
        .map_err(|e| format!("Failed to create update folder: {}", e))?;

    let installer_path = update_dir.join(&file_name);

    let mut res = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to download source: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("Download failed with HTTP status: {}", res.status()));
    }

    let total_bytes = res.content_length().unwrap_or(0);
    let total_mb = ((total_bytes as f64) / (1024.0 * 1024.0) * 100.0).round() / 100.0;

    let mut file = File::create(&installer_path)
        .map_err(|e| format!("Failed to create destination file: {}", e))?;

    let mut downloaded_bytes: u64 = 0;

    while let Some(chunk) = res.chunk().await.map_err(|e| format!("Download stream error: {}", e))? {
        file.write_all(&chunk)
            .map_err(|e| format!("Failed to write chunk to disk: {}", e))?;

        downloaded_bytes += chunk.len() as u64;

        let downloaded_mb = ((downloaded_bytes as f64) / (1024.0 * 1024.0) * 100.0).round() / 100.0;
        let percentage = if total_bytes > 0 {
            ((downloaded_bytes as f64 / total_bytes as f64) * 100.0 * 10.0).round() / 10.0
        } else {
            0.0
        };

        let payload = DownloadProgressPayload {
            downloaded_bytes,
            total_bytes,
            downloaded_mb,
            total_mb,
            percentage,
            status: "downloading".to_string(),
        };

        let _ = app.emit("app-update-progress", &payload);
    }

    let finish_payload = DownloadProgressPayload {
        downloaded_bytes,
        total_bytes,
        downloaded_mb: total_mb,
        total_mb,
        percentage: 100.0,
        status: "completed".to_string(),
    };
    let _ = app.emit("app-update-progress", &finish_payload);

    // Launch the downloaded installer cleanly and exit current application
    let exe_str = installer_path.to_string_lossy().to_string();

    std::process::Command::new(&installer_path)
        .spawn()
        .map_err(|e| format!("Failed to launch installer ({}): {}", exe_str, e))?;

    // Exit application cleanly so installer can replace files without locks
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        std::process::exit(0);
    });

    Ok(format!("Installer launched: {}", exe_str))
}

/// Simple semantic version comparator
fn is_version_newer(current: &str, candidate: &str) -> bool {
    let parse_parts = |s: &str| -> Vec<u32> {
        s.split('.')
            .map(|p| p.chars().take_while(|c| c.is_ascii_digit()).collect::<String>())
            .filter_map(|p| p.parse::<u32>().ok())
            .collect()
    };

    let curr_parts = parse_parts(current);
    let cand_parts = parse_parts(candidate);

    for (c, cand) in curr_parts.iter().zip(cand_parts.iter()) {
        if cand > c {
            return true;
        } else if cand < c {
            return false;
        }
    }

    cand_parts.len() > curr_parts.len()
}
