//! `VidCutApp` — top-level application state and `eframe::App` implementation.
//!
//! This struct owns the [`Project`], [`CommandHistory`], playback state,
//! and UI selection state. Each panel accesses and mutates app state through
//! the public accessor/action methods defined here.

use std::{collections::HashMap, path::PathBuf, sync::mpsc, time::Instant};

use eframe::egui;
use uuid::Uuid;
use vidcut_core::{
    commands::{
        add_clip::AddClipCommand, delete_clip::DeleteClipCommand, move_clip::MoveClipCommand,
        trim_clip::TrimClipCommand, CommandHistory,
    },
    AssetType, Clip, MediaAsset, Project, Track, TrackType,
};
use vidcut_media::{
    ensure_ffmpeg, extract_frame, probe_file, AssetKind, DecodedFrame, ExportEncoder,
    ExportJob, ExportProgress, ExportSegment, FfmpegStatus, OutputFormat, QualityPreset,
};

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
    /// Playback speed multiplier (1.0 = normal; negative = reverse).
    pub playback_speed: f32,

    // ── Selection state ───────────────────────────────────────────────────────
    /// The asset currently highlighted in the Media Browser.
    pub selected_asset_id: Option<Uuid>,
    /// The clip currently selected on the timeline.
    pub selected_clip_id: Option<Uuid>,

    // ── Timeline view state ───────────────────────────────────────────────────
    /// Pixels per second for the timeline zoom level.
    pub timeline_px_per_sec: f32,

    // ── Drag state ────────────────────────────────────────────────────────────
    /// While dragging a clip body, stores clip move state.
    pub dragging: Option<DragState>,
    /// While dragging a clip trim handle.
    pub trim_dragging: Option<TrimDragState>,

    // ── Thumbnail cache ───────────────────────────────────────────────────────
    /// Cached egui textures keyed by asset UUID.
    pub thumbnail_cache: HashMap<Uuid, egui::TextureHandle>,

    // ── Export state ──────────────────────────────────────────────────────
    /// Whether the export dialog is currently visible.
    pub export_dialog_open: bool,
    /// Chosen output path for the next / current export.
    pub export_output_path: Option<PathBuf>,
    /// Selected container format.
    pub export_format: OutputFormat,
    /// Selected quality preset.
    pub export_quality: QualityPreset,
    /// Running export encoder (held while export is in progress).
    export_encoder: Option<ExportEncoder>,
    /// mpsc receiver for progress updates from the encoder thread.
    export_rx: Option<mpsc::Receiver<ExportProgress>>,
    /// Progress fraction `[0, 1]` + last status line, while export is running.
    pub export_progress: Option<(f32, String)>,
    /// Last completed status message (success / error / cancelled).
    pub export_status: Option<String>,

    // ── Preview playback state ────────────────────────────────────────────────
    /// The currently displayed preview frame texture.
    pub preview_texture: Option<egui::TextureHandle>,
    /// Frame index that the current texture represents (avoids re-decoding).
    preview_frame_index: Option<u64>,
    /// Receiver for decoded frames from the background extraction thread.
    preview_rx: Option<mpsc::Receiver<PreviewFrame>>,
    /// The frame index of the most recently dispatched extraction request.
    /// Used to discard stale results when the user scrubs quickly.
    last_preview_request: Option<u64>,
    /// Throttle: earliest `Instant` at which we may dispatch a new request.
    preview_next_allowed: Instant,

    // ── FFmpeg setup state ────────────────────────────────────────────────────
    /// Current state of the FFmpeg sidecar lifecycle.
    pub ffmpeg_status: FfmpegStatus,
    /// Receiver for status updates from the background setup thread.
    ffmpeg_setup_rx: Option<mpsc::Receiver<FfmpegStatus>>,
}

/// A decoded frame delivered from the background extraction thread.
pub struct PreviewFrame {
    /// The frame index this result corresponds to.
    pub frame_index: u64,
    /// The decoded RGBA frame (None if extraction failed).
    pub frame: Option<DecodedFrame>,
}

