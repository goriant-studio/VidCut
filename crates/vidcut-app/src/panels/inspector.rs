//! Inspector panel — right sidebar showing clip / asset properties.
//!
//! Rendered as an `egui::SidePanel::right` with a fixed width of 280 px.
//! Phase 1: displays a placeholder "no selection" state.
//! Phase 2: will show editable clip transforms, timing, and media info.

use eframe::egui::{self, RichText};

use crate::{app::VidCutApp, panels::theme};

/// Show the inspector panel. Called every frame from [`VidCutApp::update`].
pub fn show(ctx: &egui::Context, _app: &mut VidCutApp) {
    egui::SidePanel::right("inspector")
        .exact_width(280.0)
        .resizable(true)
        .frame(
            egui::Frame::none()
                .fill(theme::BG_SURFACE)
                .inner_margin(egui::Margin::same(0.0)),
        )
        .show(ctx, |ui| {
            // ── Panel header ───────────────────────────────────────────────────
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(0x1a, 0x1a, 0x28))
                .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("INSPECTOR")
                            .color(theme::TEXT_MUTED)
                            .size(11.0)
                            .strong(),
                    );
                });
            ui.separator();

            // ── No-selection placeholder ───────────────────────────────────────
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("🔍").size(40.0));
                ui.add_space(8.0);
                ui.label(
                    RichText::new("No clip selected.\nClick a clip on the\ntimeline to inspect it.")
                        .color(theme::TEXT_MUTED)
                        .size(12.0),
                );
            });

            // ── Phase 2 placeholder sections ───────────────────────────────────
            ui.add_space(24.0);
            ui.separator();
            section_header(ui, "Transform");
            placeholder_row(ui, "Position", "0, 0");
            placeholder_row(ui, "Scale", "100%");
            placeholder_row(ui, "Rotation", "0°");
            placeholder_row(ui, "Opacity", "100%");

            ui.add_space(8.0);
            ui.separator();
            section_header(ui, "Timing");
            placeholder_row(ui, "In Point", "00:00:00:00");
            placeholder_row(ui, "Out Point", "00:00:00:00");
            placeholder_row(ui, "Duration", "00:00:00:00");
            placeholder_row(ui, "Speed", "100%");
        });
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.horizontal(|ui| {
        ui.add_space(12.0);
        ui.label(
            RichText::new(title)
                .color(theme::TEXT_MUTED)
                .size(11.0)
                .strong(),
        );
    });
    ui.add_space(4.0);
}

fn placeholder_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.add_space(12.0);
        ui.label(
            RichText::new(label)
                .color(theme::TEXT_MUTED)
                .size(12.0),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(12.0);
            ui.label(
                RichText::new(value)
                    .color(egui::Color32::from_rgb(0x70, 0x78, 0xc0))
                    .size(12.0),
            );
        });
    });
}
