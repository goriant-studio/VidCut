//! Timeline panel — bottom panel with track rows, clip blocks, and playhead.
//!
//! Phase 2:
//! - Clip blocks: click (select), drag body (move), drag left/right edge (trim).
//! - Snap-to-grid: moves/trims snap to nearest frame boundary.
//! - Overlap detection: prevents clips from overlapping on the same track.
//! - Waveform display inside audio clips (deterministic pattern).
//! - Thumbnail strip inside video clips (deterministic color pattern).
//! - Ruler click sets playhead; scroll-wheel zooms.
//! - Delete key removes selected clip (handled in app.rs keyboard shortcuts).

use eframe::egui::{self, Color32, PointerButton, RichText, Sense, Stroke};
use vidcut_core::TrackType;

use crate::{
    app::{TrimDragState, TrimEdge, VidCutApp},
    panels::theme,
};

const TRACK_HEADER_WIDTH: f32 = 130.0;
const TRACK_HEIGHT: f32 = 48.0;
const RULER_HEIGHT: f32 = 22.0;
const TRIM_HANDLE_WIDTH: f32 = 7.0;
const MIN_PX_PER_SEC: f32 = 5.0;
const MAX_PX_PER_SEC: f32 = 800.0;
const MIN_CLIP_DURATION: f64 = 0.05; // 50 ms minimum clip length

