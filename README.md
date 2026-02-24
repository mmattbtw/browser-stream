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

PowerShell:

```powershell
./scripts/fetch-sidecars.ps1
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
