//! Timeline panel — bottom panel with track rows and playhead.
//!
//! Rendered as an `egui::TopBottomPanel::bottom` with a height of 200 px.
//! Phase 1: placeholder track rows with ruler.
//! Phase 2: full clip drag/drop, trim handles, waveform / thumbnail strips.

use eframe::egui::{self, Color32, RichText, Stroke};

use crate::{app::VidCutApp, panels::theme};

const TRACK_HEADER_WIDTH: f32 = 120.0;
const TRACK_HEIGHT: f32 = 44.0;
const RULER_HEIGHT: f32 = 20.0;

/// Show the timeline panel. Called every frame from [`VidCutApp::update`].
pub fn show(ctx: &egui::Context, app: &mut VidCutApp) {
    egui::TopBottomPanel::bottom("timeline")
        .exact_height(200.0)
        .resizable(true)
        .frame(
            egui::Frame::none()
                .fill(theme::BG_DEEP)
                .inner_margin(egui::Margin::same(0.0)),
        )
        .show(ctx, |ui| {
            // ── Timeline header ────────────────────────────────────────────────
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
                        // Zoom indicator (placeholder)
                        ui.label(
                            RichText::new("1× zoom")
                                .color(theme::TEXT_MUTED)
                                .size(11.0),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("+ Track").clicked() {
                                app.action_add_track();
                            }
                        });
                    });
                });

            ui.separator();

            // ── Ruler + track area ─────────────────────────────────────────────
            let available = ui.available_size();

            egui::ScrollArea::both().show(ui, |ui| {
                ui.set_min_size(available);

                let track_area_width = (available.x - TRACK_HEADER_WIDTH).max(200.0);
                let total_secs = app.project_duration().max(30.0);

                // Ruler
                draw_ruler(ui, track_area_width, total_secs);

                // Placeholder track rows
                let track_names: Vec<(&str, Color32)> = vec![
                    ("Video 1", Color32::from_rgb(0x4a, 0x5a, 0xff)),
                    ("Video 2", Color32::from_rgb(0x4a, 0x5a, 0xff)),
                    ("Audio 1", Color32::from_rgb(0x2a, 0xaa, 0x6a)),
                    ("Audio 2", Color32::from_rgb(0x2a, 0xaa, 0x6a)),
                ];

                for (name, color) in &track_names {
                    draw_track_row(ui, name, *color, track_area_width);
                }
            });
        });
}

// ── Drawing helpers ───────────────────────────────────────────────────────────

fn draw_ruler(ui: &mut egui::Ui, width: f32, total_secs: f64) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(TRACK_HEADER_WIDTH + width, RULER_HEIGHT),
        egui::Sense::hover(),
    );
    let painter = ui.painter();

    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(0x18, 0x18, 0x24));

    // Draw second ticks
    let secs_per_pixel = total_secs / width as f64;
    let tick_interval_secs = pick_tick_interval(secs_per_pixel);
    let mut t = 0.0_f64;
    while t <= total_secs {
        let x = rect.left() + TRACK_HEADER_WIDTH + (t / total_secs) as f32 * width;
        let is_major = (t % (tick_interval_secs * 5.0)).abs() < 0.001;
        let tick_h = if is_major { 10.0 } else { 5.0 };
        painter.line_segment(
            [
                egui::pos2(x, rect.bottom() - tick_h),
                egui::pos2(x, rect.bottom()),
            ],
            Stroke::new(1.0, egui::Color32::from_rgb(0x44, 0x44, 0x60)),
        );
        if is_major {
            let label = format_timecode(t);
            painter.text(
                egui::pos2(x + 3.0, rect.top() + 4.0),
                egui::Align2::LEFT_TOP,
                label,
                egui::FontId::monospace(10.0),
                egui::Color32::from_rgb(0x60, 0x60, 0x80),
            );
        }
        t += tick_interval_secs;
    }
}

fn draw_track_row(ui: &mut egui::Ui, name: &str, accent: Color32, width: f32) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(TRACK_HEADER_WIDTH + width, TRACK_HEIGHT),
        egui::Sense::hover(),
    );
    let painter = ui.painter();

    // Track header
    let header_rect = egui::Rect::from_min_size(rect.min, egui::vec2(TRACK_HEADER_WIDTH, TRACK_HEIGHT));
    painter.rect_filled(header_rect, 0.0, egui::Color32::from_rgb(0x1e, 0x1e, 0x2e));
    painter.rect_stroke(header_rect, 0.0, Stroke::new(1.0, theme::BORDER));

    // Colour swatch
    let swatch = egui::Rect::from_min_size(header_rect.min, egui::vec2(3.0, TRACK_HEIGHT));
    painter.rect_filled(swatch, 0.0, accent);

    painter.text(
        header_rect.left_center() + egui::vec2(10.0, 0.0),
        egui::Align2::LEFT_CENTER,
        name,
        egui::FontId::proportional(12.0),
        theme::TEXT_PRIMARY,
    );

    // Track content area
    let content_rect = egui::Rect::from_min_size(
        rect.min + egui::vec2(TRACK_HEADER_WIDTH, 0.0),
        egui::vec2(width, TRACK_HEIGHT),
    );
    painter.rect_filled(content_rect, 0.0, egui::Color32::from_rgb(0x16, 0x16, 0x22));
    painter.rect_stroke(content_rect, 0.0, Stroke::new(1.0, theme::BORDER));
}

/// Choose a sensible tick interval given current secs-per-pixel ratio.
fn pick_tick_interval(secs_per_pixel: f64) -> f64 {
    let candidates = [0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 300.0];
    for &c in &candidates {
        if c / secs_per_pixel > 60.0 {
            return c;
        }
    }
    300.0
}

fn format_timecode(secs: f64) -> String {
    let m = (secs / 60.0) as u32;
    let s = (secs % 60.0) as u32;
    format!("{m:02}:{s:02}")
}
