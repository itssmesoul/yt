# Installs yt-dlp, ffmpeg and deno for Swiss Kit.
# Tries winget first, then Scoop. Retries when a download drops.
$ErrorActionPreference = 'Continue'
$ProgressPreference = 'SilentlyContinue'

$tools = @(
    @{ Name = 'yt-dlp'; Winget = 'yt-dlp.yt-dlp'; Scoop = 'yt-dlp' },
    @{ Name = 'ffmpeg'; Winget = 'Gyan.FFmpeg';   Scoop = 'ffmpeg' },
    @{ Name = 'deno';   Winget = 'DenoLand.Deno'; Scoop = 'deno' }
)

function Have($name) { return [bool](Get-Command $name -ErrorAction SilentlyContinue) }

function Refresh-Path {
    $machine = [Environment]::GetEnvironmentVariable('Path', 'Machine')
    $user = [Environment]::GetEnvironmentVariable('Path', 'User')
    $env:Path = "$machine;$user;$env:USERPROFILE\scoop\shims"
}

function Is-Admin {
    $id = [Security.Principal.WindowsIdentity]::GetCurrent()
    return ([Security.Principal.WindowsPrincipal]$id).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Missing { return @($tools | Where-Object { -not (Have $_.Name) }) }

function Say($text) { Write-Host ''; Write-Host "== $text" -ForegroundColor Cyan }

Refresh-Path
Say 'Checking what is already installed'
foreach ($t in $tools) {
    if (Have $t.Name) { Write-Host "  $($t.Name): found" -ForegroundColor Green }
    else { Write-Host "  $($t.Name): missing" -ForegroundColor Yellow }
}

if ((Missing).Count -eq 0) {
    Say 'Everything is already installed. Nothing to do.'
    Write-Host 'Hersey zaten kurulu.'
    exit 0
}

# --- Step 1: winget (built into most Windows 10/11 PCs) ---
if (Have 'winget') {
    Say 'Installing with winget'
    foreach ($t in (Missing)) {
        for ($try = 1; $try -le 3; $try++) {
            Write-Host "  $($t.Name) (attempt $try of 3)"
            winget install --id $t.Winget -e --accept-source-agreements --accept-package-agreements
            Refresh-Path
            if (Have $t.Name) { break }
            Start-Sleep -Seconds 3
        }
    }
}
else {
    Write-Host ''
    Write-Host 'winget was not found on this PC, so Scoop will be used instead.'
}

# --- Step 2: Scoop for anything still missing ---
if ((Missing).Count -gt 0) {
    Say 'Installing with Scoop'
    if (-not (Have 'scoop')) {
        try {
            Set-ExecutionPolicy -Scope CurrentUser RemoteSigned -Force
            if (Is-Admin) {
                # Scoop refuses to install from an Administrator window unless told it is OK.
                Invoke-Expression "& {$(Invoke-RestMethod get.scoop.sh)} -RunAsAdmin"
            }
            else {
                Invoke-RestMethod get.scoop.sh | Invoke-Expression
            }
        }
        catch {
            Write-Host "  Could not install Scoop: $($_.Exception.Message)" -ForegroundColor Red
        }
        Refresh-Path
    }
    if (Have 'scoop') {
        foreach ($t in (Missing)) {
            for ($try = 1; $try -le 3; $try++) {
                Write-Host "  $($t.Name) (attempt $try of 3)"
                scoop install $t.Scoop
                Refresh-Path
                if (Have $t.Name) { break }
                Start-Sleep -Seconds 3
            }
        }
    }
}

# --- Result ---
Refresh-Path
Say 'Result'
$bad = @()
foreach ($t in $tools) {
    if (Have $t.Name) { Write-Host "  $($t.Name): OK" -ForegroundColor Green }
    else { Write-Host "  $($t.Name): NOT INSTALLED" -ForegroundColor Red; $bad += $t.Name }
}
Write-Host ''
if ($bad.Count -eq 0) {
    Write-Host 'All done! Now double-click "Start Swiss Kit.bat".' -ForegroundColor Green
    Write-Host 'Bitti! Simdi "Start Swiss Kit.bat" dosyasina cift tiklayin.'
    Write-Host 'If Swiss Kit still says a tool is missing, restart your PC once.'
    exit 0
}
else {
    Write-Host ('Could not install: ' + ($bad -join ', ')) -ForegroundColor Red
    Write-Host 'This is usually the internet connection. Run this file again, or try another network or a VPN for the install.'
    Write-Host 'Kurulamadi. Genelde internet baglantisindan olur. Dosyayi tekrar calistirin ya da baska bir ag / VPN deneyin.'
    exit 1
}
