use assert_matches::assert_matches;

use browser_stream::rtmp::{RtmpError, build_output};

#[test]
fn builds_output_from_split_fields() {
    let output = build_output(
        None,
        Some("rtmp://live.example.com/app".to_string()),
        Some("streamkey123".to_string()),
    )
    .expect("build should succeed");

    assert_eq!(output, "rtmp://live.example.com/app/streamkey123");
}

#[test]
fn trims_slashes_and_spaces_in_stream_key() {
    let output = build_output(
        None,
        Some("rtmp://live.example.com/app/".to_string()),
        Some(" /abc123  ".to_string()),
    )
    .expect("build should succeed");

    assert_eq!(output, "rtmp://live.example.com/app/abc123");
}

#[test]
fn output_flag_takes_precedence() {
    let output = build_output(
        Some("rtmps://primary.example.com/live/final".to_string()),
        Some("rtmp://secondary.example.com/app".to_string()),
        Some("secondary".to_string()),
    )
    .expect("build should succeed");

    assert_eq!(output, "rtmps://primary.example.com/live/final");
}

#[test]
fn rejects_invalid_scheme() {
    let err = build_output(Some("https://example.com/not-rtmp".to_string()), None, None)
        .expect_err("should fail");

    assert_matches!(err, RtmpError::InvalidScheme(s) if s == "https");
}

#[test]
fn rejects_empty_key() {
    let err = build_output(
        None,
        Some("rtmp://live.example.com/app".to_string()),
        Some("   /  ".to_string()),
    )
    .expect_err("should fail");

    assert_matches!(err, RtmpError::EmptyStreamKey);
}
