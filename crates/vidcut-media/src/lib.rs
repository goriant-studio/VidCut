//! `vidcut-media` — media utilities for VidCut.
//!
//! **Phase 2 (basic):** Pure-Rust probing via `mp4` + `symphonia`.
//! No FFmpeg C-library required at this stage.
//!
//! **Phase 3:** Full decode pipeline with `ffmpeg-next`.

pub mod audio;
pub mod decoder;
pub mod encoder;
pub mod frame_cache;
pub mod thumbnail;

// Re-export the most commonly used items at crate root.
pub use decoder::{probe_file, AssetKind, MediaInfo};
