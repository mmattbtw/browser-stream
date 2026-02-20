# browser-stream

Rust CLI to stream a fullscreen browser-rendered website to RTMP/RTMPS using bundled Chromium Headless Shell and FFmpeg sidecars.

## Expected sidecar layout

The binary resolves sidecars relative to itself:

- `../sidecar/chromium/headless_shell` (`headless_shell.exe` on Windows)
- `../sidecar/ffmpeg/ffmpeg` (`ffmpeg.exe` on Windows)

You can override both paths with CLI flags:

- `--chromium-path /abs/path/to/headless_shell`
- `--ffmpeg-path /abs/path/to/ffmpeg`

## Local Sidecar Setup

When running with `cargo run`, the executable lives under `target/`, so sidecars are expected under `target/sidecar`.

Fetch sidecars for your host platform:

```bash
./scripts/fetch-sidecars.sh
```

Windows (PowerShell):

```powershell
./scripts/fetch-sidecars.ps1
```

## Usage

```bash
cargo run -- \
  --url https://example.com \
  --width 1920 \
  --height 1080 \
  --fps 30 \
  --bitrate-kbps 4500 \
  --keyint-sec 1 \
  --x264-opts bframes=0 \
  --rtmp-url rtmp://live.example.com/app \
  --stream-key mystream
```

Equivalent full-output form:

```bash
cargo run -- \
  --url https://example.com \
  --output rtmp://live.example.com/app/mystream
```

## Docker Compose

`docker-compose.yml` defines two image variants:

- `slim` (default service `browser-stream`): uses system `chromium` + `ffmpeg` packages in the container.
- `full` (service `browser-stream-full`): bundles sidecar binaries in image at `/opt/sidecar/...`.

Build and run `slim` (default):

```bash
docker compose up --build
```

Build and run `full`:

```bash
docker compose --profile full up --build browser-stream-full
```

Or build directly with Docker targets:

```bash
docker build --target slim -t browser-stream:slim .
docker build --target full -t browser-stream:full .
```

Configure with environment variables (for example in `.env`):

```bash
WEBSITE_URL=https://example.com
# Option A: full output URL
OUTPUT=rtmp://live.example.com/app/mystream

# Option B: split URL + key (used when OUTPUT is empty)
RTMP_URL=rtmp://live.example.com/app
STREAM_KEY=mystream

WIDTH=1920
HEIGHT=1080
FPS=30
BITRATE_KBPS=4500
KEYINT_SEC=1
X264_OPTS=bframes=0
RETRIES=5
RETRY_BACKOFF_MS=1000
STARTUP_DELAY_MS=2000
FRAME_TIMEOUT_MS=30000
NO_AUDIO=0
VERBOSE=0
```

Binary resolution in containers:

- `slim`: auto-detects system `chromium`/`chromium-browser` and `ffmpeg`.
- `full`: uses bundled sidecars via `BROWSER_STREAM_CHROMIUM_PATH` and `BROWSER_STREAM_FFMPEG_PATH`.

No sidecar downloads are required for either image.
Both images set `BROWSER_STREAM_NO_SANDBOX=1` for Chromium compatibility in containers.

## Defaults

- `width=1920`
- `height=1080`
- `fps=30`
- `bitrate-kbps=4500`
- `keyint-sec=1`
- `x264-opts=bframes=0`
- `retries=5`
- `retry-backoff-ms=1000`
- `startup-delay-ms=2000`
- `frame-timeout-ms=30000`
- `no-audio=false` (silent audio track enabled by default)

## Notes

- v1 supports public HTTP(S) website URLs.
- A silent audio track is included by default for RTMP compatibility.
- Use `--no-audio` to disable audio.
- RTMP output supports `rtmp://` and `rtmps://`.
- On stream failure, the app retries a fixed number of times and exits non-zero once exhausted.
- Runtime controls while streaming:
  - type `r` or `refresh` then press Enter to manually reload the page.
  - type `h` or `help` then press Enter to print controls.
  - with compose, run foreground (`docker compose up`) to send commands directly via stdin.

## GitHub Release Bundling

Workflow: `.github/workflows/build-and-release.yml`

- Builds release binaries for:
  - macOS arm64
  - Linux x86_64
  - Windows x86_64
- Downloads platform sidecars and packages archives in this layout:
  - `bin/browser-stream[.exe]`
  - `sidecar/chromium/headless_shell[.exe]`
  - `sidecar/ffmpeg/ffmpeg[.exe]`
- Uploads build artifacts on PR/push.
- Publishes release assets automatically on tags like `v0.1.0`.

## GHCR Docker Publish

Workflow: `.github/workflows/docker-publish.yml`

- Builds and publishes both Docker variants:
  - `slim`: `linux/amd64` and `linux/arm64`
  - `full`: `linux/amd64` (sidecar availability)
- Publishes to GitHub Container Registry on push to `main` and version tags (`v*`).
- Uses image name:
  - `ghcr.io/<owner>/<repo>`
  - Example for this repo: `ghcr.io/mmattbtw/browser-stream`
- Tags include:
  - `latest-slim` / `latest-full` (default branch)
  - branch/tag refs with suffixes (`-slim`, `-full`)
  - `sha-<commit>-slim` / `sha-<commit>-full`

Example:

```bash
docker pull ghcr.io/mmattbtw/browser-stream:latest-slim
docker pull ghcr.io/mmattbtw/browser-stream:latest-full
```