/// Temporary state while a clip body is being moved on the timeline.
#[derive(Debug, Clone)]
pub struct DragState {
    pub clip_id: Uuid,
    pub track_id: Uuid,
    pub original_start: f64,
    pub duration: f64,
    /// Current uncommitted start position (updated every frame during drag).
    pub current_start: f64,
}

/// Which edge of a clip is being trimmed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrimEdge {
    Left,
    Right,
}

/// Temporary state while trimming a clip edge.
#[derive(Debug, Clone)]
pub struct TrimDragState {
    pub clip_id: Uuid,
    pub track_id: Uuid,
    pub edge: TrimEdge,
    // original values (for undo)
    pub orig_timeline_start: f64,
    pub orig_timeline_end: f64,
    pub orig_source_start: f64,
    pub orig_source_end: f64,
    // live values updated each frame
    pub cur_timeline_start: f64,
    pub cur_timeline_end: f64,
    pub cur_source_start: f64,
    pub cur_source_end: f64,
}

impl VidCutApp {
    /// Initialise the application. Called once by eframe during startup.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        panels::theme::apply_dark_theme(&cc.egui_ctx);

        // Spawn background thread to ensure FFmpeg is available.
        let (ffmpeg_tx, ffmpeg_rx) = mpsc::channel::<FfmpegStatus>();
        std::thread::spawn(move || ensure_ffmpeg(ffmpeg_tx));