/// Show the timeline panel. Called every frame from [`VidCutApp::update`].
pub fn show(ctx: &egui::Context, app: &mut VidCutApp) {
    egui::TopBottomPanel::bottom("timeline")
        .min_height(200.0)
        .default_height(240.0)
        .resizable(true)
        .frame(
            egui::Frame::none()
                .fill(theme::BG_DEEP)
                .inner_margin(egui::Margin::same(0.0)),
        )
        .show(ctx, |ui| {
            // ── Timeline header ─────────────────────────────────────────────────
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(0x18, 0x18, 0x24))
                .inner_margin(egui::Margin::symmetric(12.0, 6.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("TIMELINE")
                                .color(theme::TEXT_MUTED)
                                .size(11.0)
                                .strong(),
                        );
                        ui.add_space(16.0);

                        let pps = app.timeline_px_per_sec;
                        let fps = app.project.as_ref().map(|p| p.settings.fps).unwrap_or(30u32);
                        ui.label(
                            RichText::new(format!("{pps:.0}px/s  ·  {fps}fps"))
                                .color(theme::TEXT_MUTED)
                                .size(11.0),
                        );

                        if let Some(id) = app.selected_clip_id {
                            ui.add_space(12.0);
                            ui.label(
                                RichText::new(format!("Clip {:.8} selected  [Del] to remove", id.to_string()))
                                    .color(Color32::from_rgb(0x6c, 0x7b, 0xff))
                                    .size(11.0),
                            );
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("+ Track").clicked() {
                                app.action_add_track();
                            }
                        });
                    });
                });

            ui.separator();

            // ── Scroll zoom ─────────────────────────────────────────────────────
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll_delta.abs() > 0.5 {
                let factor = 1.0 + scroll_delta * 0.003;
                app.timeline_px_per_sec = (app.timeline_px_per_sec * factor)
                    .clamp(MIN_PX_PER_SEC, MAX_PX_PER_SEC);
            }

            let px_per_sec = app.timeline_px_per_sec;
            let total_secs = app.project_duration().max(30.0);
            let fps = app.project.as_ref().map(|p| p.settings.fps).unwrap_or(30u32) as f64;
            let frame_dur = if fps > 0.0 { 1.0 / fps } else { 1.0 / 30.0 };
            let available = ui.available_size();

            egui::ScrollArea::both()
                .id_salt("timeline_scroll")
                .show(ui, |ui| {
                    let content_width = (total_secs as f32 * px_per_sec + TRACK_HEADER_WIDTH + 60.0)
                        .max(available.x);
                    ui.set_min_width(content_width);

                    let track_area_width = content_width - TRACK_HEADER_WIDTH;

                    // ── Ruler ──────────────────────────────────────────────────
                    let ruler_resp = draw_ruler(ui, track_area_width, total_secs, px_per_sec);
                    if let Some(click_x) = ruler_resp {
                        let raw_secs = (click_x / px_per_sec) as f64;
                        // Snap to frame
                        let secs = snap_to_frame(raw_secs, frame_dur);
                        app.playhead_secs = secs.max(0.0);
                    }

                    // ── Snapshot project state (avoids split borrow) ────────────
                    let tracks_data: Vec<_> = app
                        .project
                        .as_ref()
                        .map(|p| {
                            p.timeline.tracks.iter().map(|t| {
                                (
                                    t.id,
                                    t.name.clone(),
                                    t.track_type.clone(),
                                    t.clips.clone(),
                                )
                            }).collect()
                        })
                        .unwrap_or_default();

                    let selected_clip_id = app.selected_clip_id;
                    let playhead_secs = app.playhead_secs;

                    if tracks_data.is_empty() {
                        ui.add_space(16.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("No tracks yet. Import media and double-click to add to timeline.")
                                    .color(theme::TEXT_MUTED)
                                    .size(12.0),
                            );
                        });
                    } else {
                        for (track_id, track_name, track_type, clips) in &tracks_data {
                            let is_video = *track_type == TrackType::Video;
                            let track_color = if is_video {
                                Color32::from_rgb(0x4a, 0x5a, 0xff)
                            } else {
                                Color32::from_rgb(0x2a, 0xaa, 0x6a)
                            };

                            let (row_rect, _) = ui.allocate_exact_size(
                                egui::vec2(TRACK_HEADER_WIDTH + track_area_width, TRACK_HEIGHT),
                                Sense::hover(),
                            );

                            if !ui.is_rect_visible(row_rect) {
                                continue;
                            }

                            let painter = ui.painter().clone();

                            // Track header
                            let header_rect = egui::Rect::from_min_size(
                                row_rect.min,
                                egui::vec2(TRACK_HEADER_WIDTH, TRACK_HEIGHT),
                            );
                            painter.rect_filled(header_rect, 0.0, Color32::from_rgb(0x1e, 0x1e, 0x2e));
                            painter.rect_stroke(header_rect, 0.0, Stroke::new(1.0, theme::BORDER));

                            // Accent swatch
                            painter.rect_filled(
                                egui::Rect::from_min_size(header_rect.min, egui::vec2(3.0, TRACK_HEIGHT)),
                                0.0,
                                track_color,
                            );

                            // Track type icon
                            let type_label = if is_video { "🎬" } else { "🎵" };
                            painter.text(
                                header_rect.left_center() + egui::vec2(8.0, 0.0),
                                egui::Align2::LEFT_CENTER,
                                type_label,
                                egui::FontId::proportional(12.0),
                                track_color,
                            );
                            painter.text(
                                header_rect.left_center() + egui::vec2(26.0, 0.0),
                                egui::Align2::LEFT_CENTER,
                                track_name.as_str(),
                                egui::FontId::proportional(10.0),
                                theme::TEXT_PRIMARY,
                            );

                            // Content area background
                            let content_rect = egui::Rect::from_min_size(
                                row_rect.min + egui::vec2(TRACK_HEADER_WIDTH, 0.0),
                                egui::vec2(track_area_width, TRACK_HEIGHT),
                            );
                            painter.rect_filled(content_rect, 0.0, Color32::from_rgb(0x16, 0x16, 0x22));
                            painter.rect_stroke(content_rect, 0.0, Stroke::new(1.0, theme::BORDER));

                            // ── Clip blocks ───────────────────────────────────────
                            for clip in clips {
                                let is_selected = selected_clip_id == Some(clip.id);

                                let clip_x = content_rect.left() + clip.timeline_start as f32 * px_per_sec;
                                let clip_w = (clip.duration() as f32 * px_per_sec).max(4.0);
                                let clip_rect = egui::Rect::from_min_size(
                                    egui::pos2(clip_x, content_rect.top() + 2.0),
                                    egui::vec2(clip_w, TRACK_HEIGHT - 4.0),
                                );

                                // ── Clip body color ────────────────────────────────
                                let base_color = if is_video {
                                    if is_selected {
                                        Color32::from_rgb(0x7a, 0x8a, 0xff)
                                    } else {
                                        Color32::from_rgb(0x3a, 0x4a, 0xcc)
                                    }
                                } else if is_selected {
                                    Color32::from_rgb(0x3a, 0xcc, 0x8a)
                                } else {
                                    Color32::from_rgb(0x1a, 0x88, 0x55)
                                };

                                painter.rect_filled(clip_rect, 4.0, base_color);

                                // ── Thumbnail strip (video) or Waveform (audio) ────
                                if clip_w > 20.0 {
                                    if is_video {
                                        draw_thumbnail_strip(&painter, clip_rect, clip.asset_id);
                                    } else {
                                        draw_waveform(&painter, clip_rect, clip.id);
                                    }
                                }

                                // ── Clip border ────────────────────────────────────
                                painter.rect_stroke(
                                    clip_rect,
                                    4.0,
                                    Stroke::new(
                                        if is_selected { 2.0 } else { 1.0 },
                                        if is_selected { Color32::WHITE } else { Color32::from_rgb(0x5a, 0x6a, 0xff) },
                                    ),
                                );

                                // ── Clip label ─────────────────────────────────────
                                if clip_w > 30.0 {
                                    let clip_name = app
                                        .project
                                        .as_ref()
                                        .and_then(|p| p.media_pool.iter().find(|a| a.id == clip.asset_id))
                                        .map(|a| a.name.as_str())
                                        .unwrap_or("Clip");
                                    painter.text(
                                        clip_rect.left_top() + egui::vec2(TRIM_HANDLE_WIDTH + 2.0, 4.0),
                                        egui::Align2::LEFT_TOP,
                                        clip_name,
                                        egui::FontId::proportional(10.0),
                                        Color32::WHITE,
                                    );
                                }

                                // ── Trim handle rects ──────────────────────────────
                                let left_handle = egui::Rect::from_min_size(
                                    clip_rect.min,
                                    egui::vec2(TRIM_HANDLE_WIDTH, clip_rect.height()),
                                );
                                let right_handle = egui::Rect::from_min_size(
                                    egui::pos2(clip_rect.max.x - TRIM_HANDLE_WIDTH, clip_rect.min.y),
                                    egui::vec2(TRIM_HANDLE_WIDTH, clip_rect.height()),
                                );

                                // Draw trim handle visual indicators
                                let handle_color = Color32::from_rgba_premultiplied(0xff, 0xff, 0xff, 0x50);
                                painter.rect_filled(left_handle, egui::Rounding { nw: 4.0, sw: 4.0, ne: 0.0, se: 0.0 }, handle_color);
                                painter.rect_filled(right_handle, egui::Rounding { nw: 0.0, sw: 0.0, ne: 4.0, se: 4.0 }, handle_color);
                                // Grip lines on handles
                                let lx = left_handle.center().x;
                                let rx = right_handle.center().x;
                                let mid_y = clip_rect.center().y;
                                for dy in [-4.0_f32, 0.0, 4.0] {
                                    painter.line_segment(
                                        [egui::pos2(lx - 1.0, mid_y + dy), egui::pos2(lx + 1.0, mid_y + dy)],
                                        Stroke::new(1.0, Color32::from_rgba_premultiplied(0xff, 0xff, 0xff, 0xaa)),
                                    );
                                    painter.line_segment(
                                        [egui::pos2(rx - 1.0, mid_y + dy), egui::pos2(rx + 1.0, mid_y + dy)],
                                        Stroke::new(1.0, Color32::from_rgba_premultiplied(0xff, 0xff, 0xff, 0xaa)),
                                    );
                                }

                                // ── Interact: left trim handle ─────────────────────
                                let clip_id = clip.id;
                                let orig_ts = clip.timeline_start;
                                let orig_te = clip.timeline_end;
                                let orig_ss = clip.source_start;
                                let orig_se = clip.source_end;

                                // Body drag (move), but only the interior (excluding handles)
                                let body_rect = egui::Rect::from_min_max(
                                    clip_rect.min + egui::vec2(TRIM_HANDLE_WIDTH, 0.0),
                                    clip_rect.max - egui::vec2(TRIM_HANDLE_WIDTH, 0.0),
                                );
                                let body_resp = ui.allocate_rect(body_rect, Sense::click_and_drag());
                                // Trim left
                                let left_resp = ui.allocate_rect(left_handle, Sense::drag());
                                // Trim right
                                let right_resp = ui.allocate_rect(right_handle, Sense::drag());

                                // Show resize cursor on trim handles
                                if left_resp.hovered() || right_resp.hovered() {
                                    ctx.set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                                }

                                // ── Body click / drag (move) ───────────────────────
                                if body_resp.clicked() {
                                    app.selected_clip_id = Some(clip_id);
                                }

                                if body_resp.drag_started_by(PointerButton::Primary) {
                                    app.selected_clip_id = Some(clip_id);
                                    app.dragging = Some(crate::app::DragState {
                                        clip_id,
                                        track_id: *track_id,
                                        original_start: orig_ts,
                                        duration: clip.duration(),
                                        current_start: orig_ts,
                                    });
                                }

                                if body_resp.dragged_by(PointerButton::Primary) {
                                    if let Some(drag) = &mut app.dragging {
                                        if drag.clip_id == clip_id {
                                            let delta_px = body_resp.drag_delta().x;
                                            let delta_secs = delta_px as f64 / px_per_sec as f64;
                                            let raw = (drag.current_start + delta_secs).max(0.0);
                                            let snapped = snap_to_frame(raw, frame_dur);

                                            // Overlap check
                                            let proposed = snapped;
                                            let allowed = if let Some(proj) = &app.project {
                                                !would_overlap(proj, *track_id, clip_id, proposed, drag.duration)
                                            } else {
                                                true
                                            };

                                            if allowed {
                                                drag.current_start = proposed;
                                                if let Some(project) = &mut app.project {
                                                    if let Some(track) = project.timeline.tracks.iter_mut().find(|t| t.id == drag.track_id) {
                                                        if let Some(c) = track.clips.iter_mut().find(|c| c.id == clip_id) {
                                                            c.timeline_start = proposed;
                                                            c.timeline_end = proposed + drag.duration;
                                                        }
                                                    }
                                                    project.timeline.recompute_duration();
                                                }
                                            }
                                        }
                                    }
                                }

                                if body_resp.drag_stopped() {
                                    if let Some(drag) = app.dragging.take() {
                                        if drag.clip_id == clip_id {
                                            let new_start = drag.current_start;
                                            if (new_start - drag.original_start).abs() > 0.001 {
                                                app.action_commit_move_clip(
                                                    drag.clip_id,
                                                    drag.track_id,
                                                    drag.original_start,
                                                    new_start,
                                                    drag.duration,
                                                );
                                            }
                                        }
                                    }
                                }

                                // ── Left trim handle drag ──────────────────────────
                                if left_resp.drag_started_by(PointerButton::Primary) {
                                    app.selected_clip_id = Some(clip_id);
                                    app.trim_dragging = Some(TrimDragState {
                                        clip_id,
                                        track_id: *track_id,
                                        edge: TrimEdge::Left,
                                        orig_timeline_start: orig_ts,
                                        orig_timeline_end: orig_te,
                                        orig_source_start: orig_ss,
                                        orig_source_end: orig_se,
                                        cur_timeline_start: orig_ts,
                                        cur_timeline_end: orig_te,
                                        cur_source_start: orig_ss,
                                        cur_source_end: orig_se,
                                    });
                                }

                                if left_resp.dragged_by(PointerButton::Primary) {
                                    if let Some(td) = &mut app.trim_dragging {
                                        if td.clip_id == clip_id && td.edge == TrimEdge::Left {
                                            let delta_secs = left_resp.drag_delta().x as f64 / px_per_sec as f64;
                                            let raw_new_start = (td.cur_timeline_start + delta_secs)
                                                .max(0.0)
                                                .min(td.cur_timeline_end - MIN_CLIP_DURATION);
                                            let new_start = snap_to_frame(raw_new_start, frame_dur);
                                            let trim_amount = new_start - td.orig_timeline_start;
                                            td.cur_timeline_start = new_start;
                                            td.cur_source_start = (td.orig_source_start + trim_amount).max(0.0);
                                            // Apply live
                                            if let Some(project) = &mut app.project {
                                                if let Some(track) = project.timeline.tracks.iter_mut().find(|t| t.id == td.track_id) {
                                                    if let Some(c) = track.clips.iter_mut().find(|c| c.id == clip_id) {
                                                        c.timeline_start = td.cur_timeline_start;
                                                        c.source_start = td.cur_source_start;
                                                    }
                                                }
                                                project.timeline.recompute_duration();
                                            }
                                        }
                                    }
                                }

                                if left_resp.drag_stopped() {
                                    if let Some(td) = app.trim_dragging.take() {
                                        if td.clip_id == clip_id
                                            && td.edge == TrimEdge::Left
                                            && (td.cur_timeline_start - td.orig_timeline_start).abs() > 0.001
                                        {
                                            app.action_commit_trim_clip(
                                                clip_id, td.track_id,
                                                td.orig_timeline_start, td.orig_timeline_end,
                                                td.orig_source_start, td.orig_source_end,
                                                td.cur_timeline_start, td.cur_timeline_end,
                                                td.cur_source_start, td.cur_source_end,
                                            );
                                        }
                                    }
                                }

                                // ── Right trim handle drag ─────────────────────────
                                if right_resp.drag_started_by(PointerButton::Primary) {
                                    app.selected_clip_id = Some(clip_id);
                                    app.trim_dragging = Some(TrimDragState {
                                        clip_id,
                                        track_id: *track_id,
                                        edge: TrimEdge::Right,
                                        orig_timeline_start: orig_ts,
                                        orig_timeline_end: orig_te,
                                        orig_source_start: orig_ss,
                                        orig_source_end: orig_se,
                                        cur_timeline_start: orig_ts,
                                        cur_timeline_end: orig_te,
                                        cur_source_start: orig_ss,
                                        cur_source_end: orig_se,
                                    });
                                }

                                if right_resp.dragged_by(PointerButton::Primary) {
                                    if let Some(td) = &mut app.trim_dragging {
                                        if td.clip_id == clip_id && td.edge == TrimEdge::Right {
                                            let delta_secs = right_resp.drag_delta().x as f64 / px_per_sec as f64;
                                            let raw_new_end = (td.cur_timeline_end + delta_secs)
                                                .max(td.cur_timeline_start + MIN_CLIP_DURATION);
                                            let new_end = snap_to_frame(raw_new_end, frame_dur);
                                            let trim_amount = new_end - td.orig_timeline_end;
                                            td.cur_timeline_end = new_end;
                                            td.cur_source_end = (td.orig_source_end + trim_amount)
                                                .max(td.cur_source_start + MIN_CLIP_DURATION);
                                            // Apply live
                                            if let Some(project) = &mut app.project {
                                                if let Some(track) = project.timeline.tracks.iter_mut().find(|t| t.id == td.track_id) {
                                                    if let Some(c) = track.clips.iter_mut().find(|c| c.id == clip_id) {
                                                        c.timeline_end = td.cur_timeline_end;
                                                        c.source_end = td.cur_source_end;
                                                    }
                                                }
                                                project.timeline.recompute_duration();
                                            }
                                        }
                                    }
                                }

                                if right_resp.drag_stopped() {
                                    if let Some(td) = app.trim_dragging.take() {
                                        if td.clip_id == clip_id
                                            && td.edge == TrimEdge::Right
                                            && (td.cur_timeline_end - td.orig_timeline_end).abs() > 0.001
                                        {
                                            app.action_commit_trim_clip(
                                                clip_id, td.track_id,
                                                td.orig_timeline_start, td.orig_timeline_end,
                                                td.orig_source_start, td.orig_source_end,
                                                td.cur_timeline_start, td.cur_timeline_end,
                                                td.cur_source_start, td.cur_source_end,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // ── Playhead line ──────────────────────────────────────────
                    let ph_x = TRACK_HEADER_WIDTH + playhead_secs as f32 * px_per_sec;
                    let panel_top = ui.min_rect().top();
                    let panel_bottom = ui.min_rect().bottom();
                    ui.painter().line_segment(
                        [
                            egui::pos2(ui.min_rect().left() + ph_x, panel_top),
                            egui::pos2(ui.min_rect().left() + ph_x, panel_bottom),
                        ],
                        Stroke::new(2.0, Color32::from_rgb(0xff, 0x44, 0x44)),
                    );
                    // Playhead diamond handle
                    let ph_tip = egui::pos2(ui.min_rect().left() + ph_x, panel_top + RULER_HEIGHT - 2.0);
                    ui.painter().add(egui::Shape::convex_polygon(
                        vec![
                            ph_tip,
                            ph_tip + egui::vec2(-5.0, -8.0),
                            ph_tip + egui::vec2(5.0, -8.0),
                        ],
                        Color32::from_rgb(0xff, 0x44, 0x44),
                        Stroke::NONE,
                    ));
                });
        });
}

// ── Drawing helpers ───────────────────────────────────────────────────────────

/// Draw the time ruler. Returns click position in content coordinates if clicked.
fn draw_ruler(ui: &mut egui::Ui, width: f32, total_secs: f64, px_per_sec: f32) -> Option<f32> {
    let ruler_width = TRACK_HEADER_WIDTH + width;
    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(ruler_width, RULER_HEIGHT),
        Sense::click(),
    );

    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(0x18, 0x18, 0x24));

    // Separator between header and ruler content
    painter.line_segment(
        [
            egui::pos2(rect.left() + TRACK_HEADER_WIDTH, rect.top()),
            egui::pos2(rect.left() + TRACK_HEADER_WIDTH, rect.bottom()),
        ],
        Stroke::new(1.0, theme::BORDER),
    );

    // Tick marks
    let visible_secs = (width / px_per_sec) as f64;
    let tick_interval = pick_tick_interval(visible_secs / (width as f64 / 60.0));
    let mut t = 0.0_f64;
    while t <= total_secs + tick_interval {
        let x = rect.left() + TRACK_HEADER_WIDTH + t as f32 * px_per_sec;
        if x > rect.right() {
            break;
        }
        let is_major = (t / tick_interval).round() % 5.0 < 0.5;
        let tick_h = if is_major { RULER_HEIGHT * 0.55 } else { RULER_HEIGHT * 0.3 };
        painter.line_segment(
            [egui::pos2(x, rect.bottom() - tick_h), egui::pos2(x, rect.bottom())],
            Stroke::new(1.0, egui::Color32::from_rgb(0x44, 0x44, 0x60)),
        );
        if is_major {
            painter.text(
                egui::pos2(x + 3.0, rect.top() + 3.0),
                egui::Align2::LEFT_TOP,
                format_timecode(t),
                egui::FontId::monospace(9.0),
                egui::Color32::from_rgb(0x60, 0x60, 0x80),
            );
        }
        t += tick_interval;
    }

    // Return click position relative to ruler content.
    if resp.clicked() {
        resp.interact_pointer_pos().map(|p| (p.x - rect.left() - TRACK_HEADER_WIDTH).max(0.0))
    } else {
        None
    }
}

