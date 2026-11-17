@echo off
chcp 65001 >nul
cd /d "%~dp0"
echo ====================================================
echo  MTProxy Desk v1.0.2 – Dev: Mohammed Shaheed Al-Aajeebi
echo ====================================================
echo جاري فحص البيئة والمكتبات...

:: Ensure WebView2Loader.dll is in place
if not exist "%~dp0WebView2Loader.dll" (
    if exist "%~dp0src-tauri\target\release\WebView2Loader.dll" (
        copy /y "%~dp0src-tauri\target\release\WebView2Loader.dll" "%~dp0WebView2Loader.dll" >nul
    )
)

:: Clear any stale locked WebView2 processes if needed
powershell -NoProfile -Command "Get-CimInstance Win32_Process -Filter \"Name = 'msedgewebview2.exe'\" | Where-Object { $_.CommandLine -like '*mtproxydesk*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }" >nul 2>&1

echo تشغيل التطبيق الأصلي (Rust + Tauri 2)...
start "" "%~dp0mtproxy-desk.exe"
exit
