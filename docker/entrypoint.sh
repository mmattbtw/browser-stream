#!/usr/bin/env bash
set -euo pipefail

if [[ -z "${WEBSITE_URL:-}" ]]; then
  echo "WEBSITE_URL is required" >&2
  exit 1
fi

if [[ -z "${OUTPUT:-}" && ( -z "${RTMP_URL:-}" || -z "${STREAM_KEY:-}" ) ]]; then
  echo "Provide either OUTPUT or both RTMP_URL and STREAM_KEY" >&2
  exit 1
fi

if [[ -n "${BROWSER_STREAM_CHROMIUM_PATH:-}" ]]; then
  CHROMIUM_BIN="${BROWSER_STREAM_CHROMIUM_PATH}"
elif command -v chromium >/dev/null 2>&1; then
  CHROMIUM_BIN="$(command -v chromium)"
elif command -v chromium-browser >/dev/null 2>&1; then
  CHROMIUM_BIN="$(command -v chromium-browser)"
else
  echo "Could not find chromium binary in container" >&2
  exit 1
fi

if [[ ! -x "${CHROMIUM_BIN}" ]]; then
  echo "Configured chromium binary is not executable: ${CHROMIUM_BIN}" >&2
  exit 1
fi

if [[ -n "${BROWSER_STREAM_FFMPEG_PATH:-}" ]]; then
  FFMPEG_BIN="${BROWSER_STREAM_FFMPEG_PATH}"
elif command -v ffmpeg >/dev/null 2>&1; then
  FFMPEG_BIN="$(command -v ffmpeg)"
else
  echo "Could not find ffmpeg binary in container" >&2
  exit 1
fi

if [[ ! -x "${FFMPEG_BIN}" ]]; then
  echo "Configured ffmpeg binary is not executable: ${FFMPEG_BIN}" >&2
  exit 1
fi

args=(
  --url "${WEBSITE_URL}"
  --width "${WIDTH:-1920}"
  --height "${HEIGHT:-1080}"
  --fps "${FPS:-30}"
  --bitrate-kbps "${BITRATE_KBPS:-4500}"
  --keyint-sec "${KEYINT_SEC:-1}"
  --x264-opts "${X264_OPTS:-bframes=0}"
  --retries "${RETRIES:-5}"
  --retry-backoff-ms "${RETRY_BACKOFF_MS:-1000}"
  --startup-delay-ms "${STARTUP_DELAY_MS:-2000}"
  --frame-timeout-ms "${FRAME_TIMEOUT_MS:-30000}"
  --chromium-path "${CHROMIUM_BIN}"
  --ffmpeg-path "${FFMPEG_BIN}"
)

if [[ -n "${OUTPUT:-}" ]]; then
  args+=(--output "${OUTPUT}")
else
  args+=(--rtmp-url "${RTMP_URL}" --stream-key "${STREAM_KEY}")
fi

if [[ "${VERBOSE:-0}" == "1" || "${VERBOSE:-false}" == "true" ]]; then
  args+=(--verbose)
fi

if [[ "${NO_AUDIO:-0}" == "1" || "${NO_AUDIO:-false}" == "true" ]]; then
  args+=(--no-audio)
fi

exec /usr/local/bin/browser-stream "${args[@]}" "$@"
