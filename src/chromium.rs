use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::page::{
    EventScreencastFrame, ScreencastFrameAckParams, StartScreencastFormat, StartScreencastParams,
    StopScreencastParams,
};
use chromiumoxide::handler::viewport::Viewport;
use futures::StreamExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;
use tracing::{debug, error, info, warn};

use crate::cli::AppConfig;
use crate::encoder::FfmpegEncoder;
use crate::error::RuntimeError;
use crate::frame::{RgbFrame, decode_screencast_frame};

pub async fn stream_browser_to_encoder(
    config: &AppConfig,
    chromium_path: &Path,
    encoder: &mut FfmpegEncoder,
) -> Result<()> {
    let viewport = Viewport {
        width: config.width,
        height: config.height,
        device_scale_factor: Some(1.0),
        emulating_mobile: false,
        is_landscape: config.width >= config.height,
        has_touch: false,
    };

    let mut browser_builder = BrowserConfig::builder()
        .chrome_executable(chromium_path)
        .window_size(config.width, config.height)
        .new_headless_mode()
        .viewport(viewport)
        .arg("--autoplay-policy=no-user-gesture-required")
        .arg("--disable-background-timer-throttling")
        .arg("--disable-backgrounding-occluded-windows")
        .arg("--disable-renderer-backgrounding");

    if no_sandbox_from_env() {
        browser_builder = browser_builder.no_sandbox();
    }

    let browser_config = browser_builder
        .build()
        .map_err(|err| anyhow!("failed to build browser config: {err}"))?;

    info!(chromium = %chromium_path.display(), "starting chromium");

    let (mut browser, mut handler) = Browser::launch(browser_config)
        .await
        .context("failed to launch chromium")?;

    let handler_task = tokio::spawn(async move {
        while let Some(item) = handler.next().await {
            if let Err(err) = item {
                error!("chromium handler error: {err}");
                break;
            }
        }
    });

    let page = browser
        .new_page("about:blank")
        .await
        .context("failed to create page")?;

    page.goto(config.website_url.as_str())
        .await
        .with_context(|| format!("failed loading {}", config.website_url))?;

    // `goto` waits for page load completion. Delay further for dynamic JS/CSS settling.
    tokio::time::sleep(Duration::from_millis(config.startup_delay_ms)).await;

    let mut frame_events = page
        .event_listener::<EventScreencastFrame>()
        .await
        .context("failed to register screencast event listener")?;

    let start_params = StartScreencastParams::builder()
        .format(StartScreencastFormat::Jpeg)
        .quality(80_i64)
        .max_width(i64::from(config.width))
        .max_height(i64::from(config.height))
        .every_nth_frame(1_i64)
        .build();

    page.execute(start_params)
        .await
        .context("failed to start screencast")?;

    info!("runtime controls: type `r` then Enter to refresh the page");

    let mut control_rx = spawn_control_listener();
    let frame_interval = Duration::from_secs_f64(1.0_f64 / f64::from(config.fps));
    let mut frame_tick = tokio::time::interval(frame_interval);
    frame_tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
    frame_tick.tick().await;
    let mut stats_tick = tokio::time::interval(Duration::from_secs(5));
    stats_tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
    stats_tick.tick().await;

    let first_frame_timeout = tokio::time::sleep(Duration::from_millis(config.frame_timeout_ms));
    tokio::pin!(first_frame_timeout);
    let mut latest_frame: Option<RgbFrame> = None;
    let mut decoded_frames: u64 = 0;
    let mut encoded_frames: u64 = 0;

    let stream_result: Result<()> = async {
        loop {
            tokio::select! {
                biased;
                _ = frame_tick.tick() => {
                    if let Some(frame) = latest_frame.as_ref() {
                        encoder.write_frame(frame).await?;
                        encoded_frames = encoded_frames.saturating_add(1);
                    }
                }
                maybe_event = frame_events.next() => {
                    let event = maybe_event.context("screencast event stream ended unexpectedly")?;

                    page.execute(ScreencastFrameAckParams::new(event.session_id))
                        .await
                        .context("failed to ack screencast frame")?;

                    let frame = decode_screencast_frame(event.data.as_ref(), config.width, config.height)
                        .context("failed to decode screencast frame")?;

                    if latest_frame.is_none() {
                        info!("received first screencast frame");
                        // Prime ffmpeg immediately so it can initialize output without waiting for the first tick.
                        encoder.write_frame(&frame).await?;
                        encoded_frames = encoded_frames.saturating_add(1);
                    }
                    decoded_frames = decoded_frames.saturating_add(1);
                    latest_frame = Some(frame);
                }
                _ = stats_tick.tick() => {
                    debug!(
                        decoded_frames,
                        encoded_frames,
                        has_frame = latest_frame.is_some(),
                        "streaming stats"
                    );
                }
                command = control_rx.recv() => {
                    match command {
                        Some(ControlCommand::Refresh) => {
                            page.reload()
                                .await
                                .context("manual refresh failed")?;
                            info!("manual refresh applied");
                        }
                        Some(ControlCommand::Help) => {
                            info!("runtime controls: `r` or `refresh` reloads the page");
                        }
                        None => {
                            // stdin closed; continue streaming without runtime controls.
                        }
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    return Err(RuntimeError::ShutdownRequested.into());
                }
                _ = &mut first_frame_timeout, if latest_frame.is_none() => {
                    return Err(RuntimeError::ScreencastTimeout.into());
                }
            }
        }
    }
    .await;

    if let Err(err) = page.execute(StopScreencastParams::default()).await {
        warn!("failed to stop screencast cleanly: {err}");
    }

    if let Err(err) = browser.close().await {
        warn!("failed to close browser cleanly: {err}");
    }
    if let Err(err) = browser.wait().await {
        warn!("failed to wait for browser process: {err}");
    }

    handler_task.abort();

    stream_result
}

fn no_sandbox_from_env() -> bool {
    match std::env::var("BROWSER_STREAM_NO_SANDBOX") {
        Ok(value) => parse_truthy(&value),
        Err(_) => false,
    }
}

fn parse_truthy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes"
    )
}

