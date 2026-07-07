//! Frame extractor — decodes a single video frame at a given timestamp.
//!
//! Uses the `ffmpeg` CLI binary (managed by `ffmpeg-sidecar`) to seek to a
//! specific timestamp and output exactly one raw RGBA frame to stdout.
//!
//! This is intentionally a **short-lived process per frame** rather than a
//! persistent decode pipeline.  It is well-suited for scrubbing and moderate
//! playback rates.  Phase 3 will replace this with a persistent in-process
//! decoder for full real-time playback.

use std::{
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{bail, Context, Result};
use tracing::{debug, warn};

// ─── DecodedFrame ────────────────────────────────────────────────────────────

/// A single decoded video frame as raw RGBA pixel data.
#[derive(Debug, Clone)]
pub struct DecodedFrame {
    /// Width of the decoded frame in pixels.
    pub width: u32,
    /// Height of the decoded frame in pixels.
    pub height: u32,
    /// Raw RGBA pixel data, length = `width * height * 4`.
    pub rgba: Vec<u8>,
}

// ─── extract_frame ───────────────────────────────────────────────────────────

/// Extract a single RGBA frame from `path` at `timestamp_secs`.
///
/// The frame is scaled to exactly `out_width` pixels wide (height is computed
/// automatically to preserve the aspect ratio, rounded to an even number).
///
/// Uses the sidecar-managed `ffmpeg` binary; falls back to the system PATH.
///
/// # Errors
///
/// Returns an error if ffmpeg cannot be found, the file cannot be opened,
/// or the output is not a valid RGBA frame.
pub fn extract_frame(
    path: &Path,
    timestamp_secs: f64,
    out_width: u32,
) -> Result<DecodedFrame> {
    let ffmpeg_bin = ffmpeg_sidecar::paths::ffmpeg_path();

    debug!(
        "extract_frame: path={:?} ts={:.3}s width={}  ffmpeg={:?}",
        path.file_name(),
        timestamp_secs,
        out_width,
        ffmpeg_bin,
    );

    // Use `Command::output()` which captures stdout + stderr and waits.
    // This avoids deadlock issues with piped stdout/stderr.
    //
    // -ss before -i  ⇒  fast input seeking (demuxer-level seek).
    // -vframes 1     ⇒  output exactly one frame.
    // scale=W:-2     ⇒  scale to fixed width, auto height (even).
    // -f rawvideo -pix_fmt rgba  ⇒  raw RGBA bytes to stdout.
    let output = Command::new(&ffmpeg_bin)
        .args(["-ss", &format!("{:.6}", timestamp_secs)])
        .arg("-i")
        .arg(path.as_os_str())
        .args(["-vframes", "1"])
        .args(["-vf", &format!("scale={}:-2", out_width)])
        .args(["-f", "rawvideo", "-pix_fmt", "rgba"])
        .args(["-v", "error"])
        .arg("pipe:1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to run ffmpeg for frame extraction")?;

    // Log any stderr output (errors / warnings from ffmpeg).
    if !output.stderr.is_empty() {
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        warn!("ffmpeg stderr: {}", stderr_str.trim());
    }

    if !output.status.success() {
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        bail!(
            "ffmpeg exited with status {} for {:?}: {}",
            output.status,
            path.file_name(),
            stderr_str.trim()
        );
    }

    let rgba = output.stdout;

    if rgba.is_empty() {
        bail!(
            "ffmpeg produced no output for {:?} at {:.3}s",
            path.file_name(),
            timestamp_secs
        );
    }

    // Determine height from the byte count.
    // We know: width = out_width, each pixel = 4 bytes (RGBA).
    let stride = out_width as usize * 4;
    if rgba.len() % stride != 0 {
        bail!(
            "Frame data length {} is not divisible by stride {} (width={})",
            rgba.len(),
            stride,
            out_width
        );
    }

    let out_height = (rgba.len() / stride) as u32;

    debug!(
        "extract_frame OK: {}×{} ({} bytes)",
        out_width, out_height, rgba.len()
    );

    Ok(DecodedFrame {
        width: out_width,
        height: out_height,
        rgba,
    })
}
