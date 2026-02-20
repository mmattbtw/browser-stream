use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("website URL must use http or https, got `{0}`")]
    UnsupportedWebsiteScheme(String),
    #[error("invalid website URL `{0}`")]
    InvalidWebsiteUrl(String),
    #[error("`{field}` out of range: got {actual}, expected {min}..={max}")]
    OutOfRange {
        field: &'static str,
        min: u64,
        max: u64,
        actual: u64,
    },
    #[error(transparent)]
    Rtmp(#[from] crate::rtmp::RtmpError),
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("shutdown requested")]
    ShutdownRequested,
    #[error("timed out waiting for screencast frames")]
    ScreencastTimeout,
    #[error(
        "missing sidecar binary `{name}` at `{path}`. Provide an explicit override path or place sidecars at this location. For local development, fetch sidecars with `./scripts/fetch-sidecars.sh` (macOS/Linux) or `./scripts/fetch-sidecars.ps1` (Windows). Supported packaged targets: macOS arm64, Linux x86_64, Windows x86_64"
    )]
    MissingSidecar { name: &'static str, path: PathBuf },
}
