//! Decoder — video frame decoding via FFmpeg.
//!
//! **Phase 1 stubs.** All methods will be implemented in Phase 2 using the
//! `ffmpeg-next` crate once the FFmpeg C library is available via vcpkg.

use std::path::Path;

use anyhow::Result;

// ─── MediaInfo ───────────────────────────────────────────────────────────────

/// Metadata probed from a media file.
///
/// Phase 2: populated by `ffmpeg_next::format::input()`.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MediaInfo {
    /// Width of the video stream in pixels.
    pub width: u32,
    /// Height of the video stream in pixels.
    pub height: u32,
    /// Average frame-rate of the video stream.
    pub fps: f64,
    /// Total duration of the file in seconds.
    pub duration_secs: f64,
    /// `true` if the file contains at least one audio stream.
    pub has_audio: bool,
}

// ─── MediaDecoder ────────────────────────────────────────────────────────────

/// Decodes video frames from a media file.
///
/// Phase 2: wraps an `ffmpeg_next` format context + codec context.
#[allow(dead_code)]
pub struct MediaDecoder {
    // Phase 2: ffmpeg_next::format::context::Input
    _private: (),
}

impl MediaDecoder {
    /// Open a media file and probe its metadata.
    ///
    /// # Phase 2
    /// Will use `ffmpeg_next::format::input(path)` to open the container and
    /// extract stream information.
    pub fn open(_path: &Path) -> Result<(Self, MediaInfo)> {
        todo!("Phase 2: ffmpeg-next probe + open")
    }

    /// Seek to `timestamp_secs` and decode one video frame as raw RGBA bytes.
    ///
    /// # Phase 2
    /// Will seek the decoder context, decode a packet, scale the frame to
    /// RGBA via `libswscale`, and return the pixel buffer.
    pub fn decode_frame(&mut self, _timestamp_secs: f64) -> Result<Vec<u8>> {
        todo!("Phase 2: seek + decode → RGBA bytes")
    }
}
