//! FFmpeg setup overlay — shown during first-run FFmpeg download.
//!
//! Renders a full-screen dim + centred card while [`FfmpegStatus`] is
//! `Checking` or `Downloading`.  Disappears automatically once `Ready`.
//! Shows an error card with a **Retry** button on `Failed`.

use eframe::egui::{self, Color32, RichText};
use vidcut_media::FfmpegStatus;

use crate::{app::VidCutApp, panels::theme};

// ─── Public entry point ───────────────────────────────────────────────────────

/// Render the FFmpeg setup overlay if required. Called every frame from
/// [`VidCutApp::update`] — must be the last panel so it sits on top.
pub fn show(ctx: &egui::Context, app: &mut VidCutApp) {
    match &app.ffmpeg_status {
        FfmpegStatus::Ready => return, // nothing to show
        FfmpegStatus::Checking => show_checking(ctx),
        FfmpegStatus::Downloading => show_downloading(ctx),
        FfmpegStatus::Failed(msg) => show_failed(ctx, app, msg.clone()),
    }
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

fn dim_background(ctx: &egui::Context) {
    let screen = ctx.screen_rect();
    ctx.layer_painter(egui::LayerId::new(
        egui::Order::Background,
        egui::Id::new("ffmpeg_setup_bg"),
    ))
    .rect_filled(screen, 0.0, Color32::from_rgba_premultiplied(0, 0, 0, 200));
}

fn show_checking(ctx: &egui::Context) {
    dim_background(ctx);
    egui::Window::new("ffmpeg_checking")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .fixed_size([340.0, 110.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(card_frame(ctx))
        .show(ctx, |ui| {
            ui.add_space(12.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("⏳  Setting up VidCut…")
                        .color(Color32::WHITE)
                        .size(15.0)
                        .strong(),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Checking for FFmpeg…")
                        .color(theme::TEXT_MUTED)
                        .size(12.0),
                );
            });
            ui.add_space(12.0);
        });
    ctx.request_repaint();
}

fn show_downloading(ctx: &egui::Context) {
    dim_background(ctx);
    egui::Window::new("ffmpeg_downloading")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .fixed_size([380.0, 150.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(card_frame(ctx))
        .show(ctx, |ui| {
            ui.add_space(14.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("⬇  Downloading FFmpeg")
                        .color(Color32::WHITE)
                        .size(15.0)
                        .strong(),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(
                        "FFmpeg is needed for export. This is a one-time download (~70 MB).",
                    )
                    .color(theme::TEXT_MUTED)
                    .size(11.0),
                );
            });
            ui.add_space(12.0);
            // Indeterminate spinner using an animated progress bar.
            let t = ctx.input(|i| i.time) as f32;
            let animated_fraction = (t * 0.5).sin() * 0.5 + 0.5;
            ui.add(
                egui::ProgressBar::new(animated_fraction)
                    .desired_width(ui.available_width() - 16.0)
                    .fill(theme::ACCENT),
            );
            ui.add_space(6.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("Please wait, this may take a minute…")
                        .color(theme::TEXT_MUTED)
                        .size(10.0)
                        .monospace(),
                );
            });
            ui.add_space(12.0);
        });
    ctx.request_repaint();
}

fn show_failed(ctx: &egui::Context, app: &mut VidCutApp, message: String) {
    dim_background(ctx);
    egui::Window::new("ffmpeg_failed")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .fixed_size([400.0, 200.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(card_frame(ctx))
        .show(ctx, |ui| {
            ui.add_space(14.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("✗  FFmpeg Setup Failed")
                        .color(Color32::from_rgb(0xff, 0x55, 0x55))
                        .size(15.0)
                        .strong(),
                );
            });
            ui.add_space(8.0);
            ui.label(
                RichText::new(&message)
                    .color(theme::TEXT_MUTED)
                    .size(11.0),
            );
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("🔄  Retry")
                                .color(Color32::WHITE)
                                .size(13.0)
                                .strong(),
                        )
                        .fill(theme::ACCENT)
                        .min_size(egui::vec2(100.0, 28.0)),
                    )
                    .clicked()
                {
                    app.action_retry_ffmpeg_setup();
                }
                ui.add_space(8.0);
                ui.label(
                    RichText::new("or install FFmpeg manually and add it to your PATH")
                        .color(theme::TEXT_MUTED)
                        .size(11.0),
                );
            });
            ui.add_space(12.0);
        });
}

// ─── Frame helper ─────────────────────────────────────────────────────────────

fn card_frame(ctx: &egui::Context) -> egui::Frame {
    egui::Frame::window(&ctx.style())
        .fill(egui::Color32::from_rgb(0x14, 0x14, 0x24))
        .stroke(egui::Stroke::new(1.5, theme::BORDER))
        .inner_margin(egui::Margin::symmetric(16.0, 8.0))
}
