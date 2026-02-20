use thiserror::Error;
use url::Url;

#[derive(Debug, Error, PartialEq)]
pub enum RtmpError {
    #[error("provide either `--output` or both `--rtmp-url` and `--stream-key`")]
    MissingDestination,
    #[error("stream key cannot be empty")]
    EmptyStreamKey,
    #[error("invalid RTMP output URL `{0}`")]
    InvalidOutputUrl(String),
    #[error("RTMP URL scheme must be `rtmp` or `rtmps`, got `{0}`")]
    InvalidScheme(String),
}

pub fn build_output(
    output: Option<String>,
    rtmp_url: Option<String>,
    stream_key: Option<String>,
) -> Result<String, RtmpError> {
    if let Some(full_output) = output {
        let trimmed = full_output.trim();
        validate_output_url(trimmed)?;
        return Ok(trimmed.to_string());
    }

    match (rtmp_url, stream_key) {
        (Some(base), Some(key)) => {
            let normalized_key = normalize_stream_key(&key)?;
            let merged = format!("{}/{}", base.trim_end_matches('/'), normalized_key);
            validate_output_url(&merged)?;
            Ok(merged)
        }
        _ => Err(RtmpError::MissingDestination),
    }
}

fn normalize_stream_key(raw: &str) -> Result<String, RtmpError> {
    let key = raw.trim().trim_start_matches('/').trim();
    if key.is_empty() {
        return Err(RtmpError::EmptyStreamKey);
    }

    Ok(key.to_string())
}

fn validate_output_url(candidate: &str) -> Result<(), RtmpError> {
    let parsed =
        Url::parse(candidate).map_err(|_| RtmpError::InvalidOutputUrl(candidate.to_string()))?;
    match parsed.scheme() {
        "rtmp" | "rtmps" => Ok(()),
        other => Err(RtmpError::InvalidScheme(other.to_string())),
    }
}
