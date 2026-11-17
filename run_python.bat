@echo off
chcp 65001 >nul
echo ====================================================
echo  MTProxy Desk - Telegram Proxy Manager (Python)
echo ====================================================
echo تشغيل تطبيق MTProxy Desk (نسخة Python)...

where python >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    python "%~dp0mtproxy_desk.py"
    exit /b
)

if exist "%LOCALAPPDATA%\Programs\Python\Python312\python.exe" (
    "%LOCALAPPDATA%\Programs\Python\Python312\python.exe" "%~dp0mtproxy_desk.py"
    exit /b
)

if exist "%LOCALAPPDATA%\Programs\Python\Python311\python.exe" (
    "%LOCALAPPDATA%\Programs\Python\Python311\python.exe" "%~dp0mtproxy_desk.py"
    exit /b
)

echo لم يتم العثور على Python مثبت. يرجى تشغيل مثبت Windows (MTProxy_Desk_Setup_v1.0.0.exe).
pause
