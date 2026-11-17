# MTProxy Desk – Telegram Proxy Manager

**Version:** 1.0.1  
**Developer:** Mohammed Shaheed Al-Aajeebi (محمد شهيد الاعاجيبي)  
**Target Platform:** Windows 10 / Windows 11 (64-Bit)  
**Core Engine:** Rust Stable + Tokio + Tauri 2  
**Interface:** Ultra-Compact, Lightweight & High-Performance (English Default with Arabic toggle)  
**Auto-Sync Interval:** Every 30 minutes (automatic background sync + manual instant update button)

---

## 📌 About MTProxy Desk

**MTProxy Desk** is a dedicated, production-grade Windows desktop application for managing, verifying, benchmarking, and directly launching verified Telegram MTProto proxies. Built with **Rust** as its high-performance network engine, Tokio for concurrent TCP testing, and SQLite for robust offline persistence.

Designed by **Mohammed Shaheed Al-Aajeebi**, the application features an ultra-compact UI optimized to run smoothly and rapidly across all types of computers and laptops without memory or CPU overhead.

---

## 🚀 Key Features

1. **Auto-Sync Every 30 Minutes & Manual Instant Update:**
   - Automatically polls the latest live proxy feeds from GitHub every 30 minutes.
   - Includes a one-click manual update button to refresh proxies on demand.
   - Fully offline-capable: caches verified proxies in a local SQLite database with zero downtime.

2. **In-App GitHub Release Updater (v1.0.1):**
   - Real-time version check directly with GitHub Releases.
   - Only offers downloads when a newer version exists, preventing redundant downloads.
   - Live Megabytes progress indicator (`MB downloaded / MB total`).
   - Clean download management: removes previous temporary installer builds automatically.
   - Dedicated **Install Update Now (تثبيت التحديث الآن)** button on completion with smooth Windows UAC elevation.

3. **Ultra-Compact & Fast Desktop Interface:**
   - Small footprint window (840x580) tailored for any screen resolution.
   - Quick Connect and Share actions directly accessible before the server column.
   - Zero horizontal overflow.
   - English by default with an instant Arabic language toggle.

4. **Right-Click Context Menu:**
   - Right-click any server row to Connect in Telegram, Retest Ping, Bookmark as Favorite, Share, or view Full Details.

5. **High-Performance Rust Validation Engine:**
   - Tokio-powered concurrent TCP latency and DNS verification.
   - Distinguishes TCP reachability from MTProto handshake with complete transparency.
   - Live metrics: Total proxies, Reachable, Dead, and Best Ping in milliseconds.

6. **Instant 1-Click Launch in Telegram Desktop:**
   - Generates secure `tg://proxy?server=...&port=...&secret=...` links.
   - Seamlessly passes connections to Telegram Desktop with web fallback `https://t.me/proxy`.

7. **Integrated Share & QR Studio:**
   - Generates high-contrast QR codes for instant mobile camera connection.
   - Single-click copying for Server, Port, Secret, and Full Protocol Link.
   - Export proxy configuration as `.txt`.

8. **System Tray & Autostart Integration:**
   - Minimizes to Windows System Tray on close to keep testing in the background.
   - Optional automatic start on Windows boot.

---

## 📦 Distribution Packages

| File | Type | Description |
| :--- | :--- | :--- |
| **[MTProxy_Desk_Setup_v1.0.1.exe](file:///c:/Users/gahzi/Desktop/mtproto/MTProxy_Desk_Setup_v1.0.1.exe)** | Windows Setup Installer | Official NSIS installer. Configures Start Menu shortcuts and all required runtime DLLs. |
| **[MTProxy_Desk_Setup_v1.0.1.msi](file:///c:/Users/gahzi/Desktop/mtproto/MTProxy_Desk_Setup_v1.0.1.msi)** | Windows MSI Package | Enterprise MSI installer for standard Windows deployment. |
| **[MTProxy_Desk_Portable_v1.0.1.zip](file:///c:/Users/gahzi/Desktop/mtproto/MTProxy_Desk_Portable_v1.0.1.zip)** | Portable ZIP Archive | Zero-install standalone archive with `mtproxy-desk.exe` and `WebView2Loader.dll`. |
| **[run_desktop.bat](file:///c:/Users/gahzi/Desktop/mtproto/run_desktop.bat)** | 1-Click Launcher | Automatically verifies libraries and runs `mtproxy-desk.exe`. |
| **[mtproxy_desk.py](file:///c:/Users/gahzi/Desktop/mtproto/mtproxy_desk.py)** | Python Standalone Edition | Full GUI application in Python with Tkinter, SQLite, and 30-min auto-update. |
| **[run_python.bat](file:///c:/Users/gahzi/Desktop/mtproto/run_python.bat)** | Python Launcher | Runs the Python edition using local Python 3.12. |
| **[update_proxies.py](file:///c:/Users/gahzi/Desktop/mtproto/update_proxies.py)** | CLI Direct Updater | Terminal updater that tests all proxies from raw GitHub URL. |

---

## 👨‍💻 Developer & Credits

- **Developer:** Mohammed Shaheed Al-Aajeebi (محمد شهيد الاعاجيبي)
- **Application Version:** 1.0.1
- **GitHub Repository:** [moshraheem-sudo/MTProxy-Desk](https://github.com/moshraheem-sudo/MTProxy-Desk)
- **Primary Proxy Source:** [SoliSpirit/mtproto](https://github.com/SoliSpirit/mtproto)
