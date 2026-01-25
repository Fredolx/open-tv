@echo off
setlocal
echo ===================================================
echo     Open-TV Testing Launcher
echo ===================================================
echo.

echo [1/4] Closing any running instances...
taskkill /F /IM open-tv.exe >nul 2>&1
taskkill /F /IM cargo.exe >nul 2>&1
:: Be careful with node, only kill if necessary or specific to this app? 
:: For now, we rely on tauri to manage its node process, but if previous one hung...
:: We will skip killing node globally to avoid affecting other work, 
:: but we will kill the 'tauri' specific processes if we could identify them.
:: For now, just cargo and open-tv is the safest 'hard' reset.
echo       (Processes terminated)

echo.
echo [2/4] Verifying dependencies...
if not exist "node_modules" (
    echo       Node modules missing. Installing...
    call npm install
) else (
    echo       Dependencies found.
)

echo.
echo [3/4] Cleaning previous build artifacts (partial)...
:: Optionally clear target/debug/open-tv.exe to force link? 
:: No, cargo handles this well.

echo.
echo [4/4] Starting Development Server...
echo       This will compile the Rust backend and start the Angular frontend.
echo       Please wait for the window to appear.
echo.

call npm run tauri dev

if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Application failed to start.
    echo         Check the output above for details.
    pause
    exit /b %errorlevel%
)
endlocal