        Self {
            project: None,
            history: CommandHistory::new(),
            playhead_secs: 0.0,
            is_playing: false,
            playback_speed: 1.0,
            selected_asset_id: None,
            selected_clip_id: None,
            timeline_px_per_sec: 80.0,
            dragging: None,
            trim_dragging: None,
            thumbnail_cache: HashMap::new(),
            export_dialog_open: false,
            export_output_path: None,
            export_format: OutputFormat::Mp4,
            export_quality: QualityPreset::Medium,
            export_encoder: None,
            export_rx: None,
            export_progress: None,
            export_status: None,
            preview_texture: None,
            preview_frame_index: None,
            preview_rx: None,
            last_preview_request: None,
            preview_next_allowed: Instant::now(),
            ffmpeg_status: FfmpegStatus::Checking,
            ffmpeg_setup_rx: Some(ffmpeg_rx),
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

    /// Frame duration in seconds based on project FPS.
    pub fn frame_duration(&self) -> f64 {
        let fps = self.project.as_ref().map(|p| p.settings.fps).unwrap_or(30u32);
        if fps > 0 { 1.0 / fps as f64 } else { 1.0 / 30.0 }
    }

    // ── File actions ──────────────────────────────────────────────────────────

    pub fn action_new_project(&mut self) {
        self.project = Some(Project::new("Untitled Project"));
        self.history = CommandHistory::new();
        self.playhead_secs = 0.0;
        self.is_playing = false;
        self.playback_speed = 1.0;
        self.selected_asset_id = None;
        self.selected_clip_id = None;
        self.thumbnail_cache.clear();
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
                    self.thumbnail_cache.clear();
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

    /// Open the export dialog.
    pub fn action_export(&mut self) {
        // Reset status message so a fresh dialog doesn't show stale result.
        if self.export_progress.is_none() {
            self.export_status = None;
        }
        self.export_dialog_open = true;
    }

    /// Build the [`ExportJob`] from the current project and start encoding.
    pub fn action_start_export(&mut self) {
        let output_path = match &self.export_output_path {
            Some(p) => p.clone(),
            None => {
                self.export_status = Some("✗ No output path selected.".to_owned());
                return;
            }
        };

        let project = match &self.project {
            Some(p) => p.clone(),
            None => {
                self.export_status = Some("✗ No project open.".to_owned());
                return;
            }
        };

        // Build a flat ordered list of segments from all tracks.
        let mut segments: Vec<ExportSegment> = Vec::new();

        // Collect all clips sorted by timeline_start.
        let mut all_clips: Vec<_> = project
            .timeline
            .tracks
            .iter()
            .flat_map(|t| t.clips.iter().map(move |c| (t.track_type.clone(), c.clone())))
            .collect();
        all_clips.sort_by(|a, b| a.1.timeline_start.partial_cmp(&b.1.timeline_start).unwrap());

        for (_track_type, clip) in &all_clips {
            if let Some(asset) = project.media_pool.iter().find(|a| a.id == clip.asset_id) {
                segments.push(ExportSegment {
                    source_path: asset.path.clone(),
                    source_start: clip.source_start,
                    duration: clip.duration().max(0.01),
                    has_video: matches!(asset.asset_type, AssetType::Video),
                    has_audio: matches!(asset.asset_type, AssetType::Video | AssetType::Audio),
                });
            }
        }

        if segments.is_empty() {
            self.export_status = Some("✗ Timeline is empty — add some clips first.".to_owned());
            return;
        }

        let job = ExportJob {
            segments,
            output_path,
            format: self.export_format,
            quality: self.export_quality,
            fps: project.settings.fps,
            width: project.settings.width,
            height: project.settings.height,
        };

        match ExportEncoder::begin(job) {
            Ok((encoder, rx)) => {
                self.export_encoder = Some(encoder);
                self.export_rx = Some(rx);
                self.export_progress = Some((0.0, "Starting…".to_owned()));
                self.export_status = None;
                tracing::info!("Export started.");
            }
            Err(e) => {
                self.export_status = Some(format!("✗ {e}"));
                tracing::error!("Export failed to start: {e}");
            }
        }
    }

    /// Kill the running export.
    pub fn action_cancel_export(&mut self) {
        if let Some(encoder) = self.export_encoder.take() {
            encoder.cancel();
        }
        self.export_rx = None;
        self.export_progress = None;
        self.export_status = Some("Export cancelled.".to_owned());
        tracing::info!("Export cancelled by user.");
    }

    /// Poll the mpsc channel for progress updates from the encoder thread.
    /// Must be called every frame while export is in progress.
    pub fn poll_export_progress(&mut self, ctx: &egui::Context) {
        if self.export_rx.is_none() {
            return;
        }

        // Drain all pending messages (non-blocking).
        loop {
            let msg = match &self.export_rx {
                Some(rx) => rx.try_recv(),
                None => break,
            };

            match msg {
                Ok(ExportProgress::Progress { fraction, message }) => {
                    self.export_progress = Some((fraction, message));
                    // Keep UI repainting while export is running.
                    ctx.request_repaint();
                }
                Ok(ExportProgress::Done { output_path }) => {
                    self.export_progress = None;
                    self.export_encoder = None;
                    self.export_rx = None;
                    self.export_status =
                        Some(format!("✓ Export complete: {}", output_path.display()));
                    tracing::info!("Export done: {:?}", output_path);

                    // Update Windows taskbar progress to indeterminate-done.
                    #[cfg(target_os = "windows")]
                    taskbar_set_complete();
                    break;
                }
                Ok(ExportProgress::Failed { message }) => {
                    self.export_progress = None;
                    self.export_encoder = None;
                    self.export_rx = None;
                    self.export_status = Some(format!("✗ Export failed: {message}"));
                    tracing::error!("Export failed: {message}");

                    #[cfg(target_os = "windows")]
                    taskbar_clear();
                    break;
                }
                Ok(ExportProgress::Cancelled) => {
                    self.export_progress = None;
                    self.export_encoder = None;
                    self.export_rx = None;
                    if self.export_status.is_none() {
                        self.export_status = Some("Export cancelled.".to_owned());
                    }

                    #[cfg(target_os = "windows")]
                    taskbar_clear();
                    break;
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    // Thread exited without sending Done/Failed — treat as cancelled.
                    self.export_progress = None;
                    self.export_encoder = None;
                    self.export_rx = None;
                    break;
                }
            }
        }

        // Update taskbar progress bar on Windows.
        #[cfg(target_os = "windows")]
        if let Some((fraction, _)) = &self.export_progress {
            taskbar_set_progress(*fraction);
        }
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
            transform: Default::default(),
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

    /// Commit a trim operation (called when trim drag ends).
    #[allow(clippy::too_many_arguments)]
    pub fn action_commit_trim_clip(
        &mut self,
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
    ) {
        let project = match self.project.as_mut() {
            Some(p) => p,
            None => return,
        };
        let cmd = TrimClipCommand::new(
            clip_id, track_id,
            old_timeline_start, old_timeline_end, old_source_start, old_source_end,
            new_timeline_start, new_timeline_end, new_source_start, new_source_end,
        );
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
        if self.is_playing && self.playback_speed == 0.0 {
            // was paused via K key — resume
            self.playback_speed = 1.0;
            self.is_playing = true;
        } else {
            self.is_playing = !self.is_playing;
            if self.is_playing {
                self.playback_speed = 1.0;
            }
        }
        tracing::debug!("Play: {} speed: {}", self.is_playing, self.playback_speed);
    }

    pub fn action_stop(&mut self) {
        self.is_playing = false;
        self.playback_speed = 1.0;
        self.playhead_secs = 0.0;
    }

    /// Step forward by one frame.
    pub fn action_step_forward(&mut self) {
        self.is_playing = false;
        let frame_dur = self.frame_duration();
        let dur = self.project_duration();
        self.playhead_secs = (self.playhead_secs + frame_dur).min(dur.max(frame_dur));
    }

    /// Step backward by one frame.
    pub fn action_step_backward(&mut self) {
        self.is_playing = false;
        let frame_dur = self.frame_duration();
        self.playhead_secs = (self.playhead_secs - frame_dur).max(0.0);
    }

    /// J key — reverse / slow down.
    pub fn action_j(&mut self) {
        if !self.is_playing {
            self.is_playing = true;
            self.playback_speed = -1.0;
        } else {
            self.playback_speed = (self.playback_speed - 1.0).clamp(-4.0, -0.25);
        }
    }

    /// K key — pause.
    pub fn action_k(&mut self) {
        self.is_playing = false;
        self.playback_speed = 1.0;
    }

    /// L key — play / speed up.
    pub fn action_l(&mut self) {
        if !self.is_playing {
            self.is_playing = true;
            self.playback_speed = 1.0;
        } else {
            let candidates = [0.25_f32, 0.5, 1.0, 2.0, 4.0];
            let next = candidates.iter().copied()
                .find(|&s| s > self.playback_speed.max(0.0))
                .unwrap_or(4.0);
            self.playback_speed = next;
        }
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

    // ── FFmpeg setup polling ─────────────────────────────────────────────────

    /// Poll the background FFmpeg setup thread for status updates.
    /// Must be called every frame while setup is running.
    pub fn poll_ffmpeg_status(&mut self, ctx: &egui::Context) {
        loop {
            let msg = match &self.ffmpeg_setup_rx {
                Some(rx) => rx.try_recv(),
                None => break,
            };
            match msg {
                Ok(status) => {
                    let done = matches!(status, FfmpegStatus::Ready | FfmpegStatus::Failed(_));
                    self.ffmpeg_status = status;
                    if done {
                        // Stop polling once terminal state reached.
                        self.ffmpeg_setup_rx = None;
                        break;
                    }
                    // Keep repainting while downloading.
                    ctx.request_repaint();
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    // Thread exited without sending Ready — mark as failed.
                    if !matches!(self.ffmpeg_status, FfmpegStatus::Ready | FfmpegStatus::Failed(_)) {
                        self.ffmpeg_status = FfmpegStatus::Failed(
                            "FFmpeg setup thread exited unexpectedly.".to_owned(),
                        );
                    }
                    self.ffmpeg_setup_rx = None;
                    break;
                }
            }
        }
    }

    /// Re-trigger the FFmpeg setup process (e.g. after a failed download).
    pub fn action_retry_ffmpeg_setup(&mut self) {
        let (tx, rx) = mpsc::channel::<FfmpegStatus>();
        self.ffmpeg_status = FfmpegStatus::Checking;
        self.ffmpeg_setup_rx = Some(rx);
        std::thread::spawn(move || vidcut_media::ensure_ffmpeg(tx));
    }

    // ── Preview frame pipeline ────────────────────────────────────────────────

    /// Determine which clip (if any) is under the playhead and request a
    /// background frame extraction if we don't already have that frame.
    pub fn request_preview_frame(&mut self, ctx: &egui::Context) {
        // Don't extract if FFmpeg isn't ready.
        if !matches!(self.ffmpeg_status, FfmpegStatus::Ready) {
            return;
        }

        // Don't dispatch if a request is already in flight — wait for it.
        if self.preview_rx.is_some() {
            return;
        }

        let project = match &self.project {
            Some(p) => p,
            None => return,
        };

        let fps = project.settings.fps.max(1) as f64;
        let frame_index = (self.playhead_secs * fps) as u64;

        // Already have this frame — nothing to do.
        if self.preview_frame_index == Some(frame_index) {
            return;
        }

        // Throttle: at most one request every ~50ms to avoid flooding.
        if Instant::now() < self.preview_next_allowed {
            // Schedule a repaint so we retry soon.
            ctx.request_repaint();
            return;
        }

        // Find the topmost video clip at the current playhead position.
        let playhead = self.playhead_secs;
        let mut found_clip = None;

        // Iterate tracks in reverse so higher tracks have visual priority.
        for track in project.timeline.tracks.iter().rev() {
            if track.track_type != vidcut_core::TrackType::Video {
                continue;
            }
            for clip in &track.clips {
                if playhead >= clip.timeline_start && playhead < clip.timeline_end {
                    found_clip = Some(clip.clone());
                    break;
                }
            }
            if found_clip.is_some() {
                break;
            }
        }

        let clip = match found_clip {
            Some(c) => c,
            None => {
                // No clip under playhead — clear preview.
                self.preview_texture = None;
                self.preview_frame_index = None;
                return;
            }
        };

        // Resolve source file path.
        let asset = match project.media_pool.iter().find(|a| a.id == clip.asset_id) {
            Some(a) => a,
            None => return,
        };

        let source_timestamp =
            clip.source_start + (playhead - clip.timeline_start);
        let source_path = asset.path.clone();

        // Create channel for this extraction.
        let (tx, rx) = mpsc::channel::<PreviewFrame>();
        self.preview_rx = Some(rx);
        self.last_preview_request = Some(frame_index);
        self.preview_next_allowed = Instant::now() + std::time::Duration::from_millis(50);

        tracing::debug!(
            "Preview: requesting frame {} at {:.3}s (source {:.3}s) from {:?}",
            frame_index, playhead, source_timestamp, source_path.file_name()
        );

        // Spawn background thread.
        let repaint_ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = extract_frame(&source_path, source_timestamp, 640);
            match &result {
                Ok(f) => tracing::debug!(
                    "Preview: frame {} decoded OK ({}×{})",
                    frame_index, f.width, f.height
                ),
                Err(e) => tracing::warn!(
                    "Preview: frame {} extraction failed: {:#}",
                    frame_index, e
                ),
            }
            let _ = tx.send(PreviewFrame {
                frame_index,
                frame: result.ok(),
            });
            // Wake the UI so it polls the result.
            repaint_ctx.request_repaint();
        });
    }

    /// Poll for completed frame extractions and upload the texture.
    pub fn poll_preview_frame(&mut self, ctx: &egui::Context) {
        let rx = match &self.preview_rx {
            Some(rx) => rx,
            None => return,
        };

        match rx.try_recv() {
            Ok(pf) => {
                if let Some(frame) = pf.frame {
                    tracing::debug!(
                        "Preview: uploading texture {}×{} for frame {}",
                        frame.width, frame.height, pf.frame_index
                    );
                    let image = egui::ColorImage::from_rgba_unmultiplied(
                        [frame.width as usize, frame.height as usize],
                        &frame.rgba,
                    );
                    let texture = ctx.load_texture(
                        "preview_frame",
                        image,
                        egui::TextureOptions::LINEAR,
                    );
                    self.preview_texture = Some(texture);
                    self.preview_frame_index = Some(pf.frame_index);
                } else {
                    tracing::warn!("Preview: frame {} had no data (extraction failed)", pf.frame_index);
                }

                self.preview_rx = None;
                ctx.request_repaint();
            }
            Err(mpsc::TryRecvError::Empty) => { /* still waiting */ }
            Err(mpsc::TryRecvError::Disconnected) => {
                tracing::trace!("Preview: extraction thread disconnected");
                self.preview_rx = None;
            }
        }
    }
}

// ─── eframe::App ─────────────────────────────────────────────────────────────

impl eframe::App for VidCutApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Advance playhead if playing.
        if self.is_playing {
            let dt = ctx.input(|i| i.unstable_dt) as f64;
            self.playhead_secs += dt * self.playback_speed as f64;
            let dur = self.project_duration();

            // Clamp and stop at boundaries.
            if self.playhead_secs < 0.0 {
                self.playhead_secs = 0.0;
                self.is_playing = false;
                self.playback_speed = 1.0;
            } else if dur > 0.0 && self.playhead_secs > dur {
                self.is_playing = false;
                self.playback_speed = 1.0;
                self.playhead_secs = 0.0;
            }
            ctx.request_repaint();
        }

        // Poll export progress.
        self.poll_export_progress(ctx);

        // Poll FFmpeg setup thread.
        self.poll_ffmpeg_status(ctx);

        // Preview frame pipeline.
        self.request_preview_frame(ctx);
        self.poll_preview_frame(ctx);

        // Keyboard shortcuts.
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                self.action_play_pause();
            }
            if i.key_pressed(egui::Key::J) {
                self.action_j();
            }
            if i.key_pressed(egui::Key::K) {
                self.action_k();
            }
            if i.key_pressed(egui::Key::L) {
                self.action_l();
            }
            if i.key_pressed(egui::Key::ArrowRight) && !i.modifiers.any() {
                self.action_step_forward();
            }
            if i.key_pressed(egui::Key::ArrowLeft) && !i.modifiers.any() {
                self.action_step_backward();
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
        // Export dialog (modal, rendered on top of all other panels).
        panels::export_dialog::show(ctx, self);
        // FFmpeg setup overlay (topmost — blocks interaction until ready).
        panels::ffmpeg_setup::show(ctx, self);
    }
}

