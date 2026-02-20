FROM rust:1.89-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim AS sidecar-fetch
ARG TARGETARCH

RUN arch="${TARGETARCH:-amd64}" \
  && test "${arch}" = "amd64" || (echo "full image sidecars currently support linux/amd64 only" >&2 && exit 1)

RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    jq \
    unzip \
  && rm -rf /var/lib/apt/lists/*

RUN set -euo pipefail; \
  mkdir -p /out/sidecar/chromium /out/sidecar/ffmpeg; \
  chromium_manifest_url="https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions-with-downloads.json"; \
  chromium_url="$(curl -fsSL "${chromium_manifest_url}" | jq -r '.channels.Stable.downloads["chrome-headless-shell"][] | select(.platform == "linux64") | .url')"; \
  test -n "${chromium_url}" && test "${chromium_url}" != "null"; \
  curl -fL "${chromium_url}" -o /tmp/chromium.zip; \
  unzip -q /tmp/chromium.zip -d /tmp/chromium; \
  chromium_source="$(find /tmp/chromium -type f -name chrome-headless-shell | head -n 1)"; \
  test -n "${chromium_source}"; \
  chromium_root="$(dirname "${chromium_source}")"; \
  cp -R "${chromium_root}"/. /out/sidecar/chromium/; \
  cp "${chromium_source}" /out/sidecar/chromium/headless_shell; \
  chmod +x /out/sidecar/chromium/headless_shell; \
  ffmpeg_url="https://ffmpeg.martin-riedl.de/redirect/latest/linux/amd64/release/ffmpeg.zip"; \
  curl -fL "${ffmpeg_url}" -o /tmp/ffmpeg.zip; \
  unzip -q /tmp/ffmpeg.zip -d /tmp/ffmpeg; \
  ffmpeg_source="$(find /tmp/ffmpeg -type f -name ffmpeg | head -n 1)"; \
  test -n "${ffmpeg_source}"; \
  cp "${ffmpeg_source}" /out/sidecar/ffmpeg/ffmpeg; \
  chmod +x /out/sidecar/ffmpeg/ffmpeg

FROM debian:bookworm-slim AS runtime-base
WORKDIR /app

RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    ca-certificates \
    fonts-liberation \
    libasound2 \
    libatk-bridge2.0-0 \
    libatk1.0-0 \
    libc6 \
    libcairo2 \
    libcups2 \
    libdbus-1-3 \
    libdrm2 \
    libexpat1 \
    libgbm1 \
    libglib2.0-0 \
    libgtk-3-0 \
    libnspr4 \
    libnss3 \
    libpango-1.0-0 \
    libu2f-udev \
    libx11-6 \
    libx11-xcb1 \
    libxcb1 \
    libxcomposite1 \
    libxcursor1 \
    libxdamage1 \
    libxext6 \
    libxfixes3 \
    libxi6 \
    libxkbcommon0 \
    libxrandr2 \
    libxrender1 \
    libxshmfence1 \
    xdg-utils \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/browser-stream /usr/local/bin/browser-stream
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

RUN groupadd --system browserstream \
  && useradd --system --gid browserstream --create-home --home-dir /home/browserstream browserstream

ENV RUST_LOG=info
ENV BROWSER_STREAM_NO_SANDBOX=1

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]

FROM runtime-base AS full
COPY --from=sidecar-fetch /out/sidecar /opt/sidecar
RUN chown -R browserstream:browserstream /opt/sidecar

ENV BROWSER_STREAM_CHROMIUM_PATH=/opt/sidecar/chromium/headless_shell
ENV BROWSER_STREAM_FFMPEG_PATH=/opt/sidecar/ffmpeg/ffmpeg

USER browserstream

FROM runtime-base AS slim
RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    chromium \
    ffmpeg \
  && rm -rf /var/lib/apt/lists/*

USER browserstream

FROM slim AS default
