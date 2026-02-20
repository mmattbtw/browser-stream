use assert_matches::assert_matches;
use clap::Parser;

use browser_stream::cli::CliArgs;
use browser_stream::error::ConfigError;
use browser_stream::rtmp::RtmpError;

#[test]
fn parses_defaults_and_required_fields() {
    let args = CliArgs::try_parse_from([
        "browser-stream",
        "--url",
        "https://example.com",
        "--output",
        "rtmp://live.example.com/app/stream",
    ])
    .expect("cli parse should succeed");

    let config = args.into_config().expect("config should validate");

    assert_eq!(config.width, 1920);
    assert_eq!(config.height, 1080);
    assert_eq!(config.fps, 30);
    assert_eq!(config.bitrate_kbps, 4500);
    assert_eq!(config.keyint_sec, 1);
    assert_eq!(config.x264_opts, "bframes=0");
    assert_eq!(config.retries, 5);
    assert_eq!(config.retry_backoff_ms, 1000);
    assert_eq!(config.startup_delay_ms, 2000);
    assert_eq!(config.frame_timeout_ms, 30000);
    assert!(!config.no_audio);
}

#[test]
fn requires_destination_settings() {
    let args = CliArgs::try_parse_from(["browser-stream", "--url", "https://example.com"])
        .expect("cli parse should succeed");

    let err = args.into_config().expect_err("validation should fail");
    assert_matches!(err, ConfigError::Rtmp(RtmpError::MissingDestination));
}

#[test]
fn output_precedence_wins_over_split_fields() {
    let args = CliArgs::try_parse_from([
        "browser-stream",
        "--url",
        "https://example.com",
        "--output",
        "rtmps://primary.example.com/app/final",
        "--rtmp-url",
        "rtmp://secondary.example.com/app",
        "--stream-key",
        "secondary",
    ])
    .expect("cli parse should succeed");

    let config = args.into_config().expect("config should validate");
    assert_eq!(config.output, "rtmps://primary.example.com/app/final");
}

#[test]
fn rejects_non_http_website_scheme() {
    let args = CliArgs::try_parse_from([
        "browser-stream",
        "--url",
        "file:///tmp/index.html",
        "--output",
        "rtmp://live.example.com/app/key",
    ])
    .expect("cli parse should succeed");

    let err = args.into_config().expect_err("validation should fail");
    assert_matches!(err, ConfigError::UnsupportedWebsiteScheme(s) if s == "file");
}

#[test]
fn rejects_out_of_range_fps() {
    let args = CliArgs::try_parse_from([
        "browser-stream",
        "--url",
        "https://example.com",
        "--output",
        "rtmp://live.example.com/app/key",
        "--fps",
        "121",
    ])
    .expect("cli parse should succeed");

    let err = args.into_config().expect_err("validation should fail");
    assert_matches!(
        err,
        ConfigError::OutOfRange { field, min: 1, max: 120, actual: 121 } if field == "fps"
    );
}

#[test]
fn rejects_out_of_range_frame_timeout() {
    let args = CliArgs::try_parse_from([
        "browser-stream",
        "--url",
        "https://example.com",
        "--output",
        "rtmp://live.example.com/app/key",
        "--frame-timeout-ms",
        "500",
    ])
    .expect("cli parse should succeed");

    let err = args.into_config().expect_err("validation should fail");
    assert_matches!(
        err,
        ConfigError::OutOfRange {
            field,
            min: 1000,
            max,
            actual: 500
        } if field == "frame-timeout-ms" && max == u64::MAX
    );
}