/// Draw a fake but visually convincing waveform inside an audio clip block.
fn draw_waveform(painter: &egui::Painter, clip_rect: egui::Rect, clip_id: uuid::Uuid) {
    let bar_w = 3.0_f32;
    let bar_gap = 1.0_f32;
    let step = bar_w + bar_gap;
    let max_h = (clip_rect.height() - 6.0).max(2.0);
    let mid_y = clip_rect.center().y;
    let left = clip_rect.min.x + TRIM_HANDLE_WIDTH + 2.0;
    let right = clip_rect.max.x - TRIM_HANDLE_WIDTH - 2.0;
    let waveform_color = Color32::from_rgba_premultiplied(0x80, 0xff, 0xb0, 0x55);

    // Deterministic seed from clip UUID bytes
    let bytes = clip_id.as_bytes();
    let mut x = left;
    let mut idx = 0usize;
    while x + bar_w <= right {
        let seed = bytes[idx % 16] as f32;
        let h = (((seed * 7.0 + (x * 0.13).sin() * 50.0).abs() % 100.0) / 100.0 * max_h).max(2.0);
        painter.rect_filled(
            egui::Rect::from_center_size(
                egui::pos2(x + bar_w * 0.5, mid_y),
                egui::vec2(bar_w, h),
            ),
            1.0,
            waveform_color,
        );
        x += step;
        idx += 1;
    }
}

