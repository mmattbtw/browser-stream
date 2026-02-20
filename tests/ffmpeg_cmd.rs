use std::path::PathBuf;

use browser_stream::encoder::{EncoderSettings, build_ffmpeg_args};

#[test]
fn derives_keyint_from_fps_and_seconds() {
    let settings = EncoderSettings {
        width: 1280,
        height: 720,
        fps: 30,
        bitrate_kbps: 2500,
        keyint_sec: 2,
        x264_opts: "bframes=0".to_string(),
        output: "rtmp://live.example.com/app/key".to_string(),
        include_silent_audio: true,
        ffmpeg_path: PathBuf::from("/tmp/ffmpeg"),
    };

    let args = build_ffmpeg_args(&settings);

    assert_pair(&args, "-g", "60");
    assert_pair(&args, "-keyint_min", "60");
}

#[test]
fn includes_cbr_like_flags() {
    let settings = EncoderSettings {
        width: 1920,
        height: 1080,
        fps: 60,
        bitrate_kbps: 4500,
        keyint_sec: 1,
        x264_opts: "bframes=0".to_string(),
        output: "rtmps://live.example.com/app/key".to_string(),
        include_silent_audio: true,
        ffmpeg_path: PathBuf::from("/tmp/ffmpeg"),
    };

    let args = build_ffmpeg_args(&settings);

    assert_pair(&args, "-b:v", "4500k");
    assert_pair(&args, "-maxrate", "4500k");
    assert_pair(&args, "-bufsize", "9000k");
}

#[test]
fn passes_x264_opts_and_output() {
    let settings = EncoderSettings {
        width: 1920,
        height: 1080,
        fps: 30,
        bitrate_kbps: 3000,
        keyint_sec: 1,
        x264_opts: "bframes=0:scenecut=0".to_string(),
        output: "rtmp://live.example.com/app/key".to_string(),
        include_silent_audio: true,
        ffmpeg_path: PathBuf::from("/tmp/ffmpeg"),
    };

    let args = build_ffmpeg_args(&settings);

    assert_pair(&args, "-x264-params", "bframes=0:scenecut=0");
    assert_eq!(
        args.last().expect("args should not be empty"),
        "rtmp://live.example.com/app/key"
    );
}

fn assert_pair(args: &[String], flag: &str, value: &str) {
    let index = args
        .iter()
        .position(|item| item == flag)
        .expect("flag should exist in arg list");

    let next = args
        .get(index + 1)
        .expect("flag should have a following value");

    assert_eq!(next, value);
}
