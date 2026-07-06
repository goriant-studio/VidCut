//! Thumbnail generator — produces small preview images for media assets.
//!
//! **Phase 1 stub.** Phase 2 will use `tokio::task::spawn_blocking` to decode
//! a single frame via `ffmpeg-next` and scale it to a thumbnail-sized RGBA
//! buffer, then upload it to an `egui::TextureHandle`.

use std::path::Path;

use anyhow::Result;

/// Generates thumbnail images for media assets.
///
/// Phase 2: runs FFmpeg frame decode on a blocking Tokio thread so the UI
/// stays responsive during thumbnail generation.
#[allow(dead_code)]
pub struct ThumbnailGenerator {
    _private: (),
}

impl ThumbnailGenerator {
    /// Create a new generator.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Generate a thumbnail at the given `timestamp_secs` offset in the file.
    ///
    /// Returns raw RGBA bytes of size `(width × height × 4)`.
    ///
    /// # Phase 2
    /// Will open `path` with `ffmpeg-next`, seek to `timestamp_secs`, decode
    /// one frame, and scale to `(width, height)` via libswscale.
    pub fn generate(
        &self,
        _path: &Path,
        _timestamp_secs: f64,
        _width: u32,
        _height: u32,
    ) -> Result<Vec<u8>> {
        todo!("Phase 2: async thumbnail via ffmpeg-next + tokio::task::spawn_blocking")
    }
}

impl Default for ThumbnailGenerator {
    fn default() -> Self {
        Self::new()
    }
}
