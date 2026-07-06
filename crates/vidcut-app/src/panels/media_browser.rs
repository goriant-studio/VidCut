//! Media Browser panel — left sidebar showing the imported media pool.
//!
//! Rendered as an `egui::SidePanel::left` with a fixed width of 250 px.
//! Phase 1: displays a placeholder empty state.
//! Phase 2: will list assets with thumbnails and support drag-to-timeline.

use eframe::egui::{self, RichText};

use crate::{app::VidCutApp, panels::theme};

/// Show the media browser panel. Called every frame from [`VidCutApp::update`].
pub fn show(ctx: &egui::Context, app: &mut VidCutApp) {
    egui::SidePanel::left("media_browser")
        .exact_width(250.0)
        .resizable(true)
        .frame(
            egui::Frame::none()
                .fill(theme::BG_SURFACE)
                .inner_margin(egui::Margin::same(0.0)),
        )
        .show(ctx, |ui| {
            // ── Panel header ───────────────────────────────────────────────────
            panel_header(ui, "Media Browser");

            // ── Import button ──────────────────────────────────────────────────
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("+ Import Media").color(theme::ACCENT).size(13.0),
                        )
                        .min_size(egui::vec2(230.0, 30.0)),
                    )
                    .clicked()
                {
                    app.action_import_media();
                }
            });

            ui.add_space(8.0);
            ui.separator();

            // ── Asset list ─────────────────────────────────────────────────────
            let pool = app.media_pool();
            if pool.is_empty() {
                empty_state(ui, "No media imported.\nClick \"+ Import Media\" to begin.");
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for asset in pool {
                        ui.horizontal(|ui| {
                            ui.add_space(8.0);
                            // Phase 2: show thumbnail here.
                            let type_icon = match asset.asset_type {
                                vidcut_core::AssetType::Video => "🎬",
                                vidcut_core::AssetType::Audio => "🎵",
                                vidcut_core::AssetType::Image => "🖼",
                            };
                            ui.label(RichText::new(type_icon).size(16.0));
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(&asset.name)
                                        .color(theme::TEXT_PRIMARY)
                                        .size(12.0),
                                );
                                let dur = asset.duration_secs;
                                ui.label(
                                    RichText::new(format!(
                                        "{:02}:{:02}",
                                        (dur / 60.0) as u32,
                                        (dur % 60.0) as u32
                                    ))
                                    .color(theme::TEXT_MUTED)
                                    .size(11.0),
                                );
                            });
                        });
                        ui.add_space(4.0);
                    }
                });
            }
        });
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn panel_header(ui: &mut egui::Ui, title: &str) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(0x1a, 0x1a, 0x28))
        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new(title)
                    .color(theme::TEXT_MUTED)
                    .size(11.0)
                    .strong(),
            );
        });
    ui.separator();
}

fn empty_state(ui: &mut egui::Ui, message: &str) {
    ui.add_space(40.0);
    ui.vertical_centered(|ui| {
        ui.label(RichText::new("📁").size(40.0));
        ui.add_space(8.0);
        ui.label(
            RichText::new(message)
                .color(theme::TEXT_MUTED)
                .size(12.0),
        );
    });
}
