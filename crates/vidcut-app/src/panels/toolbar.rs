//! Toolbar panel — top bar with file controls and transport buttons.
//!
//! Rendered as an `egui::TopBottomPanel::top`. Contains:
//! - File actions: New, Open, Save, Export
//! - Transport: Undo, Redo, Play/Pause, Stop
//! - Project name display

use eframe::egui::{self, Color32, RichText};

use crate::{app::VidCutApp, panels::theme};

/// Show the toolbar panel. Called every frame from [`VidCutApp::update`].
pub fn show(ctx: &egui::Context, app: &mut VidCutApp) {
    egui::TopBottomPanel::top("toolbar")
        .exact_height(44.0)
        .frame(
            egui::Frame::none()
                .fill(theme::BG_DEEP)
                .inner_margin(egui::Margin::symmetric(12.0, 6.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                // ── Brand ──────────────────────────────────────────────────────
                ui.label(
                    RichText::new("✂  VidCut")
                        .color(theme::ACCENT)
                        .size(16.0)
                        .strong(),
                );

                ui.separator();

                // ── File actions ───────────────────────────────────────────────
                if icon_button(ui, "🗋", "New Project").clicked() {
                    app.action_new_project();
                }
                if icon_button(ui, "📂", "Open").clicked() {
                    app.action_open_project();
                }
                if icon_button(ui, "💾", "Save").clicked() {
                    app.action_save_project();
                }
                if icon_button(ui, "⬆", "Export").clicked() {
                    app.action_export();
                }

                ui.separator();

                // ── Undo / Redo ────────────────────────────────────────────────
                let can_undo = app.can_undo();
                let can_redo = app.can_redo();

                ui.add_enabled_ui(can_undo, |ui| {
                    if icon_button(ui, "↩", "Undo").clicked() {
                        app.action_undo();
                    }
                });
                ui.add_enabled_ui(can_redo, |ui| {
                    if icon_button(ui, "↪", "Redo").clicked() {
                        app.action_redo();
                    }
                });

                ui.separator();

                // ── Transport ──────────────────────────────────────────────────
                let play_label = if app.is_playing() { "⏸" } else { "▶" };
                if icon_button(ui, play_label, if app.is_playing() { "Pause" } else { "Play" })
                    .clicked()
                {
                    app.action_play_pause();
                }
                if icon_button(ui, "⏹", "Stop").clicked() {
                    app.action_stop();
                }

                // ── Playhead time display ──────────────────────────────────────
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let secs = app.playhead_secs();
                    let h = (secs / 3600.0) as u32;
                    let m = ((secs % 3600.0) / 60.0) as u32;
                    let s = (secs % 60.0) as u32;
                    let f = ((secs.fract()) * 30.0) as u32;
                    ui.label(
                        RichText::new(format!("{h:02}:{m:02}:{s:02}:{f:02}"))
                            .color(Color32::from_rgb(0xb0, 0xb8, 0xff))
                            .monospace()
                            .size(13.0),
                    );
                });
            });
        });
}

/// A small icon-only button with a tooltip.
fn icon_button(ui: &mut egui::Ui, icon: &str, tooltip: &str) -> egui::Response {
    ui.add(egui::Button::new(RichText::new(icon).size(16.0)).frame(false))
        .on_hover_text(tooltip)
}
