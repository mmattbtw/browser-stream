use anyhow::{Context, Result};
use base64::Engine;

#[derive(Debug, Clone)]
pub struct RgbFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

pub fn decode_screencast_frame(
    encoded_data: &str,
    target_width: u32,
    target_height: u32,
) -> Result<RgbFrame> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded_data)
        .context("failed to decode CDP frame payload")?;

    let img = image::load_from_memory(&bytes).context("failed to decode image bytes")?;
    let normalized = if img.width() == target_width && img.height() == target_height {
        img
    } else {
        img.resize_exact(
            target_width,
            target_height,
            image::imageops::FilterType::Triangle,
        )
    };

    let rgb = normalized.to_rgb8();
    Ok(RgbFrame {
        width: target_width,
        height: target_height,
        data: rgb.into_raw(),
    })
}
