//! `VidCutApp` — top-level application state and `eframe::App` implementation.
//!
//! This struct owns the [`Project`], [`CommandHistory`], playback state,
//! and UI selection state. Each panel accesses and mutates app state through
//! the public accessor/action methods defined here.

use std::path::PathBuf;

use eframe::egui;
use uuid::Uuid;
use vidcut_core::{
    commands::{
        add_clip::AddClipCommand, delete_clip::DeleteClipCommand, move_clip::MoveClipCommand,
        CommandHistory,
    },
    Clip, MediaAsset, AssetType, Project, Track, TrackType,
};
use vidcut_media::{probe_file, AssetKind};

use crate::panels;

// ─── VidCutApp ────────────────────────────────────────────────────────────────

/// Root application state.
pub struct VidCutApp {
    /// Currently open project.
    pub project: Option<Project>,
    /// Undo/redo command history (operates on the project's timeline).
    pub history: CommandHistory,
    /// Current playhead position in seconds.
    pub playhead_secs: f64,
    /// Whether the timeline is currently playing.
    pub is_playing: bool,

    // ── Selection state ───────────────────────────────────────────────────────
    /// The asset currently highlighted in the Media Browser.
    pub selected_asset_id: Option<Uuid>,
    /// The clip currently selected on the timeline.
    pub selected_clip_id: Option<Uuid>,

    // ── Timeline view state ───────────────────────────────────────────────────
    /// Pixels per second for the timeline zoom level.
    pub timeline_px_per_sec: f32,

    // ── Drag state ────────────────────────────────────────────────────────────
    /// While dragging a clip, stores (clip_id, track_id, original_start, drag_offset_secs).
    pub dragging: Option<DragState>,
}

