@echo off
title Swiss Kit
cd /d "%~dp0"
if not exist "swiss-kit-app.exe" (
  echo swiss-kit-app.exe is missing. Make sure you extracted the whole zip first.
  pause
  exit /b 1
)
echo Starting Swiss Kit... your browser will open in a moment.
echo Keep this window open while you use the app. Close it to quit.
echo.
swiss-kit-app.exe
echo.
echo Swiss Kit stopped.
pause
