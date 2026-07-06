//! `DeleteClipCommand` — removes a clip from a track.
//!
//! On **execute**: removes the clip from its track.
//! On **undo**: re-inserts the clip at its original position.

use crate::{commands::Command, Clip, Timeline};

/// Removes a clip from the timeline.
pub struct DeleteClipCommand {
    /// The clip to delete (kept here for undo).
    pub clip: Clip,
}

impl DeleteClipCommand {
    /// Create a delete command for the given clip.
    pub fn new(clip: Clip) -> Self {
        Self { clip }
    }
}

impl Command for DeleteClipCommand {
    fn execute(&mut self, timeline: &mut Timeline) {
        if let Some(track) = timeline.tracks.iter_mut().find(|t| t.id == self.clip.track_id) {
            track.clips.retain(|c| c.id != self.clip.id);
        }
        timeline.recompute_duration();
    }

    fn undo(&mut self, timeline: &mut Timeline) {
        if let Some(track) = timeline.tracks.iter_mut().find(|t| t.id == self.clip.track_id) {
            track.clips.push(self.clip.clone());
            track.clips.sort_by(|a, b| a.timeline_start.partial_cmp(&b.timeline_start).unwrap());
        }
        timeline.recompute_duration();
    }

    fn label(&self) -> &str {
        "Delete Clip"
    }
}
