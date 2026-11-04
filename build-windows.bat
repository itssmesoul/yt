@echo off
setlocal
cd /d "%~dp0"
echo [1/3] Derleniyor (ilk seferde birkac dakika surer)...
cargo build --release -p swiss-kit-app || goto :fail
if not exist dist mkdir dist
copy /y target\release\swiss-kit-app.exe "dist\Swiss Kit.exe" >nul

echo [2/3] yt-dlp indiriliyor...
if not exist dist\yt-dlp.exe curl -L -o dist\yt-dlp.exe https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe || goto :fail

echo [3/3] ffmpeg indiriliyor...
if not exist dist\ffmpeg.exe (
  curl -L -o dist\ffmpeg.zip https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip || goto :fail
  powershell -NoProfile -Command "Expand-Archive -Force dist\ffmpeg.zip dist\_ff; Copy-Item (Get-ChildItem dist\_ff -Recurse -Filter ffmpeg.exe | Select-Object -First 1).FullName dist\ffmpeg.exe; Remove-Item -Recurse -Force dist\_ff, dist\ffmpeg.zip" || goto :fail
)
echo.
echo Hazir: dist\Swiss Kit.exe  (yt-dlp.exe ve ffmpeg.exe ayni klasorde)
exit /b 0
:fail
echo HATA: yukaridaki mesaja bakin.
exit /b 1
