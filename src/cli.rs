use std::path::PathBuf;

use clap::Parser;
use url::Url;

use crate::error::ConfigError;

#[derive(Debug, Parser, Clone)]
#[command(
    name = "browser-stream",
    version,
    about = "Stream a browser page to RTMP"
)]
pub struct CliArgs {
    #[arg(long)]
    pub url: String,

    #[arg(long, default_value_t = 1920)]
    pub width: u32,

    #[arg(long, default_value_t = 1080)]
    pub height: u32,

    #[arg(long, default_value_t = 30)]
    pub fps: u32,

    #[arg(long, default_value_t = 4500)]
    pub bitrate_kbps: u32,

    #[arg(long, default_value_t = 1)]
    pub keyint_sec: u32,

    #[arg(long, default_value = "bframes=0")]
    pub x264_opts: String,

    #[arg(long)]
    pub rtmp_url: Option<String>,

    #[arg(long)]
    pub stream_key: Option<String>,

    #[arg(long)]
    pub output: Option<String>,

    #[arg(long, default_value_t = 5)]
    pub retries: u32,

    #[arg(long, default_value_t = 1000)]
    pub retry_backoff_ms: u64,

    #[arg(long, default_value_t = 2000)]
    pub startup_delay_ms: u64,

    #[arg(long, default_value_t = 30000)]
    pub frame_timeout_ms: u64,

    #[arg(long, default_value_t = false)]
    pub no_audio: bool,

    #[arg(long)]
    pub ffmpeg_path: Option<PathBuf>,

    #[arg(long)]
    pub chromium_path: Option<PathBuf>,

    #[arg(long, default_value_t = false)]
    pub verbose: bool,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub website_url: Url,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub bitrate_kbps: u32,
    pub keyint_sec: u32,
    pub x264_opts: String,
    pub output: String,
    pub retries: u32,
    pub retry_backoff_ms: u64,
    pub startup_delay_ms: u64,
    pub frame_timeout_ms: u64,
    pub no_audio: bool,
    pub ffmpeg_path: Option<PathBuf>,
    pub chromium_path: Option<PathBuf>,
    pub verbose: bool,
}

impl CliArgs {
    pub fn into_config(self) -> Result<AppConfig, ConfigError> {
        validate_range("width", self.width as u64, 16, u32::MAX as u64)?;
        validate_range("height", self.height as u64, 16, u32::MAX as u64)?;
        validate_range("fps", self.fps as u64, 1, 120)?;
        validate_range(
            "bitrate-kbps",
            self.bitrate_kbps as u64,
            100,
            u32::MAX as u64,
        )?;
        validate_range("keyint-sec", self.keyint_sec as u64, 1, 60)?;
        validate_range("frame-timeout-ms", self.frame_timeout_ms, 1000, u64::MAX)?;

        let website_url =
            Url::parse(&self.url).map_err(|_| ConfigError::InvalidWebsiteUrl(self.url.clone()))?;
        match website_url.scheme() {
            "http" | "https" => {}
            other => return Err(ConfigError::UnsupportedWebsiteScheme(other.to_string())),
        }

        let output = crate::rtmp::build_output(self.output, self.rtmp_url, self.stream_key)?;

        Ok(AppConfig {
            website_url,
            width: self.width,
            height: self.height,
            fps: self.fps,
            bitrate_kbps: self.bitrate_kbps,
            keyint_sec: self.keyint_sec,
            x264_opts: self.x264_opts,
            output,
            retries: self.retries,
            retry_backoff_ms: self.retry_backoff_ms,
            startup_delay_ms: self.startup_delay_ms,
            frame_timeout_ms: self.frame_timeout_ms,
            no_audio: self.no_audio,
            ffmpeg_path: self.ffmpeg_path,
            chromium_path: self.chromium_path,
            verbose: self.verbose,
        })
    }
}

fn validate_range(field: &'static str, actual: u64, min: u64, max: u64) -> Result<(), ConfigError> {
    if actual < min || actual > max {
        return Err(ConfigError::OutOfRange {
            field,
            min,
            max,
            actual,
        });
    }
    Ok(())
}
