//! `AddClipCommand` — places a new clip onto a track.
//!
//! On **execute** the clip is inserted into the correct track.
//! On **undo** the clip is removed again by its id.

use crate::{commands::Command, Clip, Timeline};

/// Inserts `clip` into the track identified by `clip.track_id`.
pub struct AddClipCommand {
    /// The clip to add.
    pub clip: Clip,
}

impl AddClipCommand {
    /// Create a new command that will add `clip` to the timeline.
    pub fn new(clip: Clip) -> Self {
        Self { clip }
    }
}

impl Command for AddClipCommand {
    fn execute(&mut self, timeline: &mut Timeline) {
        if let Some(track) = timeline.tracks.iter_mut().find(|t| t.id == self.clip.track_id) {
            track.clips.push(self.clip.clone());
            // Keep clips sorted by start time.
            track
                .clips
                .sort_by(|a, b| a.timeline_start.partial_cmp(&b.timeline_start).unwrap());
        }
        timeline.recompute_duration();
    }

    fn undo(&mut self, timeline: &mut Timeline) {
        if let Some(track) = timeline.tracks.iter_mut().find(|t| t.id == self.clip.track_id) {
            track.clips.retain(|c| c.id != self.clip.id);
        }
        timeline.recompute_duration();
    }

    fn label(&self) -> &str {
        "Add Clip"
    }
}
