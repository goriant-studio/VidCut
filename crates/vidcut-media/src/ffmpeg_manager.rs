//! FFmpeg lifecycle manager — ensures a local FFmpeg binary is available.
//!
//! On first run this module downloads a pre-built FFmpeg binary via
//! [`ffmpeg_sidecar`] and stores it next to the executable (i.e. in
//! `sidecar_dir()`). Subsequent runs skip the download and start immediately.
//!
//! # Usage
//!
//! ```no_run
//! let (tx, rx) = std::sync::mpsc::channel();
//! std::thread::spawn(|| vidcut_media::ensure_ffmpeg(tx));
//! // poll `rx` each frame for FfmpegStatus updates
//! ```

use std::sync::mpsc;

use ffmpeg_sidecar::{
    download::auto_download,
    paths::sidecar_path,
};
use tracing::{error, info};

// ─── Public types ─────────────────────────────────────────────────────────────

/// Status updates sent from the FFmpeg setup background thread to the UI.
#[derive(Debug, Clone)]
pub enum FfmpegStatus {
    /// Checking whether FFmpeg is already present.
    Checking,
    /// Downloading FFmpeg (~70 MB). Shows an indeterminate spinner.
    Downloading,
    /// FFmpeg is ready to use.
    Ready,
    /// Setup failed with an error message.
    Failed(String),
}

// ─── ensure_ffmpeg ────────────────────────────────────────────────────────────

/// Ensure a local FFmpeg binary is available, downloading it if necessary.
///
/// Sends [`FfmpegStatus`] updates over `tx` as work progresses.
/// Intended to be called once from a background thread at application startup.
pub fn ensure_ffmpeg(tx: mpsc::Sender<FfmpegStatus>) {
    let _ = tx.send(FfmpegStatus::Checking);

    // Check whether the sidecar binary already exists on disk.
    let is_present = sidecar_path()
        .map(|p| p.exists())
        .unwrap_or(false);
    if is_present {
        info!("FFmpeg sidecar already present — skipping download.");
        let _ = tx.send(FfmpegStatus::Ready);
        return;
    }

    info!("FFmpeg sidecar not found — starting download…");
    let _ = tx.send(FfmpegStatus::Downloading);

    match auto_download() {
        Ok(()) => {
            info!("FFmpeg download complete.");
            let _ = tx.send(FfmpegStatus::Ready);
        }
        Err(e) => {
            error!("FFmpeg download failed: {e}");
            let _ = tx.send(FfmpegStatus::Failed(format!(
                "Failed to download FFmpeg: {e}\n\nCheck your internet connection and restart VidCut."
            )));
        }
    }
}

/// Returns `true` if the sidecar-managed FFmpeg binary exists on disk.
pub fn ffmpeg_ready() -> bool {
    sidecar_path()
        .map(|p| p.exists())
        .unwrap_or(false)
}
