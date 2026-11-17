@echo off
chcp 65001 >nul
cd /d "%~dp0"
echo ====================================================
echo  MTProxy Desk v1.0.0 – Dev: Mohammed Shaheed Al-Aajeebi
echo ====================================================
echo التحقق من بيئة التشغيل والمكتبات...

:: Ensure WebView2Loader.dll exists
if not exist "%~dp0WebView2Loader.dll" (
    if exist "%~dp0src-tauri\target\release\WebView2Loader.dll" (
        copy /y "%~dp0src-tauri\target\release\WebView2Loader.dll" "%~dp0WebView2Loader.dll" >nul
    )
)

echo تشغيل التطبيق الأصلي (Rust + Tauri 2)...
start "" "%~dp0mtproxy-desk.exe"
exit
