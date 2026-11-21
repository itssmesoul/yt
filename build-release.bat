@echo off
title Build Swiss Kit
cd /d "%~dp0"
echo Building Swiss Kit. The first time takes a few minutes...
cargo build --release -p swiss-kit-app
if errorlevel 1 goto fail
copy /y "target\release\swiss-kit-app.exe" "dist\swiss-kit-app.exe" >nul
if exist "Swiss-Kit-Windows.zip" del "Swiss-Kit-Windows.zip"
powershell -NoProfile -Command "Compress-Archive -Path 'dist\*' -DestinationPath 'Swiss-Kit-Windows.zip'"
echo.
echo Done. Send this file to your friends: Swiss-Kit-Windows.zip
pause
exit /b 0
:fail
echo.
echo The build failed. Scroll up to see why.
pause
exit /b 1
