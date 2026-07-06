//! `MoveClipCommand` — moves a clip to a new timeline position.
//!
//! On **execute**: updates `timeline_start` and `timeline_end` of the clip.
//! On **undo**: restores the original position.

use uuid::Uuid;
use crate::{commands::Command, Timeline};

/// Moves a clip identified by `clip_id` by `delta_secs` seconds.
pub struct MoveClipCommand {
    /// The clip to move.
    pub clip_id: Uuid,
    /// The track the clip is on.
    pub track_id: Uuid,
    /// Original timeline start (for undo).
    pub old_start: f64,
    /// New timeline start (for execute).
    pub new_start: f64,
    /// Duration of the clip (unchanged by move).
    pub duration: f64,
}

impl MoveClipCommand {
    /// Create a move command for a clip that starts at `old_start`.
    pub fn new(clip_id: Uuid, track_id: Uuid, old_start: f64, new_start: f64, duration: f64) -> Self {
        Self { clip_id, track_id, old_start, new_start, duration }
    }
}

impl Command for MoveClipCommand {
    fn execute(&mut self, timeline: &mut Timeline) {
        apply_move(timeline, self.track_id, self.clip_id, self.new_start, self.duration);
    }

    fn undo(&mut self, timeline: &mut Timeline) {
        apply_move(timeline, self.track_id, self.clip_id, self.old_start, self.duration);
    }

    fn label(&self) -> &str {
        "Move Clip"
    }
}

fn apply_move(timeline: &mut Timeline, track_id: Uuid, clip_id: Uuid, new_start: f64, duration: f64) {
    if let Some(track) = timeline.tracks.iter_mut().find(|t| t.id == track_id) {
        if let Some(clip) = track.clips.iter_mut().find(|c| c.id == clip_id) {
            let new_start = new_start.max(0.0);
            clip.timeline_start = new_start;
            clip.timeline_end = new_start + duration;
        }
        track.clips.sort_by(|a, b| a.timeline_start.partial_cmp(&b.timeline_start).unwrap());
    }
    timeline.recompute_duration();
}