/// Draw a fake thumbnail strip inside a video clip block.
fn draw_thumbnail_strip(painter: &egui::Painter, clip_rect: egui::Rect, asset_id: uuid::Uuid) {
    let thumb_w = clip_rect.height() - 4.0; // square thumbnails
    let left = clip_rect.min.x + TRIM_HANDLE_WIDTH + 2.0;
    let right = clip_rect.max.x - TRIM_HANDLE_WIDTH - 2.0;
    let top = clip_rect.min.y + 2.0;

    // Deterministic palette from asset UUID
    let bytes = asset_id.as_bytes();
    let base_r = bytes[0];
    let base_g = bytes[1];
    let base_b = bytes[2];

    let mut x = left;
    let mut idx = 0usize;
    while x + thumb_w <= right {
        let variation = bytes[idx % 16] as i16;
        let r = ((base_r as i16 + variation / 4).clamp(0x10, 0x60)) as u8;
        let g = ((base_g as i16 + variation / 3).clamp(0x10, 0x60)) as u8;
        let b = ((base_b as i16 - variation / 4).clamp(0x20, 0x80)) as u8;
        let thumb_rect = egui::Rect::from_min_size(
            egui::pos2(x, top),
            egui::vec2(thumb_w - 1.0, clip_rect.height() - 4.0),
        );
        painter.rect_filled(thumb_rect, 2.0, Color32::from_rgb(r, g, b));
        // Play icon on first thumb
        if idx == 0 {
            painter.text(
                thumb_rect.center(),
                egui::Align2::CENTER_CENTER,
                "▶",
                egui::FontId::proportional(8.0),
                Color32::from_rgba_premultiplied(0xff, 0xff, 0xff, 0x60),
            );
        }
        x += thumb_w;
        idx += 1;
    }
}

