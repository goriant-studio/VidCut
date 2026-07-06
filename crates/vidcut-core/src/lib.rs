//! `vidcut-core` — Core engine for VidCut.
//!
//! This crate contains the pure data model and business logic:
//! project serialisation, timeline structure, clip/track management,
//! and the undo/redo command system.
//!
//! **No UI, no FFmpeg, no OS-specific code belongs here.**

pub mod clip;
pub mod commands;
pub mod media_asset;
pub mod project;
pub mod timeline;
pub mod track;

// Re-export the most commonly used types at crate root.
pub use clip::Clip;
pub use media_asset::{AssetType, MediaAsset};
pub use project::{Project, ProjectSettings};
pub use timeline::Timeline;
pub use track::{Track, TrackType};
