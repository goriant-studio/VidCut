//! Clip — a segment of a media asset placed on a timeline track.
//!
//! A [`Clip`] records both *where* it sits on the timeline
//! (`timeline_start` / `timeline_end`) and *which portion* of the source
//! asset it plays (`source_start` / `source_end`), enabling non-destructive
//! trimming.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── ClipTransform ────────────────────────────────────────────────────────────

/// 2D spatial transform for a clip in the preview canvas.
///
/// All values are in "logical" units:
/// - `x` / `y`: offset in pixels from the canvas centre.
/// - `scale_x` / `scale_y`: multiplicative scale (1.0 = 100 %).
/// - `rotation`: clockwise rotation in degrees.
/// - `opacity`: transparency in the range \[0.0, 1.0\].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipTransform {
    pub x: f64,
    pub y: f64,
    pub scale_x: f64,
    pub scale_y: f64,
    pub rotation: f64,
    pub opacity: f64,
}

impl Default for ClipTransform {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
            opacity: 1.0,
        }
    }
}

// ─── Clip ────────────────────────────────────────────────────────────────────

/// A trimmed region of a [`MediaAsset`] placed at a specific position on a
/// [`Track`].
///
/// All time values are in seconds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    /// Unique identifier for this clip instance.
    pub id: Uuid,
    /// The [`MediaAsset`] this clip references (by asset id).
    pub asset_id: Uuid,
    /// The [`Track`] this clip belongs to (by track id).
    pub track_id: Uuid,
    /// Start position on the timeline (seconds from the timeline origin).
    pub timeline_start: f64,
    /// End position on the timeline (seconds from the timeline origin).
    pub timeline_end: f64,
    /// Start position within the source file (seconds from the file origin).
    pub source_start: f64,
    /// End position within the source file (seconds from the file origin).
    pub source_end: f64,
    /// 2D transform applied to this clip in the preview.
    #[serde(default)]
    pub transform: ClipTransform,
}

impl Clip {
    /// Duration of this clip as it appears on the timeline.
    #[inline]
    pub fn duration(&self) -> f64 {
        self.timeline_end - self.timeline_start
    }
}
