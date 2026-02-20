use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use clap::Parser;
use tracing::{info, warn};

use browser_stream::chromium;
use browser_stream::cli::{AppConfig, CliArgs};
use browser_stream::encoder::{self, EncoderSettings, FfmpegEncoder};
use browser_stream::error::RuntimeError;
use browser_stream::retry::RetryPolicy;

#[derive(Debug, Clone)]
struct RuntimePaths {
    ffmpeg: PathBuf,
    chromium: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();
    init_tracing(args.verbose);

    let config = args.into_config()?;
    let runtime_paths = resolve_runtime_paths(&config)?;
    let retry_policy = RetryPolicy::new(
        config.retries,
        Duration::from_millis(config.retry_backoff_ms),
    );

    run_with_retry(&config, &runtime_paths, &retry_policy).await
}

async fn run_with_retry(
    config: &AppConfig,
    runtime_paths: &RuntimePaths,
    retry_policy: &RetryPolicy,
) -> Result<()> {
    let mut failures = 0_u32;

    loop {
        let attempt = failures + 1;
        info!(attempt, "starting stream attempt");

        let result = run_once(config, runtime_paths).await;

        match result {
            Ok(()) => return Ok(()),
            Err(err) => {
                if is_shutdown_error(&err) {
                    info!("shutdown requested, exiting");
                    // Force process termination in case any background runtime task/thread
                    // holds the process open after graceful shutdown.
                    std::process::exit(0);
                }

                failures = failures.saturating_add(1);
                if !retry_policy.should_retry(failures) {
                    return Err(err.context(format!(
                        "stream failed after {attempt} attempt(s) with {failures} failure(s)"
                    )));
                }

                warn!(
                    attempt,
                    failures,
                    backoff_ms = retry_policy.backoff.as_millis(),
                    error = %err,
                    "stream attempt failed; retrying"
                );

                tokio::select! {
                    _ = tokio::time::sleep(retry_policy.backoff) => {}
                    _ = tokio::signal::ctrl_c() => {
                        info!("shutdown requested during retry backoff, exiting");
                        return Ok(());
                    }
                }
            }
        }
    }
}

async fn run_once(config: &AppConfig, runtime_paths: &RuntimePaths) -> Result<()> {
    let settings = EncoderSettings {
        width: config.width,
        height: config.height,
        fps: config.fps,
        bitrate_kbps: config.bitrate_kbps,
        keyint_sec: config.keyint_sec,
        x264_opts: config.x264_opts.clone(),
        output: config.output.clone(),
        include_silent_audio: !config.no_audio,
        ffmpeg_path: runtime_paths.ffmpeg.clone(),
    };

    let mut encoder = FfmpegEncoder::spawn(&settings, config.verbose).await?;

    let stream_result =
        chromium::stream_browser_to_encoder(config, &runtime_paths.chromium, &mut encoder).await;

    match stream_result {
        Ok(()) => {
            let status = encoder.wait_for_exit().await?;
            if !status.success() {
                bail!("ffmpeg exited with status {status}");
            }
            Ok(())
        }
        Err(err) => {
            encoder.kill_and_wait().await;
            Err(err)
        }
    }
}

fn resolve_runtime_paths(config: &AppConfig) -> Result<RuntimePaths> {
    let current_exe =
        std::env::current_exe().context("failed to determine current executable path")?;
    let exe_dir = current_exe
        .parent()
        .context("failed to determine current executable directory")?;

    let ffmpeg_path = resolve_ffmpeg_path(config.ffmpeg_path.clone(), exe_dir)?;

    let chromium_path = resolve_binary_path(
        config.chromium_path.clone(),
        chromium::default_chromium_sidecar_path(exe_dir),
        "headless_shell",
    )?;

    Ok(RuntimePaths {
        ffmpeg: ffmpeg_path,
        chromium: chromium_path,
    })
}

fn resolve_binary_path(
    override_path: Option<PathBuf>,
    default_path: PathBuf,
    name: &'static str,
) -> Result<PathBuf> {
    let candidate = override_path.unwrap_or(default_path);

    if candidate.is_file() {
        return Ok(candidate);
    }

    Err(RuntimeError::MissingSidecar {
        name,
        path: candidate,
    }
    .into())
}

fn resolve_ffmpeg_path(
    override_path: Option<PathBuf>,
    exe_dir: &std::path::Path,
) -> Result<PathBuf> {
    if let Some(path) = override_path {
        return resolve_binary_path(Some(path), PathBuf::new(), "ffmpeg");
    }

    let sidecar = encoder::default_ffmpeg_sidecar_path(exe_dir);
    let system = find_in_path(encoder::ffmpeg_executable_name());

    if cfg!(target_os = "macos") {
        if let Some(system_path) = system {
            info!(
                ffmpeg = %system_path.display(),
                "using system ffmpeg on macOS (preferred over sidecar)"
            );
            return Ok(system_path);
        }
    }

    if sidecar.is_file() {
        return Ok(sidecar);
    }

    if let Some(system_path) = find_in_path(encoder::ffmpeg_executable_name()) {
        info!(
            ffmpeg = %system_path.display(),
            "using system ffmpeg from PATH"
        );
        return Ok(system_path);
    }

    Err(RuntimeError::MissingSidecar {
        name: "ffmpeg",
        path: sidecar,
    }
    .into())
}

fn find_in_path(executable_name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    let candidates = std::env::split_paths(&path_var);

    for dir in candidates {
        let direct = dir.join(executable_name);
        if direct.is_file() {
            return Some(direct);
        }

        if cfg!(target_os = "windows") {
            let exe = dir.join(format!("{executable_name}.exe"));
            if exe.is_file() {
                return Some(exe);
            }
        }
    }

    None
}

fn is_shutdown_error(err: &anyhow::Error) -> bool {
    err.downcast_ref::<RuntimeError>()
        .is_some_and(|runtime| matches!(runtime, RuntimeError::ShutdownRequested))
}

fn init_tracing(verbose: bool) {
    let filter = if verbose {
        tracing_subscriber::EnvFilter::new("info,browser_stream=debug,ffmpeg=info")
    } else {
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
    };

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .try_init();
}
