//! Timeline panel — bottom panel with track rows, clip blocks, and playhead.
//!
//! Phase 2: Renders actual tracks + clips from the project.
//! - Clip blocks are clickable (select) and draggable (move).
//! - Ruler click sets playhead.
//! - Scroll wheel zooms in/out.
//! - Delete key removes selected clip (handled in app.rs keyboard shortcuts).

use eframe::egui::{self, Color32, PointerButton, RichText, Sense, Stroke};
use vidcut_core::TrackType;

use crate::{app::VidCutApp, panels::theme};

const TRACK_HEADER_WIDTH: f32 = 130.0;
const TRACK_HEIGHT: f32 = 48.0;
const RULER_HEIGHT: f32 = 22.0;
const MIN_PX_PER_SEC: f32 = 5.0;
const MAX_PX_PER_SEC: f32 = 800.0;

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
                        ui.label(
                            RichText::new(format!("{pps:.0}px/s"))
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
                        // click_x is relative to the ruler's content area start
                        let secs = (click_x / px_per_sec) as f64;
                        app.playhead_secs = secs.max(0.0);
                    }

                    // ── Tracks ─────────────────────────────────────────────────
                    // Snapshot the project data we need without holding &mut app.
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
                        // Empty state
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
                            let track_color = match track_type {
                                TrackType::Video => Color32::from_rgb(0x4a, 0x5a, 0xff),
                                TrackType::Audio => Color32::from_rgb(0x2a, 0xaa, 0x6a),
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

                            painter.text(
                                header_rect.left_center() + egui::vec2(10.0, 0.0),
                                egui::Align2::LEFT_CENTER,
                                track_name.as_str(),
                                egui::FontId::proportional(11.0),
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

                                let clip_color = if is_selected {
                                    Color32::from_rgb(0x8a, 0x9a, 0xff)
                                } else {
                                    Color32::from_rgb(0x3a, 0x4a, 0xcc)
                                };

                                painter.rect_filled(clip_rect, 4.0, clip_color);
                                painter.rect_stroke(
                                    clip_rect,
                                    4.0,
                                    Stroke::new(
                                        if is_selected { 2.0 } else { 1.0 },
                                        if is_selected { Color32::WHITE } else { Color32::from_rgb(0x5a, 0x6a, 0xff) },
                                    ),
                                );

                                // Clip label
                                if clip_w > 30.0 {
                                    // Lookup asset name from project
                                    let clip_name = app
                                        .project
                                        .as_ref()
                                        .and_then(|p| p.media_pool.iter().find(|a| a.id == clip.asset_id))
                                        .map(|a| a.name.as_str())
                                        .unwrap_or("Clip");
                                    painter.text(
                                        clip_rect.left_center() + egui::vec2(6.0, 0.0),
                                        egui::Align2::LEFT_CENTER,
                                        clip_name,
                                        egui::FontId::proportional(10.0),
                                        Color32::WHITE,
                                    );
                                }

                                // Allocate interactive sense on the clip rect.
                                let clip_id = clip.id;
                                let original_start = clip.timeline_start;
                                let clip_duration = clip.duration();

                                let clip_resp = ui.allocate_rect(clip_rect, Sense::click_and_drag());

                                if clip_resp.clicked() {
                                    app.selected_clip_id = Some(clip_id);
                                }

                                // Drag handling
                                if clip_resp.drag_started_by(PointerButton::Primary) {
                                    app.selected_clip_id = Some(clip_id);
                                    app.dragging = Some(crate::app::DragState {
                                        clip_id,
                                        track_id: *track_id,
                                        original_start,
                                        duration: clip_duration,
                                        current_start: original_start,
                                    });
                                }

                                if clip_resp.dragged_by(PointerButton::Primary) {
                                    if let Some(drag) = &mut app.dragging {
                                        if drag.clip_id == clip_id {
                                            let delta_px = clip_resp.drag_delta().x;
                                            let delta_secs = delta_px as f64 / px_per_sec as f64;
                                            drag.current_start = (drag.current_start + delta_secs).max(0.0);

                                            // Apply live visual update directly (without command history).
                                            if let Some(project) = &mut app.project {
                                                if let Some(track) = project.timeline.tracks.iter_mut().find(|t| t.id == drag.track_id) {
                                                    if let Some(c) = track.clips.iter_mut().find(|c| c.id == clip_id) {
                                                        let new_start = drag.current_start.max(0.0);
                                                        c.timeline_start = new_start;
                                                        c.timeline_end = new_start + drag.duration;
                                                    }
                                                }
                                                project.timeline.recompute_duration();
                                            }
                                        }
                                    }
                                }

                                if clip_resp.drag_stopped() {
                                    if let Some(drag) = app.dragging.take() {
                                        if drag.clip_id == clip_id {
                                            let new_start = drag.current_start;
                                            // Only push command if actually moved.
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