pub fn chromium_executable_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "headless_shell.exe"
    } else {
        "headless_shell"
    }
}

pub fn default_chromium_sidecar_path(exe_dir: &Path) -> PathBuf {
    exe_dir
        .join("..")
        .join("sidecar")
        .join("chromium")
        .join(chromium_executable_name())
}

#[derive(Debug, Copy, Clone)]
enum ControlCommand {
    Refresh,
    Help,
}

fn parse_control_command(input: &str) -> Option<ControlCommand> {
    match input.trim().to_ascii_lowercase().as_str() {
        "r" | "refresh" => Some(ControlCommand::Refresh),
        "h" | "help" => Some(ControlCommand::Help),
        _ => None,
    }
}

fn spawn_control_listener() -> mpsc::UnboundedReceiver<ControlCommand> {
    let (tx, rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        let mut lines = BufReader::new(tokio::io::stdin()).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if let Some(command) = parse_control_command(&line) {
                if tx.send(command).is_err() {
                    break;
                }
            }
        }
    });

    rx
}

#[cfg(test)]
mod tests {
    use super::{ControlCommand, parse_control_command, parse_truthy};

    #[test]
    fn parses_refresh_shortcut() {
        assert!(matches!(
            parse_control_command("r"),
            Some(ControlCommand::Refresh)
        ));
    }

    #[test]
    fn parses_refresh_word() {
        assert!(matches!(
            parse_control_command(" refresh "),
            Some(ControlCommand::Refresh)
        ));
    }

    #[test]
    fn ignores_unknown_commands() {
        assert!(parse_control_command("noop").is_none());
    }

    #[test]
    fn truthy_parser() {
        assert!(parse_truthy("true"));
        assert!(parse_truthy("1"));
        assert!(parse_truthy("YES"));
        assert!(!parse_truthy("0"));
        assert!(!parse_truthy("false"));
    }
}
