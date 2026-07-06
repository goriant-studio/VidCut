//! Preview panel — central area displaying the video preview.
//!
//! Rendered as an `egui::CentralPanel`.
//! Phase 2: dark background with transport info overlay (timecode, speed, resolution).
//! Phase 3: will display a `wgpu` texture streamed from the decode pipeline.

use eframe::egui::{self, Color32, RichText, Stroke};

use crate::{app::VidCutApp, panels::theme};

/// Show the preview panel. Called every frame from [`VidCutApp::update`].
pub fn show(ctx: &egui::Context, app: &mut VidCutApp) {
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(theme::BG_PANEL))
        .show(ctx, |ui| {
            // ── Preview viewport ───────────────────────────────────────────────
            let available = ui.available_size();
            let aspect = 16.0 / 9.0;
            let (pw, _ph) = if available.x / available.y > aspect {
                (available.y * aspect, available.y)
            } else {
                (available.x, available.x / aspect)
            };
            let pw = pw.min(available.x - 32.0);
            let ph = (pw / aspect).min(available.y - 60.0);

            ui.add_space((available.y - ph - 60.0).max(8.0) / 2.0);

            ui.vertical_centered(|ui| {
                // ── Preview frame ──────────────────────────────────────────────
                let (rect, _) = ui.allocate_exact_size(egui::vec2(pw, ph), egui::Sense::hover());
                let painter = ui.painter();

                // Background
                painter.rect_filled(rect, egui::Rounding::same(4.0), Color32::from_rgb(0x08, 0x08, 0x0e));

                // Crosshair
                let cx = rect.center();
                let half = 20.0_f32;
                let stroke = Stroke::new(1.0, Color32::from_rgb(0x25, 0x25, 0x45));
                painter.line_segment([cx - egui::vec2(half, 0.0), cx + egui::vec2(half, 0.0)], stroke);
                painter.line_segment([cx - egui::vec2(0.0, half), cx + egui::vec2(0.0, half)], stroke);

                // "Preview" label
                painter.text(
                    cx,
                    egui::Align2::CENTER_CENTER,
                    "Preview",
                    egui::FontId::proportional(14.0),
                    Color32::from_rgb(0x30, 0x30, 0x50),
                );

                // Border
                painter.rect_stroke(rect, egui::Rounding::same(4.0), Stroke::new(1.0, theme::BORDER));

                // ── Timecode overlay (bottom-left) ─────────────────────────────
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

                // ── Speed overlay (bottom-centre) ──────────────────────────────
                let speed = app.playback_speed;
                if speed != 1.0 || !app.is_playing() {
                    let speed_str = if speed == 1.0 && !app.is_playing() {
                        "■ Stopped".to_owned()
                    } else if speed < 0.0 {
                        format!("◀◀ {:.1}×", speed.abs())
                    } else if speed > 1.0 {
                        format!("▶▶ {speed:.1}×")
                    } else if speed < 1.0 {
                        format!("▶ {speed:.2}×")
                    } else {
                        "▶ 1×".to_owned()
                    };

                    let speed_color = if !app.is_playing() {
                        Color32::from_rgba_premultiplied(0x60, 0x68, 0x80, 0xaa)
                    } else if speed == 1.0 {
                        Color32::from_rgba_premultiplied(0x80, 0xc0, 0x80, 0xcc)
                    } else {
                        Color32::from_rgba_premultiplied(0xff, 0xc0, 0x60, 0xcc)
                    };

                    painter.text(
                        rect.center_bottom() + egui::vec2(0.0, -8.0),
                        egui::Align2::CENTER_BOTTOM,
                        speed_str,
                        egui::FontId::monospace(10.0),
                        speed_color,
                    );
                }

                // ── Resolution label (bottom-right) ────────────────────────────
                let (w, h_px) = app.preview_resolution();
                painter.text(
                    rect.right_bottom() + egui::vec2(-8.0, -8.0),
                    egui::Align2::RIGHT_BOTTOM,
                    format!("{w}×{h_px}"),
                    egui::FontId::monospace(11.0),
                    Color32::from_rgba_premultiplied(0x60, 0x60, 0x80, 0xcc),
                );

                // ── Play indicator (top-right) ─────────────────────────────────
                if app.is_playing() {
                    painter.circle_filled(
                        rect.right_top() + egui::vec2(-12.0, 12.0),
                        5.0,
                        Color32::from_rgb(0xff, 0x44, 0x44),
                    );
                }
            });

            // ── Transport hint ─────────────────────────────────────────────────
            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("Space: play/pause  ·  J/K/L: reverse/pause/play  ·  ←/→: frame step  ·  Del: delete clip")
                        .color(theme::TEXT_MUTED)
                        .size(10.0),
                );
            });
        });
}
