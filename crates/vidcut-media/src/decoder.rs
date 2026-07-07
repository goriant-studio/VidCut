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
    process::{Command, Stdio},
};

use anyhow::Result;
use tracing::{info, warn};

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
/// Uses pure-Rust libraries when possible:
/// - `mp4` crate for MP4/M4V containers
/// - `symphonia` crate for audio files
/// - Extension-based detection for images (no decode needed)
/// - Falls back to `ffprobe` CLI for formats not covered above
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

// ─── Video probe (mp4 crate + ffprobe fallback) ──────────────────────────────

fn probe_video(path: &Path, kind: AssetKind) -> MediaInfo {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    // Try the pure-Rust `mp4` crate for MP4/M4V first.
    if matches!(ext.as_deref(), Some("mp4" | "m4v")) {
        let info = probe_mp4(path, kind.clone());
        if info.duration_secs > 0.0 {
            return info;
        }
        // If mp4 crate returned 0 duration, fall through to ffprobe.
        warn!("mp4 crate returned 0 duration for {:?} — trying ffprobe", path.file_name());
    }

    // For MOV/MKV/AVI/WebM/etc. (or MP4 fallback), use ffprobe.
    match probe_ffprobe(path, kind.clone()) {
        Some(info) => info,
        None => {
            warn!(
                "Cannot probe duration for {:?} — ffprobe unavailable or failed",
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

// ─── ffprobe-based probe ─────────────────────────────────────────────────────

/// Resolve the path to the `ffprobe` binary.
///
/// Prefers the sidecar-managed binary next to the sidecar `ffmpeg`.
/// Falls back to system `ffprobe` in PATH.
fn ffprobe_bin() -> String {
    // The sidecar puts ffprobe next to ffmpeg in the same directory.
    if let Ok(ffmpeg_path) = ffmpeg_sidecar::paths::sidecar_path() {
        let probe_path = ffmpeg_path.with_file_name(
            if cfg!(target_os = "windows") { "ffprobe.exe" } else { "ffprobe" }
        );
        if probe_path.exists() {
            return probe_path.to_string_lossy().into_owned();
        }
    }
    // Fall back to system PATH.
    "ffprobe".to_string()
}

/// Probe a media file using the `ffprobe` CLI to extract duration, resolution,
/// fps, and audio presence.
fn probe_ffprobe(path: &Path, kind: AssetKind) -> Option<MediaInfo> {
    let bin = ffprobe_bin();
    let output = Command::new(&bin)
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(path.as_os_str())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        Ok(o) => {
            warn!("ffprobe exited with status {} for {:?}", o.status, path.file_name());
            return None;
        }
        Err(e) => {
            warn!("Failed to run ffprobe ({bin}): {e}");
            return None;
        }
    };

    let json_str = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => {
            warn!("Failed to parse ffprobe JSON: {e}");
            return None;
        }
    };

    // ── Extract container-level duration ─────────────────────────────────────
    let duration_secs = json["format"]["duration"]
        .as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    // ── Scan streams ────────────────────────────────────────────────────────
    let mut width = 0u32;
    let mut height = 0u32;
    let mut fps = 0.0f64;
    let mut has_audio = false;

    if let Some(streams) = json["streams"].as_array() {
        for stream in streams {
            let codec_type = stream["codec_type"].as_str().unwrap_or("");
            match codec_type {
                "video" => {
                    if width == 0 {
                        width = stream["width"].as_u64().unwrap_or(0) as u32;
                        height = stream["height"].as_u64().unwrap_or(0) as u32;

                        // Parse r_frame_rate (e.g. "30000/1001" or "30/1")
                        if let Some(rfr) = stream["r_frame_rate"].as_str() {
                            if let Some((num_s, den_s)) = rfr.split_once('/') {
                                if let (Ok(num), Ok(den)) = (
                                    num_s.trim().parse::<f64>(),
                                    den_s.trim().parse::<f64>(),
                                ) {
                                    if den > 0.0 {
                                        fps = num / den;
                                    }
                                }
                            }
                        }
                    }
                }
                "audio" => {
                    has_audio = true;
                }
                _ => {}
            }
        }
    }

    info!(
        "ffprobe {:?}: {:.2}s, {}×{}, {:.2} fps, audio={}",
        path.file_name(),
        duration_secs,
        width,
        height,
        fps,
        has_audio,
    );

    Some(MediaInfo {
        width,
        height,
        fps,
        duration_secs,
        has_audio,
        kind,
    })
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
