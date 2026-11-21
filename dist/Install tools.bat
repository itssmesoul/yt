@echo off
title Install Swiss Kit tools
cd /d "%~dp0"
if not exist "install-tools.ps1" (
  echo install-tools.ps1 is missing. Make sure you extracted the whole zip first.
  pause
  exit /b 1
)
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0install-tools.ps1"
echo.
pause
