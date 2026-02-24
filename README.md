# browser-stream

Stream a fullscreen website to RTMP/RTMPS using Chromium + FFmpeg.

## Example usage

```bash
cargo run -- \
  --url https://example.com \
  --output rtmp://live.example.com/app/mystream
```

Equivalent split URL + stream key:

```bash
cargo run -- \
  --url https://example.com \
  --rtmp-url rtmp://live.example.com/app \
  --stream-key mystream
```

## Build and run locally

1. Fetch sidecars:

```bash
./scripts/fetch-sidecars.sh
```

2. Build:

```bash
cargo build --release
```

3. Run:

```bash
cargo run -- \
  --url https://example.com \
  --output rtmp://live.example.com/app/mystream
```

## Docker

Use environment variables (copy `.env.example` to `.env` and edit values), then run:

```bash
docker compose up --build
```

Full image variant:

```bash
docker compose --profile full up --build browser-stream-full
```

Build images directly:

```bash
docker build --target slim -t browser-stream:slim .
docker build --target full -t browser-stream:full .
```

## CLI arguments

`browser-stream` supports the following flags:

| Flag | Type | Default | Required | Notes |
| --- | --- | --- | --- | --- |
| `--url` | string | none | yes | Website URL. Must be `http` or `https`. |
| `--width` | u32 | `1920` | no | Frame width. Min `16`. |
| `--height` | u32 | `1080` | no | Frame height. Min `16`. |
| `--fps` | u32 | `30` | no | Frame rate. Range `1..=120`. |
| `--bitrate-kbps` | u32 | `4500` | no | Video bitrate in kbps. Min `100`. |
| `--keyint-sec` | u32 | `1` | no | GOP/keyframe interval in seconds. Range `1..=60`. |
| `--x264-opts` | string | `bframes=0` | no | Passed to x264 options. |
| `--rtmp-url` | string | none | conditional | Use with `--stream-key` if `--output` is not set. |
| `--stream-key` | string | none | conditional | Use with `--rtmp-url` if `--output` is not set. |
| `--output` | string | none | conditional | Full output URL (for example `rtmp://.../app/key`). Alternative to `--rtmp-url` + `--stream-key`. |
| `--retries` | u32 | `5` | no | Number of retry attempts after failure. |
| `--retry-backoff-ms` | u64 | `1000` | no | Delay between retries (milliseconds). |
| `--startup-delay-ms` | u64 | `2000` | no | Delay before starting frame capture (milliseconds). |
| `--frame-timeout-ms` | u64 | `30000` | no | Frame read timeout (milliseconds). Min `1000`. |
| `--no-audio` | bool flag | `false` | no | Disable silent audio track. |
| `--ffmpeg-path` | path | auto | no | Override ffmpeg binary path. |
| `--chromium-path` | path | auto | no | Override chromium/headless shell binary path. |
| `--verbose` | bool flag | `false` | no | Enable verbose logging. |

Output selection rules:

- Provide `--output`, or
- Provide both `--rtmp-url` and `--stream-key`.
