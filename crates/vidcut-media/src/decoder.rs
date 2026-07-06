//! Decoder — media file probing and frame decoding.
//!
//! **Phase 2 (basic):** Pure-Rust probe using `mp4` (for MP4/M4V containers)
//! and `symphonia` (for audio files). No FFmpeg C-library required.
//!
//! Supported for probing:
//! - Video:  `.mp4`, `.m4v`
//! - Audio:  `.mp3`, `.wav`, `.flac`, `.ogg`, `.aac`, `.m4a`
//! - Image:  `.png`, `.jpg`, `.jpeg`, `.bmp`, `.webp` (duration = 0)
//!
//! Unknown formats fall back gracefully (duration = 0, no crash).

use std::{
    fs::File,
    io::BufReader,
    path::Path,
};

use anyhow::Result;
use tracing::warn;

// ─── AssetKind ───────────────────────────────────────────────────────────────

/// Detected media kind from file extension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetKind {
    Video,
    Audio,
    Image,
    Unknown,
}

impl AssetKind {
    pub fn from_path(path: &Path) -> Self {
        match path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .as_deref()
        {
            Some("mp4" | "m4v" | "mov" | "avi" | "mkv" | "wmv" | "webm" | "flv") => Self::Video,
            Some("mp3" | "wav" | "flac" | "ogg" | "aac" | "m4a" | "opus" | "wma") => Self::Audio,
            Some("png" | "jpg" | "jpeg" | "bmp" | "webp" | "gif" | "tiff" | "tif") => Self::Image,
            _ => Self::Unknown,
        }
    }
}

// ─── MediaInfo ───────────────────────────────────────────────────────────────

/// Metadata probed from a media file.
#[derive(Debug, Clone)]
pub struct MediaInfo {
    /// Width of the video stream in pixels (0 for audio-only / images).
    pub width: u32,
    /// Height of the video stream in pixels (0 for audio-only / images).
    pub height: u32,
    /// Average frame-rate of the video stream (0.0 for audio-only).
    pub fps: f64,
    /// Total duration of the file in seconds (0.0 for images).
    pub duration_secs: f64,
    /// `true` if the file contains at least one audio stream.
    pub has_audio: bool,
    /// Detected asset kind.
    pub kind: AssetKind,
}

impl Default for MediaInfo {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            fps: 0.0,
            duration_secs: 0.0,
            has_audio: false,
            kind: AssetKind::Unknown,
        }
    }
}

// ─── probe_file ──────────────────────────────────────────────────────────────

/// Probe a media file and return its [`MediaInfo`].
///
/// Uses pure-Rust libraries:
/// - `mp4` crate for MP4/M4V containers
/// - `symphonia` crate for audio files
/// - Extension-based detection for images (no decode needed)
///
/// Never panics — returns defaults on any error.
pub fn probe_file(path: &Path) -> MediaInfo {
    let kind = AssetKind::from_path(path);
    match &kind {
        AssetKind::Video => probe_video(path, kind),
        AssetKind::Audio => probe_audio(path, kind),
        AssetKind::Image => MediaInfo {
            kind: AssetKind::Image,
            ..Default::default()
        },
        AssetKind::Unknown => {
            // Try audio probe as fallback (symphonia is permissive)
            let info = probe_audio(path, AssetKind::Unknown);
            if info.duration_secs > 0.0 {
                info
            } else {
                MediaInfo {
                    kind: AssetKind::Unknown,
                    ..Default::default()
                }
            }
        }
    }
}

// ─── Video probe (mp4 crate) ─────────────────────────────────────────────────

fn probe_video(path: &Path, kind: AssetKind) -> MediaInfo {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    // Only `mp4` crate can parse MP4/M4V; fall back for others.
    match ext.as_deref() {
        Some("mp4" | "m4v") => probe_mp4(path, kind),
        _ => {
            // For MOV/MKV/AVI etc. we can't probe without FFmpeg.
            // Return a stub with kind=Video and duration=0 — still usable.
            warn!(
                "Cannot probe duration for {:?} without FFmpeg — using defaults",
                path.file_name()
            );
            MediaInfo {
                kind,
                has_audio: true, // Assume has audio
                ..Default::default()
            }
        }
    }
}

fn probe_mp4(path: &Path, kind: AssetKind) -> MediaInfo {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            warn!("Cannot open {:?}: {e}", path);
            return MediaInfo { kind, ..Default::default() };
        }
    };

    let size = file.metadata().map(|m| m.len()).unwrap_or(0);
    let reader = BufReader::new(file);

    let mp4 = match mp4::Mp4Reader::read_header(reader, size) {
        Ok(m) => m,
        Err(e) => {
            warn!("mp4 probe failed for {:?}: {e}", path);
            return MediaInfo { kind, ..Default::default() };
        }
    };

    let duration_secs = mp4.duration().as_secs_f64();
    let mut width = 0u32;
    let mut height = 0u32;
    let mut fps = 0.0f64;
    let mut has_audio = false;

    for track in mp4.tracks().values() {
        use mp4::TrackType;
        match track.track_type() {
            Ok(TrackType::Video) => {
                width = track.width() as u32;
                height = track.height() as u32;
                let frame_rate = track.frame_rate();
                if frame_rate > 0.0 {
                    fps = frame_rate as f64;
                }
            }
            Ok(TrackType::Audio) => {
                has_audio = true;
            }
            _ => {}
        }
    }

    MediaInfo {
        width,
        height,
        fps,
        duration_secs,
        has_audio,
        kind,
    }
}

// ─── Audio probe (symphonia) ─────────────────────────────────────────────────

fn probe_audio(path: &Path, kind: AssetKind) -> MediaInfo {
    use symphonia::core::{
        formats::FormatOptions,
        io::MediaSourceStream,
        meta::MetadataOptions,
        probe::Hint,
    };

    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            warn!("Cannot open {:?}: {e}", path);
            return MediaInfo { kind, ..Default::default() };
        }
    };

    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = match symphonia::default::get_probe().format(
        &hint,
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    ) {
        Ok(p) => p,
        Err(e) => {
            warn!("symphonia probe failed for {:?}: {e}", path);
            return MediaInfo { kind, ..Default::default() };
        }
    };

    let format = probed.format;
    let mut duration_secs = 0.0f64;

    for track in format.tracks() {
        if let Some(n_frames) = track.codec_params.n_frames {
            if let Some(tb) = track.codec_params.time_base {
                let secs = n_frames as f64 * tb.numer as f64 / tb.denom as f64;
                if secs > duration_secs {
                    duration_secs = secs;
                }
            }
        }
    }

    MediaInfo {
        duration_secs,
        has_audio: true,
        kind,
        ..Default::default()
    }
}

// ─── MediaDecoder (Phase 3 — frame decode) ───────────────────────────────────

/// Decodes video frames from a media file.
///
/// Phase 3: will wrap ffmpeg-next for full decode pipeline.
#[allow(dead_code)]
pub struct MediaDecoder {
    _private: (),
}

impl MediaDecoder {
    /// Open a media file for frame-by-frame decoding.
    ///
    /// Phase 3: use ffmpeg-next.
    pub fn open(_path: &Path) -> Result<(Self, MediaInfo)> {
        todo!("Phase 3: ffmpeg-next full decode")
    }

    /// Decode one video frame as raw RGBA bytes at `timestamp_secs`.
    pub fn decode_frame(&mut self, _timestamp_secs: f64) -> Result<Vec<u8>> {
        todo!("Phase 3: seek + decode → RGBA bytes")
    }
}
