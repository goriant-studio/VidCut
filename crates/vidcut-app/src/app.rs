//! `VidCutApp` — top-level application state and `eframe::App` implementation.
//!
//! This struct owns the [`Project`], [`CommandHistory`], and playback state.
//! Each panel accesses and mutates app state through the public accessor/action
//! methods defined here, keeping panel code free of direct field access.

use eframe::egui;
use vidcut_core::{
    commands::CommandHistory,
    MediaAsset, Project,
};

use crate::panels;

// ─── VidCutApp ────────────────────────────────────────────────────────────────

/// Root application state.
pub struct VidCutApp {
    /// Currently open project, if any.
    project: Option<Project>,
    /// Undo/redo command history (operates on the project's timeline).
    history: CommandHistory,
    /// Current playhead position in seconds.
    playhead_secs: f64,
    /// Whether the timeline is currently playing.
    is_playing: bool,
}

impl VidCutApp {
    /// Initialise the application. Called once by eframe during startup.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        panels::theme::apply_dark_theme(&cc.egui_ctx);
        Self {
            project: None,
            history: CommandHistory::new(),
            playhead_secs: 0.0,
            is_playing: false,
        }
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    /// Current playhead position in seconds.
    pub fn playhead_secs(&self) -> f64 {
        self.playhead_secs
    }

    /// Whether the timeline is currently playing.
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    /// Whether there is a command to undo.
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Whether there is a command to redo.
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Slice of media assets in the current project's media pool.
    pub fn media_pool(&self) -> &[MediaAsset] {
        self.project
            .as_ref()
            .map(|p| p.media_pool.as_slice())
            .unwrap_or(&[])
    }

    /// Total duration of the current project in seconds (0 if no project).
    pub fn project_duration(&self) -> f64 {
        self.project
            .as_ref()
            .map(|p| p.timeline.duration_secs)
            .unwrap_or(0.0)
    }

    /// Preview resolution (width, height) from project settings.
    pub fn preview_resolution(&self) -> (u32, u32) {
        self.project
            .as_ref()
            .map(|p| (p.settings.width, p.settings.height))
            .unwrap_or((1920, 1080))
    }

    // ── Actions ───────────────────────────────────────────────────────────────

    /// Create a new empty project, discarding any unsaved changes.
    pub fn action_new_project(&mut self) {
        self.project = Some(Project::new("Untitled Project"));
        self.history = CommandHistory::new();
        self.playhead_secs = 0.0;
        self.is_playing = false;
        tracing::info!("New project created");
    }

    /// Open a project from disk (Phase 2: show OS file dialog).
    pub fn action_open_project(&mut self) {
        // Phase 2: use IFileOpenDialog via windows-rs to pick a .vidcut file,
        // then call Project::load(path).
        tracing::info!("Open project — Phase 2");
    }

    /// Save the current project to disk (Phase 2: show OS save dialog if no path).
    pub fn action_save_project(&mut self) {
        // Phase 2: if project has a known path, call Project::save(path).
        // Otherwise open IFileSaveDialog.
        tracing::info!("Save project — Phase 2");
    }

    /// Start an export session (Phase 2: show export dialog, run ExportEncoder).
    pub fn action_export(&mut self) {
        tracing::info!("Export — Phase 2");
    }

    /// Import one or more media files (Phase 2: IFileOpenDialog + ffmpeg probe).
    pub fn action_import_media(&mut self) {
        tracing::info!("Import media — Phase 2");
    }

    /// Add a new default video track to the current project's timeline.
    pub fn action_add_track(&mut self) {
        if let Some(project) = &mut self.project {
            use vidcut_core::{Track, TrackType};
            let n = project.timeline.tracks.len() + 1;
            project.timeline.tracks.push(Track::new(
                format!("Video {n}"),
                TrackType::Video,
            ));
            tracing::info!("Track added");
        }
    }

    /// Toggle play/pause.
    pub fn action_play_pause(&mut self) {
        self.is_playing = !self.is_playing;
        tracing::debug!("Play: {}", self.is_playing);
    }

    /// Stop playback and reset playhead to 0.
    pub fn action_stop(&mut self) {
        self.is_playing = false;
        self.playhead_secs = 0.0;
    }

    /// Undo the most recent command.
    pub fn action_undo(&mut self) {
        if let Some(project) = &mut self.project {
            self.history.undo(&mut project.timeline);
        }
    }

    /// Redo the next command.
    pub fn action_redo(&mut self) {
        if let Some(project) = &mut self.project {
            self.history.redo(&mut project.timeline);
        }
    }
}

// ─── eframe::App ─────────────────────────────────────────────────────────────

impl eframe::App for VidCutApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Advance playhead if playing (Phase 2: sync to real frame clock).
        if self.is_playing {
            let dt = ctx.input(|i| i.unstable_dt) as f64;
            self.playhead_secs += dt;
            ctx.request_repaint();
        }

        // Keyboard shortcuts
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                self.action_play_pause();
            }
        });

        // Panels — order matters for egui layout resolution:
        // top → bottom panels first, then side panels, then central.
        panels::toolbar::show(ctx, self);
        panels::timeline::show(ctx, self);
        panels::media_browser::show(ctx, self);
        panels::inspector::show(ctx, self);
        panels::preview::show(ctx, self);
    }
}