// ─── Windows Taskbar Progress ─────────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn with_taskbar<F: FnOnce(&windows::Win32::UI::Shell::ITaskbarList3)>(f: F) {
    use windows::{
        Win32::{
            System::Com::{
                CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
            },
            UI::Shell::{ITaskbarList3, TaskbarList},
        },
    };
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if let Ok(tbl) =
            CoCreateInstance::<_, ITaskbarList3>(&TaskbarList, None, CLSCTX_INPROC_SERVER)
        {
            f(&tbl);
        }
    }
}

/// Set Windows taskbar progress bar to `fraction` (0.0–1.0).
#[cfg(target_os = "windows")]
fn taskbar_set_progress(fraction: f32) {
    use windows::Win32::{Foundation::HWND, UI::Shell::TBPFLAG};
    with_taskbar(|tbl| unsafe {
        let _ = tbl.SetProgressState(HWND(0 as _), TBPFLAG(0x2)); // TBPF_NORMAL
        let completed = (fraction * 10_000.0) as u64;
        let _ = tbl.SetProgressValue(HWND(0 as _), completed, 10_000);
    });
}

/// Set Windows taskbar progress to complete (full green bar).
#[cfg(target_os = "windows")]
fn taskbar_set_complete() {
    use windows::Win32::{Foundation::HWND, UI::Shell::TBPFLAG};
    with_taskbar(|tbl| unsafe {
        let _ = tbl.SetProgressValue(HWND(0 as _), 10_000, 10_000);
        let _ = tbl.SetProgressState(HWND(0 as _), TBPFLAG(0x2)); // TBPF_NORMAL at 100%
    });
}

/// Clear the Windows taskbar progress bar.
#[cfg(target_os = "windows")]
fn taskbar_clear() {
    use windows::Win32::{Foundation::HWND, UI::Shell::TBPFLAG};
    with_taskbar(|tbl| unsafe {
        let _ = tbl.SetProgressState(HWND(0 as _), TBPFLAG(0x0)); // TBPF_NOPROGRESS
    });
}