/// Snap a time value to the nearest frame boundary.
fn snap_to_frame(secs: f64, frame_dur: f64) -> f64 {
    if frame_dur <= 0.0 {
        return secs;
    }
    (secs / frame_dur).round() * frame_dur
}

/// Check if placing a clip at `start` with `duration` on `track_id` would overlap
/// any other clip (excluding `clip_id` itself).
fn would_overlap(
    project: &vidcut_core::Project,
    track_id: uuid::Uuid,
    clip_id: uuid::Uuid,
    start: f64,
    duration: f64,
) -> bool {
    let end = start + duration;
    if let Some(track) = project.timeline.tracks.iter().find(|t| t.id == track_id) {
        for other in &track.clips {
            if other.id == clip_id {
                continue;
            }
            // Overlap if not (end <= other.start || start >= other.end)
            let gap = 0.001;
            if end > other.timeline_start + gap && start < other.timeline_end - gap {
                return true;
            }
        }
    }
    false
}

fn pick_tick_interval(secs_per_60px: f64) -> f64 {
    let candidates = [0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0, 600.0];
    for &c in &candidates {
        if c >= secs_per_60px * 0.8 {
            return c;
        }
    }
    600.0
}

fn format_timecode(secs: f64) -> String {
    let m = (secs / 60.0) as u32;
    let s = (secs % 60.0) as u32;
    if m > 0 {
        format!("{m}:{s:02}")
    } else {
        format!("{s}s")
    }
}
