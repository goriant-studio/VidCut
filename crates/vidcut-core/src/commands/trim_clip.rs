//! `TrimClipCommand` — adjusts the in/out points of an existing clip.
//!
//! Stores the *previous* in/out values so the edit can be undone exactly.

use crate::{commands::Command, Timeline};
use uuid::Uuid;

/// Trims a clip's timeline and source in/out points.
pub struct TrimClipCommand {
    /// The clip to trim (identified by id).
    clip_id: Uuid,
    /// The track the clip lives on (for fast lookup).
    track_id: Uuid,
    // New values applied by execute ────────────────────────────────────────────
    new_timeline_start: f64,
    new_timeline_end: f64,
    new_source_start: f64,
    new_source_end: f64,
    // Previous values restored by undo ─────────────────────────────────────────
    old_timeline_start: f64,
    old_timeline_end: f64,
    old_source_start: f64,
    old_source_end: f64,
}

impl TrimClipCommand {
    /// Create a trim command.
    ///
    /// `old_*` values are the current clip state; `new_*` values are what the
    /// edit should apply.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        clip_id: Uuid,
        track_id: Uuid,
        old_timeline_start: f64,
        old_timeline_end: f64,
        old_source_start: f64,
        old_source_end: f64,
        new_timeline_start: f64,
        new_timeline_end: f64,
        new_source_start: f64,
        new_source_end: f64,
    ) -> Self {
        Self {
            clip_id,
            track_id,
            new_timeline_start,
            new_timeline_end,
            new_source_start,
            new_source_end,
            old_timeline_start,
            old_timeline_end,
            old_source_start,
            old_source_end,
        }
    }
}

impl Command for TrimClipCommand {
    fn execute(&mut self, timeline: &mut Timeline) {
        if let Some(track) = timeline.tracks.iter_mut().find(|t| t.id == self.track_id) {
            if let Some(clip) = track.clips.iter_mut().find(|c| c.id == self.clip_id) {
                clip.timeline_start = self.new_timeline_start;
                clip.timeline_end = self.new_timeline_end;
                clip.source_start = self.new_source_start;
                clip.source_end = self.new_source_end;
            }
        }
        timeline.recompute_duration();
    }

    fn undo(&mut self, timeline: &mut Timeline) {
        if let Some(track) = timeline.tracks.iter_mut().find(|t| t.id == self.track_id) {
            if let Some(clip) = track.clips.iter_mut().find(|c| c.id == self.clip_id) {
                clip.timeline_start = self.old_timeline_start;
                clip.timeline_end = self.old_timeline_end;
                clip.source_start = self.old_source_start;
                clip.source_end = self.old_source_end;
            }
        }
        timeline.recompute_duration();
    }

    fn label(&self) -> &str {
        "Trim Clip"
    }
}
