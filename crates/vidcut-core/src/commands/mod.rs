//! Command system — undo/redo via the Command pattern.
//!
//! Every user action that modifies the [`Timeline`] is encapsulated as a
//! [`Command`]. [`CommandHistory`] maintains a cursor into the command stack,
//! enabling arbitrarily deep undo and redo.

pub mod add_clip;
pub mod delete_clip;
pub mod move_clip;
pub mod trim_clip;

use crate::Timeline;

// ─── Command trait ────────────────────────────────────────────────────────────

/// An invertible operation that mutates a [`Timeline`].
///
/// Implement this trait for every user action that should be undoable.
pub trait Command: Send + Sync {
    /// Apply the command to `timeline`.
    fn execute(&mut self, timeline: &mut Timeline);
    /// Reverse the effect of a previous [`execute`](Command::execute) call.
    fn undo(&mut self, timeline: &mut Timeline);
    /// Human-readable label shown in the Undo/Redo menu items.
    fn label(&self) -> &str;
}

// ─── CommandHistory ───────────────────────────────────────────────────────────

/// Maintains a bounded stack of executed commands with an undo cursor.
///
/// The cursor points to the index *after* the last executed command:
/// - `cursor == 0` means nothing can be undone.
/// - `cursor == history.len()` means nothing can be redone.
pub struct CommandHistory {
    history: Vec<Box<dyn Command>>,
    cursor: usize,
}

impl CommandHistory {
    /// Create an empty history.
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            cursor: 0,
        }
    }

    /// Execute `command` against `timeline` and push it onto the history stack.
    ///
    /// Any commands beyond the current cursor (the "redo stack") are discarded.
    pub fn push(&mut self, mut command: Box<dyn Command>, timeline: &mut Timeline) {
        // Discard any future redo states.
        self.history.truncate(self.cursor);
        command.execute(timeline);
        self.history.push(command);
        self.cursor += 1;
    }

    /// Undo the most recent command. Returns `false` if there is nothing to undo.
    pub fn undo(&mut self, timeline: &mut Timeline) -> bool {
        if self.cursor == 0 {
            return false;
        }
        self.cursor -= 1;
        self.history[self.cursor].undo(timeline);
        true
    }

    /// Redo the next command. Returns `false` if there is nothing to redo.
    pub fn redo(&mut self, timeline: &mut Timeline) -> bool {
        if self.cursor >= self.history.len() {
            return false;
        }
        self.history[self.cursor].execute(timeline);
        self.cursor += 1;
        true
    }

    /// Returns `true` if there is at least one command to undo.
    pub fn can_undo(&self) -> bool {
        self.cursor > 0
    }

    /// Returns `true` if there is at least one command to redo.
    pub fn can_redo(&self) -> bool {
        self.cursor < self.history.len()
    }

    /// Label of the command that would be undone next, if any.
    pub fn undo_label(&self) -> Option<&str> {
        self.cursor.checked_sub(1).map(|i| self.history[i].label())
    }

    /// Label of the command that would be redone next, if any.
    pub fn redo_label(&self) -> Option<&str> {
        self.history.get(self.cursor).map(|c| c.label())
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}
