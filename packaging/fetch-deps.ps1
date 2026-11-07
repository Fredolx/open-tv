# Puts mpv, ffmpeg and yt-dlp into packaging\deps for the Windows installer.
#
#   .\packaging\fetch-deps.ps1             download the latest builds (what CI does)
#   .\packaging\fetch-deps.ps1 -FromFredTV  copy them from an installed Fred TV instead
param([switch]$FromFredTV)

$ErrorActionPreference = 'Stop'
$deps = Join-Path $PSScriptRoot 'deps'
New-Item -ItemType Directory -Force -Path $deps | Out-Null

if ($FromFredTV) {
    $installed = 'C:\Program Files\Fred TV\deps'
    foreach ($file in 'mpv.exe', 'vulkan-1.dll', 'ffmpeg.exe', 'yt-dlp.exe') {
        Copy-Item (Join-Path $installed $file) $deps -Force
    }
    Get-ChildItem $deps
    return
}

function Get-WithRetry([string]$Uri, [string]$OutFile) {
    for ($i = 1; $i -le 3; $i++) {
        try {
            Invoke-WebRequest -UserAgent 'Wget' -Uri $Uri -OutFile $OutFile
            return
        }
        catch {
            if ($i -eq 3) { throw }
            Write-Host "Download failed ($_), retrying in 5 seconds..."
            Start-Sleep -Seconds 5
        }
    }
}

$work = Join-Path ([System.IO.Path]::GetTempPath()) "lily-tv-deps-$PID"
New-Item -ItemType Directory -Force -Path $work | Out-Null
try {
    # mpv: latest successful build of mpv's own CI
    $page = Invoke-WebRequest -Uri 'https://nightly.link/mpv-player/mpv/workflows/build/master' -UseBasicParsing
    $mpvUrl = $page.Links.Href | Where-Object { $_ -match 'x86_64-pc-windows-msvc\.zip$' } | Select-Object -First 1
    if (-not $mpvUrl) { throw 'Could not find the mpv Windows build on nightly.link' }
    Get-WithRetry $mpvUrl "$work\mpv.zip"
    Expand-Archive "$work\mpv.zip" "$work\mpv" -Force
    foreach ($file in 'mpv.exe', 'vulkan-1.dll') {
        Get-ChildItem "$work\mpv" -Recurse -Filter $file | Select-Object -First 1 | Copy-Item -Destination $deps -Force
    }

    # ffmpeg: BtbN's GPL build (includes libzmq, x264 and x265)
    Get-WithRetry 'https://github.com/BtbN/FFmpeg-Builds/releases/latest/download/ffmpeg-master-latest-win64-gpl.zip' "$work\ffmpeg.zip"
    Expand-Archive "$work\ffmpeg.zip" "$work\ffmpeg" -Force
    Get-ChildItem "$work\ffmpeg" -Recurse -Filter 'ffmpeg.exe' | Select-Object -First 1 | Copy-Item -Destination $deps -Force

    Get-WithRetry 'https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe' (Join-Path $deps 'yt-dlp.exe')
}
finally {
    Remove-Item $work -Recurse -Force -ErrorAction SilentlyContinue
}
Get-ChildItem $deps
