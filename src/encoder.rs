use std::path::{Path, PathBuf};
use std::process::ExitStatus;

use anyhow::{Context, Result, bail};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

use crate::frame::RgbFrame;

#[derive(Debug, Clone)]
pub struct EncoderSettings {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub bitrate_kbps: u32,
    pub keyint_sec: u32,
    pub x264_opts: String,
    pub output: String,
    pub include_silent_audio: bool,
    pub ffmpeg_path: PathBuf,
}

pub fn build_ffmpeg_args(settings: &EncoderSettings) -> Vec<String> {
    build_ffmpeg_args_with_loglevel(settings, "warning")
}

pub fn build_ffmpeg_args_with_loglevel(settings: &EncoderSettings, loglevel: &str) -> Vec<String> {
    let keyint = settings.fps.saturating_mul(settings.keyint_sec).max(1);
    let bufsize = settings.bitrate_kbps.saturating_mul(2);

    let mut args = vec![
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        loglevel.to_string(),
        "-stats_period".to_string(),
        "5".to_string(),
        "-stats".to_string(),
        "-f".to_string(),
        "rawvideo".to_string(),
        "-pix_fmt".to_string(),
        "rgb24".to_string(),
        "-s".to_string(),
        format!("{}x{}", settings.width, settings.height),
        "-r".to_string(),
        settings.fps.to_string(),
        "-i".to_string(),
        "-".to_string(),
    ];

    if settings.include_silent_audio {
        args.extend([
            "-f".to_string(),
            "lavfi".to_string(),
            "-i".to_string(),
            "anullsrc=r=48000:cl=stereo".to_string(),
        ]);
    }

    args.extend([
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "veryfast".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
        "-b:v".to_string(),
        format!("{}k", settings.bitrate_kbps),
        "-maxrate".to_string(),
        format!("{}k", settings.bitrate_kbps),
        "-bufsize".to_string(),
        format!("{}k", bufsize),
        "-g".to_string(),
        keyint.to_string(),
        "-keyint_min".to_string(),
        keyint.to_string(),
        "-x264-params".to_string(),
        settings.x264_opts.clone(),
    ]);

    if settings.include_silent_audio {
        args.extend([
            "-c:a".to_string(),
            "aac".to_string(),
            "-b:a".to_string(),
            "128k".to_string(),
            "-ar".to_string(),
            "48000".to_string(),
            "-ac".to_string(),
            "2".to_string(),
        ]);
    } else {
        args.push("-an".to_string());
    }

    args.extend(["-f".to_string(), "flv".to_string(), settings.output.clone()]);

    args
}

#[derive(Debug)]
pub struct FfmpegEncoder {
    child: Child,
    stdin: ChildStdin,
    stderr_task: JoinHandle<()>,
}

impl FfmpegEncoder {
    pub async fn spawn(settings: &EncoderSettings, verbose: bool) -> Result<Self> {
        let args =
            build_ffmpeg_args_with_loglevel(settings, if verbose { "info" } else { "warning" });

        info!(
            ffmpeg = %settings.ffmpeg_path.display(),
            output = %settings.output,
            "starting ffmpeg"
        );

        let mut cmd = Command::new(&settings.ffmpeg_path);
        cmd.args(&args)
            .stdin(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null());

        let mut child = cmd.spawn().with_context(|| {
            format!(
                "failed to spawn ffmpeg from {}",
                settings.ffmpeg_path.display()
            )
        })?;

        let stdin = child.stdin.take().context("ffmpeg stdin unavailable")?;

        let stderr = child.stderr.take().context("ffmpeg stderr unavailable")?;

        let stderr_task = tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if verbose {
                    info!(target: "ffmpeg", "{line}");
                } else {
                    debug!(target: "ffmpeg", "{line}");
                }
            }
        });

        Ok(Self {
            child,
            stdin,
            stderr_task,
        })
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>> {
        self.child
            .try_wait()
            .context("failed to poll ffmpeg process")
    }

    pub async fn write_frame(&mut self, frame: &RgbFrame) -> Result<()> {
        if frame.width == 0 || frame.height == 0 {
            bail!("invalid frame dimensions {}x{}", frame.width, frame.height);
        }

        if let Some(status) = self.try_wait()? {
            bail!("ffmpeg exited early with status {status}");
        }

        self.stdin
            .write_all(&frame.data)
            .await
            .context("failed writing frame to ffmpeg stdin")?;

        Ok(())
    }

    pub async fn kill_and_wait(&mut self) {
        match self.child.kill().await {
            Ok(()) => {}
            Err(err) => warn!("failed to kill ffmpeg: {err}"),
        }

        match self.child.wait().await {
            Ok(status) => debug!("ffmpeg exited after kill: {status}"),
            Err(err) => warn!("failed waiting on ffmpeg: {err}"),
        }
    }

    pub async fn wait_for_exit(mut self) -> Result<ExitStatus> {
        drop(self.stdin);
        let status = self
            .child
            .wait()
            .await
            .context("failed waiting for ffmpeg exit")?;

        self.stderr_task.abort();
        Ok(status)
    }
}

pub fn ffmpeg_executable_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    }
}

pub fn default_ffmpeg_sidecar_path(exe_dir: &Path) -> PathBuf {
    exe_dir
        .join("..")
        .join("sidecar")
        .join("ffmpeg")
        .join(ffmpeg_executable_name())
}
