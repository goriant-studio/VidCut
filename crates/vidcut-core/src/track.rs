//! Track — a single row on the timeline (video or audio).
//!
//! Each track contains an ordered list of [`Clip`]s. Tracks can be muted,
//! soloed, or locked to prevent accidental edits.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Clip;

/// Discriminates between video and audio tracks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackType {
    /// Carries video frames (and optionally embedded audio).
    Video,
    /// Carries audio-only content.
    Audio,
}

/// A single timeline row containing ordered clips.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    /// Unique identifier for this track.
    pub id: Uuid,
    /// Display name shown in the track header.
    pub name: String,
    /// Whether this track carries video or audio content.
    pub track_type: TrackType,
    /// Clips placed on this track, sorted by `timeline_start`.
    pub clips: Vec<Clip>,
    /// When `true`, this track produces silence / black during playback.
    pub muted: bool,
    /// When `true`, edits to this track are blocked in the UI.
    pub locked: bool,
}

impl Track {
    /// Create a new empty track of the given type.
    pub fn new(name: impl Into<String>, track_type: TrackType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            track_type,
            clips: Vec::new(),
            muted: false,
            locked: false,
        }
    }
}
