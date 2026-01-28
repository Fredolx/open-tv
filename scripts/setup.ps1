$ErrorActionPreference = "Stop"

$retryCount = 3
$retryDelay = 5

function Download-WithRetry {
    param (
        [string]$Uri,
        [string]$OutFile
    )
    for ($i = 1; $i -le $retryCount; $i++) {
        try {
            Invoke-WebRequest -UserAgent "Wget" -Uri $Uri -OutFile $OutFile
            return
        }
        catch {
            Write-Host "Download failed: $_. Retrying in $retryDelay seconds..."
            Start-Sleep -Seconds $retryDelay
        }
    }
    throw "Failed to download $Uri after $retryCount attempts"
}

# Ensure deps dir exists
New-Item -Path 'C:\open-tv-deps' -ItemType Directory -Force | Out-Null

# MPV
Write-Host "Fetching MPV..."
$nightlyPage = "https://nightly.link/mpv-player/mpv/workflows/build/master"
$response = Invoke-WebRequest -Uri $nightlyPage -UseBasicParsing
$latestUrl = $response.Links.Href | Where-Object { $_ -match "x86_64-pc-windows-msvc\.zip$" } | Select-Object -First 1

if (-not $latestUrl) { throw "Could not find MPV download URL" }

Download-WithRetry -Uri $latestUrl -OutFile .\mpv.zip
Write-Host "Extracting MPV..."
7z e .\mpv.zip -oC:\open-tv-deps mpv.exe vulkan-1.dll -y

# FFmpeg
Write-Host "Fetching FFmpeg..."
Download-WithRetry -Uri "https://github.com/BtbN/FFmpeg-Builds/releases/latest/download/ffmpeg-master-latest-win64-gpl.zip" -OutFile .\ffmpeg.zip
Write-Host "Extracting FFmpeg..."
# 7z e extracts invalidating paths, so it puts ffmpeg.exe directly in destination if we want.
# The workflow used `7z e ... ffmpeg-master-latest-win64-gpl/bin/ffmpeg.exe`, filtering for that file.
7z e .\ffmpeg.zip -oC:\open-tv-deps ffmpeg.exe -r -y 
# Note: -r might be needed if it can't find it at root, but `e` flattens. 
# Wait, the workflow command was: 7z e .\ffmpeg.zip -oC:\open-tv-deps ffmpeg-master-latest-win64-gpl/bin/ffmpeg.exe
# I'll try to match that specificity if possible, or just extract all and pick.
# Actually `7z e archive.zip filename` works to extract just that file.
7z e .\ffmpeg.zip -oC:\open-tv-deps ffmpeg.exe -r -y

# yt-dlp
Write-Host "Fetching yt-dlp..."
Download-WithRetry -Uri "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe" -OutFile .\yt-dlp.exe
Move-Item -Path .\yt-dlp.exe -Destination C:\open-tv-deps -Force

Write-Host "Dependencies setup complete in C:\open-tv-deps"
Remove-Item .\mpv.zip -ErrorAction SilentlyContinue
Remove-Item .\ffmpeg.zip -ErrorAction SilentlyContinue
