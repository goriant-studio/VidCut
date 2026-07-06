//! Timeline — the master sequence of tracks and their clips.
//!
//! A [`Timeline`] owns all [`Track`]s. The computed duration is derived
//! from the latest clip end-point across all tracks, though it can also
//! be set explicitly for sequences with tailing silence/black.

use serde::{Deserialize, Serialize};

use crate::Track;

/// The master sequence that holds all tracks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    /// Ordered list of video and audio tracks.
    pub tracks: Vec<Track>,
    /// Total duration of the timeline in seconds.
    /// May be set explicitly or computed from clip extents.
    pub duration_secs: f64,
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            tracks: Vec::new(),
            duration_secs: 0.0,
        }
    }
}

impl Timeline {
    /// Recompute `duration_secs` from the maximum clip end point across all tracks.
    pub fn recompute_duration(&mut self) {
        self.duration_secs = self
            .tracks
            .iter()
            .flat_map(|t| t.clips.iter().map(|c| c.timeline_end))
            .fold(0.0_f64, f64::max);
    }
}