/// Temporary state while a clip is being dragged on the timeline.
#[derive(Debug, Clone)]
pub struct DragState {
    pub clip_id: Uuid,
    pub track_id: Uuid,
    pub original_start: f64,
    pub duration: f64,
    /// Current uncommitted start position (updated every frame during drag).
    pub current_start: f64,
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
            selected_asset_id: None,
            selected_clip_id: None,
            timeline_px_per_sec: 80.0,
            dragging: None,
        }
    }

    // ── Project helpers ───────────────────────────────────────────────────────

    /// Returns a reference to the current project, creating a default one if absent.
    pub fn ensure_project(&mut self) -> &mut Project {
        if self.project.is_none() {
            self.project = Some(Project::new("Untitled Project"));
        }
        self.project.as_mut().unwrap()
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    pub fn playhead_secs(&self) -> f64 {
        self.playhead_secs
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub fn media_pool(&self) -> &[MediaAsset] {
        self.project
            .as_ref()
            .map(|p| p.media_pool.as_slice())
            .unwrap_or(&[])
    }

    pub fn project_duration(&self) -> f64 {
        self.project
            .as_ref()
            .map(|p| p.timeline.duration_secs)
            .unwrap_or(0.0)
    }

    pub fn preview_resolution(&self) -> (u32, u32) {
        self.project
            .as_ref()
            .map(|p| (p.settings.width, p.settings.height))
            .unwrap_or((1920, 1080))
    }

    // ── File actions ──────────────────────────────────────────────────────────

    pub fn action_new_project(&mut self) {
        self.project = Some(Project::new("Untitled Project"));
        self.history = CommandHistory::new();
        self.playhead_secs = 0.0;
        self.is_playing = false;
        self.selected_asset_id = None;
        self.selected_clip_id = None;
        tracing::info!("New project created");
    }

    pub fn action_open_project(&mut self) {
        // Use rfd to show a native open dialog (blocking on this thread is fine for now).
        let path = rfd::FileDialog::new()
            .add_filter("VidCut Project", &["vidcut"])
            .set_title("Open Project")
            .pick_file();

        if let Some(path) = path {
            match Project::load(&path) {
                Ok(proj) => {
                    self.project = Some(proj);
                    self.history = CommandHistory::new();
                    self.playhead_secs = 0.0;
                    self.is_playing = false;
                    tracing::info!("Opened project: {:?}", path);
                }
                Err(e) => {
                    tracing::error!("Failed to open project: {e}");
                }
            }
        }
    }

    pub fn action_save_project(&mut self) {
        if self.project.is_none() {
            return;
        }
        let path = rfd::FileDialog::new()
            .add_filter("VidCut Project", &["vidcut"])
            .set_file_name("project.vidcut")
            .set_title("Save Project")
            .save_file();

        if let Some(path) = path {
            let proj = self.project.as_ref().unwrap();
            if let Err(e) = proj.save(&path) {
                tracing::error!("Failed to save project: {e}");
            } else {
                tracing::info!("Project saved to {:?}", path);
            }
        }
    }

    pub fn action_export(&mut self) {
        tracing::info!("Export — Phase 3");
    }

    // ── Media import ──────────────────────────────────────────────────────────

    /// Open a file picker and import selected media files into the project.
    pub fn action_import_media(&mut self) {
        let paths = rfd::FileDialog::new()
            .add_filter(
                "Video / Audio / Image",
                &[
                    "mp4", "m4v", "mov", "avi", "mkv", "wmv", "webm",
                    "mp3", "wav", "flac", "ogg", "aac", "m4a",
                    "png", "jpg", "jpeg", "bmp", "webp",
                ],
            )
            .set_title("Import Media")
            .pick_files();

        let paths: Vec<PathBuf> = paths.unwrap_or_default();
        if paths.is_empty() {
            return;
        }

        self.ensure_project();

        for path in paths {
            let info = probe_file(&path);
            tracing::info!(
                "Probed {:?}: {:?}, {:.2}s, {}×{}",
                path.file_name(),
                info.kind,
                info.duration_secs,
                info.width,
                info.height,
            );

            let asset_type = match info.kind {
                AssetKind::Video => AssetType::Video,
                AssetKind::Audio => AssetType::Audio,
                AssetKind::Image => AssetType::Image,
                AssetKind::Unknown => AssetType::Video,
            };

            let asset = MediaAsset::new(
                path,
                asset_type,
                info.duration_secs,
                if info.width > 0 { Some(info.width) } else { None },
                if info.height > 0 { Some(info.height) } else { None },
                if info.fps > 0.0 { Some(info.fps) } else { None },
            );

            let project = self.project.as_mut().unwrap();
            project.media_pool.push(asset);
        }
    }

    // ── Timeline actions ──────────────────────────────────────────────────────

    /// Add a media asset to the timeline (appended after existing clips on first matching track).
    pub fn action_add_asset_to_timeline(&mut self, asset_id: Uuid) {
        let project = match self.project.as_mut() {
            Some(p) => p,
            None => return,
        };

        // Find the asset.
        let asset = match project.media_pool.iter().find(|a| a.id == asset_id) {
            Some(a) => a.clone(),
            None => return,
        };

        // Determine target track type.
        let target_type = match asset.asset_type {
            AssetType::Video | AssetType::Image => TrackType::Video,
            AssetType::Audio => TrackType::Audio,
        };

        // Find or create the first matching track.
        let track_id = if let Some(t) = project.timeline.tracks.iter().find(|t| t.track_type == target_type) {
            t.id
        } else {
            let track_name = match target_type {
                TrackType::Video => format!("Video {}", project.timeline.tracks.len() + 1),
                TrackType::Audio => format!("Audio {}", project.timeline.tracks.len() + 1),
            };
            let new_track = Track::new(track_name, target_type);
            let id = new_track.id;
            project.timeline.tracks.push(new_track);
            id
        };

        // Compute start: append after last clip on this track.
        let timeline_start = {
            let track = project.timeline.tracks.iter().find(|t| t.id == track_id).unwrap();
            track.clips.iter().map(|c| c.timeline_end).fold(0.0_f64, f64::max)
        };

        let duration = asset.duration_secs.max(1.0); // at least 1 second
        let clip = Clip {
            id: Uuid::new_v4(),
            asset_id,
            track_id,
            timeline_start,
            timeline_end: timeline_start + duration,
            source_start: 0.0,
            source_end: duration,
        };

        let clip_id = clip.id;
        self.history.push(Box::new(AddClipCommand::new(clip)), &mut project.timeline);
        self.selected_clip_id = Some(clip_id);
        tracing::info!("Added clip to timeline at {:.2}s", timeline_start);
    }

    /// Add a new empty track.
    pub fn action_add_track(&mut self) {
        let project = self.ensure_project();
        let n = project.timeline.tracks.iter().filter(|t| t.track_type == TrackType::Video).count() + 1;
        project.timeline.tracks.push(Track::new(format!("Video {n}"), TrackType::Video));
        tracing::info!("Track added");
    }

    /// Move the selected clip by `delta_secs` (commit on drag end).
    pub fn action_commit_move_clip(&mut self, clip_id: Uuid, track_id: Uuid, old_start: f64, new_start: f64, duration: f64) {
        let project = match self.project.as_mut() {
            Some(p) => p,
            None => return,
        };
        let cmd = MoveClipCommand::new(clip_id, track_id, old_start, new_start, duration);
        self.history.push(Box::new(cmd), &mut project.timeline);
    }

    /// Delete the currently selected clip.
    pub fn action_delete_selected_clip(&mut self) {
        let selected = match self.selected_clip_id {
            Some(id) => id,
            None => return,
        };

        let project = match self.project.as_mut() {
            Some(p) => p,
            None => return,
        };

        // Find and clone the clip.
        let clip = project
            .timeline
            .tracks
            .iter()
            .flat_map(|t| t.clips.iter())
            .find(|c| c.id == selected)
            .cloned();

        if let Some(clip) = clip {
            let cmd = DeleteClipCommand::new(clip);
            self.history.push(Box::new(cmd), &mut project.timeline);
            self.selected_clip_id = None;
        }
    }

    // ── Transport ─────────────────────────────────────────────────────────────

    pub fn action_play_pause(&mut self) {
        self.is_playing = !self.is_playing;
        tracing::debug!("Play: {}", self.is_playing);
    }

    pub fn action_stop(&mut self) {
        self.is_playing = false;
        self.playhead_secs = 0.0;
    }

    pub fn action_undo(&mut self) {
        if let Some(project) = &mut self.project {
            self.history.undo(&mut project.timeline);
        }
    }

    pub fn action_redo(&mut self) {
        if let Some(project) = &mut self.project {
            self.history.redo(&mut project.timeline);
        }
    }
}

// ─── eframe::App ─────────────────────────────────────────────────────────────

impl eframe::App for VidCutApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Advance playhead if playing.
        if self.is_playing {
            let dt = ctx.input(|i| i.unstable_dt) as f64;
            self.playhead_secs += dt;
            let dur = self.project_duration();
            if dur > 0.0 && self.playhead_secs > dur {
                self.is_playing = false;
                self.playhead_secs = 0.0;
            }
            ctx.request_repaint();
        }

        // Keyboard shortcuts.
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                self.action_play_pause();
            }
            if i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace) {
                self.action_delete_selected_clip();
            }
            let ctrl = i.modifiers.ctrl || i.modifiers.mac_cmd;
            if ctrl && i.key_pressed(egui::Key::Z) {
                self.action_undo();
            }
            if ctrl && (i.key_pressed(egui::Key::Y) || (i.modifiers.shift && i.key_pressed(egui::Key::Z))) {
                self.action_redo();
            }
        });

        // Panels — order matters for egui layout resolution.
        panels::toolbar::show(ctx, self);
        panels::timeline::show(ctx, self);
        panels::media_browser::show(ctx, self);
        panels::inspector::show(ctx, self);
        panels::preview::show(ctx, self);
    }
}
