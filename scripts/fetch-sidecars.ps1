param(
  [string]$Destination = "target/sidecar"
)

$ErrorActionPreference = "Stop"

Write-Host "Fetching Chromium headless shell and FFmpeg sidecars..."

New-Item -ItemType Directory -Force -Path "$Destination/chromium" | Out-Null
New-Item -ItemType Directory -Force -Path "$Destination/ffmpeg" | Out-Null

$tmpRoot = Join-Path $env:TEMP ("browser-stream-sidecars-" + [guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $tmpRoot | Out-Null

try {
  $manifestUrl = "https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions-with-downloads.json"
  $manifest = Invoke-RestMethod -Uri $manifestUrl

  $chromiumEntry = $manifest.channels.Stable.downloads.'chrome-headless-shell' |
    Where-Object { $_.platform -eq 'win64' } |
    Select-Object -First 1

  if (-not $chromiumEntry) {
    throw "Could not resolve chrome-headless-shell download URL for win64"
  }

  $chromiumZip = Join-Path $tmpRoot "chromium.zip"
  Invoke-WebRequest -Uri $chromiumEntry.url -OutFile $chromiumZip
  Expand-Archive -Path $chromiumZip -DestinationPath (Join-Path $tmpRoot "chromium") -Force

  $chromiumExe = Get-ChildItem -Path (Join-Path $tmpRoot "chromium") -Recurse -File -Filter "chrome-headless-shell.exe" | Select-Object -First 1
  if (-not $chromiumExe) {
    throw "Could not find chrome-headless-shell.exe in downloaded Chromium archive"
  }

  $chromiumRoot = $chromiumExe.Directory.FullName
  Copy-Item -Path (Join-Path $chromiumRoot '*') -Destination (Join-Path $Destination "chromium") -Recurse -Force
  Copy-Item -Path $chromiumExe.FullName -Destination (Join-Path $Destination "chromium/headless_shell.exe") -Force

  $ffmpegUrl = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip"
  $ffmpegZip = Join-Path $tmpRoot "ffmpeg.zip"
  Invoke-WebRequest -Uri $ffmpegUrl -OutFile $ffmpegZip
  Expand-Archive -Path $ffmpegZip -DestinationPath (Join-Path $tmpRoot "ffmpeg") -Force

  $ffmpegExe = Get-ChildItem -Path (Join-Path $tmpRoot "ffmpeg") -Recurse -File -Filter "ffmpeg.exe" | Select-Object -First 1
  if (-not $ffmpegExe) {
    throw "Could not find ffmpeg.exe in downloaded FFmpeg archive"
  }

  Copy-Item -Path $ffmpegExe.FullName -Destination (Join-Path $Destination "ffmpeg/ffmpeg.exe") -Force

  Write-Host "Sidecars installed:"
  Write-Host "  Chromium: $Destination/chromium/headless_shell.exe"
  Write-Host "  FFmpeg:   $Destination/ffmpeg/ffmpeg.exe"
}
finally {
  if (Test-Path $tmpRoot) {
    Remove-Item -Path $tmpRoot -Recurse -Force
  }
}
