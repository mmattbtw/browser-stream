#!/usr/bin/env bash
set -euo pipefail

DESTINATION="target/sidecar"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --destination)
      DESTINATION="$2"
      shift 2
      ;;
    -h|--help)
      cat <<'USAGE'
Fetch bundled Chromium Headless Shell and FFmpeg sidecars for the current platform.

Usage:
  ./scripts/fetch-sidecars.sh [--destination <dir>]

Defaults:
  destination: target/sidecar
USAGE
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if ! command -v curl >/dev/null 2>&1; then
  echo "curl is required" >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required" >&2
  exit 1
fi

if ! command -v unzip >/dev/null 2>&1; then
  echo "unzip is required" >&2
  exit 1
fi

uname_s="$(uname -s)"
uname_m="$(uname -m)"

chromium_platform=""
ffmpeg_url=""
chromium_exec_name="chrome-headless-shell"
ffmpeg_exec_name="ffmpeg"

case "${uname_s}:${uname_m}" in
  Darwin:arm64)
    chromium_platform="mac-arm64"
    ffmpeg_url="https://ffmpeg.martin-riedl.de/redirect/latest/macos/arm64/release/ffmpeg.zip"
    ;;
  Linux:x86_64)
    chromium_platform="linux64"
    ffmpeg_url="https://ffmpeg.martin-riedl.de/redirect/latest/linux/amd64/release/ffmpeg.zip"
    ;;
  *)
    echo "Unsupported platform ${uname_s}/${uname_m}. This script supports macOS arm64 and Linux x86_64." >&2
    exit 1
    ;;
esac

chrome_manifest_url="https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions-with-downloads.json"
chromium_url="$(curl -fsSL "$chrome_manifest_url" | jq -r --arg platform "$chromium_platform" '.channels.Stable.downloads["chrome-headless-shell"][] | select(.platform == $platform) | .url')"

if [[ -z "$chromium_url" || "$chromium_url" == "null" ]]; then
  echo "Failed to resolve Chromium headless shell URL for platform ${chromium_platform}" >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

mkdir -p "$DESTINATION/chromium" "$DESTINATION/ffmpeg"

chromium_zip="$tmp_dir/chromium.zip"
ffmpeg_zip="$tmp_dir/ffmpeg.zip"

printf 'Downloading Chromium headless shell (%s)\n' "$chromium_platform"
curl -fL "$chromium_url" -o "$chromium_zip"
unzip -q "$chromium_zip" -d "$tmp_dir/chromium"

chromium_source="$(find "$tmp_dir/chromium" -type f -name "$chromium_exec_name" | head -n 1)"
if [[ -z "$chromium_source" ]]; then
  echo "Could not find ${chromium_exec_name} inside Chromium archive" >&2
  exit 1
fi

chromium_root="$(dirname "$chromium_source")"
cp -R "$chromium_root"/. "$DESTINATION/chromium/"
cp "$chromium_source" "$DESTINATION/chromium/headless_shell"
chmod +x "$DESTINATION/chromium/headless_shell"

printf 'Downloading FFmpeg\n'
curl -fL "$ffmpeg_url" -o "$ffmpeg_zip"
unzip -q "$ffmpeg_zip" -d "$tmp_dir/ffmpeg"

ffmpeg_source="$(find "$tmp_dir/ffmpeg" -type f -name "$ffmpeg_exec_name" | head -n 1)"
if [[ -z "$ffmpeg_source" ]]; then
  echo "Could not find ${ffmpeg_exec_name} inside FFmpeg archive" >&2
  exit 1
fi

cp "$ffmpeg_source" "$DESTINATION/ffmpeg/ffmpeg"
chmod +x "$DESTINATION/ffmpeg/ffmpeg"

printf 'Sidecars installed:\n'
printf '  Chromium: %s\n' "$DESTINATION/chromium/headless_shell"
printf '  FFmpeg:   %s\n' "$DESTINATION/ffmpeg/ffmpeg"
