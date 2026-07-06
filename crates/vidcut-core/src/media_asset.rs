//! MediaAsset — a file in the project's media pool.
//!
//! When the user imports a video, audio, or image file, a [`MediaAsset`] is
//! created and stored in [`Project::media_pool`]. Clips on the timeline
//! reference assets by their [`Uuid`] id.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Classifies the type of media stored in a file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetType {
    /// A file that contains video (may also contain audio streams).
    Video,
    /// An audio-only file (WAV, MP3, AAC, …).
    Audio,
    /// A still image (PNG, JPEG, …).
    Image,
}

/// A media file imported into the project's media pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAsset {
    /// Unique identifier used by [`Clip`]s to reference this asset.
    pub id: Uuid,
    /// Absolute path to the source file on disk.
    pub path: PathBuf,
    /// Display name (defaults to the file stem).
    pub name: String,
    /// Total duration of the media in seconds (0.0 for images).
    pub duration_secs: f64,
    /// Whether this is a video, audio, or image asset.
    pub asset_type: AssetType,
    /// Pixel width — `None` for audio-only assets.
    pub width: Option<u32>,
    /// Pixel height — `None` for audio-only assets.
    pub height: Option<u32>,
    /// Frame-rate — `None` for audio-only assets and images.
    pub fps: Option<f64>,
}

impl MediaAsset {
    /// Construct a new asset from a file path with probed metadata.
    ///
    /// Phase 2 will fill `duration_secs`, `width`, `height`, and `fps` via
    /// ffmpeg-next probing. For now callers supply these manually.
    pub fn new(
        path: PathBuf,
        asset_type: AssetType,
        duration_secs: f64,
        width: Option<u32>,
        height: Option<u32>,
        fps: Option<f64>,
    ) -> Self {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_owned();
        Self {
            id: Uuid::new_v4(),
            path,
            name,
            duration_secs,
            asset_type,
            width,
            height,
            fps,
        }
    }
}
