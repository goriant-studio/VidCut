//! Preview panel — central area displaying the video preview.
//!
//! Rendered as an `egui::CentralPanel`.
//! Phase 1: dark background with a "Preview" label and aspect-ratio frame.
//! Phase 2: will display a `wgpu` texture streamed from the decode pipeline.

use eframe::egui::{self, Color32, RichText, Stroke};

use crate::{app::VidCutApp, panels::theme};

/// Show the preview panel. Called every frame from [`VidCutApp::update`].
pub fn show(ctx: &egui::Context, app: &mut VidCutApp) {
    // The central panel sits between the side panels and above the timeline.
    // We allocate the bottom 200px to the timeline, so we render the preview
    // in whatever space remains in the centre.
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(theme::BG_PANEL))
        .show(ctx, |ui| {
            // ── Preview viewport ───────────────────────────────────────────────
            let available = ui.available_size();
            // Target 16:9 aspect ratio, centred within available space.
            let aspect = 16.0 / 9.0;
            let (pw, _ph) = if available.x / available.y > aspect {
                (available.y * aspect, available.y)
            } else {
                (available.x, available.x / aspect)
            };
            let pw = pw.min(available.x - 32.0);
            let ph = (pw / aspect).min(available.y - 48.0);

            ui.add_space((available.y - ph - 40.0).max(8.0) / 2.0);

            ui.vertical_centered(|ui| {
                // ── Preview frame ──────────────────────────────────────────────
                let (rect, _) = ui.allocate_exact_size(egui::vec2(pw, ph), egui::Sense::hover());
                let painter = ui.painter();

                // Background
                painter.rect_filled(rect, egui::Rounding::same(4.0), Color32::from_rgb(0x08, 0x08, 0x0e));

                // Phase 2 placeholder — crosshair + label
                let cx = rect.center();
                let half = 20.0_f32;
                let stroke = Stroke::new(1.0, Color32::from_rgb(0x35, 0x35, 0x55));
                painter.line_segment([cx - egui::vec2(half, 0.0), cx + egui::vec2(half, 0.0)], stroke);
                painter.line_segment([cx - egui::vec2(0.0, half), cx + egui::vec2(0.0, half)], stroke);

                painter.text(
                    cx,
                    egui::Align2::CENTER_CENTER,
                    "Preview",
                    egui::FontId::proportional(16.0),
                    Color32::from_rgb(0x40, 0x40, 0x60),
                );

                // Border
                painter.rect_stroke(rect, egui::Rounding::same(4.0), Stroke::new(1.0, theme::BORDER));

                // ── Timecode overlay (bottom-left of frame) ────────────────────
                let secs = app.playhead_secs();
                let h = (secs / 3600.0) as u32;
                let m = ((secs % 3600.0) / 60.0) as u32;
                let s = (secs % 60.0) as u32;
                let f = (secs.fract() * 30.0) as u32;
                painter.text(
                    rect.left_bottom() + egui::vec2(8.0, -8.0),
                    egui::Align2::LEFT_BOTTOM,
                    format!("{h:02}:{m:02}:{s:02}:{f:02}"),
                    egui::FontId::monospace(11.0),
                    Color32::from_rgba_premultiplied(0xb0, 0xb8, 0xff, 0xcc),
                );

                // ── Resolution label (bottom-right) ────────────────────────────
                let (w, h_px) = app.preview_resolution();
                painter.text(
                    rect.right_bottom() + egui::vec2(-8.0, -8.0),
                    egui::Align2::RIGHT_BOTTOM,
                    format!("{w}×{h_px}"),
                    egui::FontId::monospace(11.0),
                    Color32::from_rgba_premultiplied(0x60, 0x60, 0x80, 0xcc),
                );
            });

            // ── Transport hint ─────────────────────────────────────────────────
            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("Space to play · J/K/L for speed · ← → to step frames")
                        .color(theme::TEXT_MUTED)
                        .size(11.0),
                );
            });
        });
}
