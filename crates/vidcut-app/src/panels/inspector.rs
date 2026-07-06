//! Inspector panel — right sidebar showing clip / asset properties.
//!
//! Phase 2: Shows real info for the selected clip or asset.
//! - If a clip is selected: timing, source asset info, file path.
//! - If an asset is selected in the browser: media metadata.
//! - Otherwise: empty state.

use eframe::egui::{self, Color32, RichText};

use crate::{app::VidCutApp, panels::theme};

/// Show the inspector panel. Called every frame from [`VidCutApp::update`].
pub fn show(ctx: &egui::Context, app: &mut VidCutApp) {
    egui::SidePanel::right("inspector")
        .exact_width(280.0)
        .resizable(true)
        .frame(
            egui::Frame::none()
                .fill(theme::BG_SURFACE)
                .inner_margin(egui::Margin::same(0.0)),
        )
        .show(ctx, |ui| {
            panel_header(ui, "INSPECTOR");

            // ── Determine what to display ──────────────────────────────────────
            // Priority: selected clip > selected asset in browser.
            let selected_clip = app.selected_clip_id.and_then(|clip_id| {
                app.project.as_ref()?.timeline.tracks.iter()
                    .flat_map(|t| t.clips.iter())
                    .find(|c| c.id == clip_id)
                    .cloned()
            });

            if let Some(clip) = selected_clip {
                // Find the backing asset.
                let asset = app.project.as_ref()
                    .and_then(|p| p.media_pool.iter().find(|a| a.id == clip.asset_id))
                    .cloned();

                show_clip_inspector(ui, &clip, asset.as_ref());
            } else if let Some(asset_id) = app.selected_asset_id {
                let asset = app.project.as_ref()
                    .and_then(|p| p.media_pool.iter().find(|a| a.id == asset_id))
                    .cloned();
                if let Some(asset) = asset {
                    show_asset_inspector(ui, &asset);
                } else {
                    empty_state(ui);
                }
            } else {
                empty_state(ui);
            }
        });
}

// ── Clip inspector ─────────────────────────────────────────────────────────────

fn show_clip_inspector(
    ui: &mut egui::Ui,
    clip: &vidcut_core::Clip,
    asset: Option<&vidcut_core::MediaAsset>,
) {
    section_header(ui, "Clip Timing");

    info_row(ui, "Timeline In",  &format_tc(clip.timeline_start));
    info_row(ui, "Timeline Out", &format_tc(clip.timeline_end));
    info_row(ui, "Duration",     &format_tc(clip.duration()));
    info_row(ui, "Source In",    &format_tc(clip.source_start));
    info_row(ui, "Source Out",   &format_tc(clip.source_end));

    if let Some(asset) = asset {
        ui.add_space(8.0);
        ui.separator();
        section_header(ui, "Source Media");
        info_row(ui, "Name", &asset.name);
        info_row(ui, "Type", &format!("{:?}", asset.asset_type));
        if asset.duration_secs > 0.0 {
            info_row(ui, "Full Duration", &format_tc(asset.duration_secs));
        }
        if let (Some(w), Some(h)) = (asset.width, asset.height) {
            info_row(ui, "Resolution", &format!("{w}×{h}"));
        }
        if let Some(fps) = asset.fps {
            info_row(ui, "FPS", &format!("{fps:.2}"));
        }
        ui.add_space(8.0);
        ui.separator();
        section_header(ui, "File");
        // Show path truncated to fit
        let path_str = asset.path.to_string_lossy();
        ui.horizontal_wrapped(|ui| {
            ui.add_space(12.0);
            ui.label(
                RichText::new(path_str.as_ref())
                    .color(Color32::from_rgb(0x60, 0x68, 0xa0))
                    .size(10.0),
            );
        });
    }

    ui.add_space(16.0);
    ui.separator();
    section_header(ui, "Transform");
    dimmed_row(ui, "Position", "0, 0");
    dimmed_row(ui, "Scale",    "100%");
    dimmed_row(ui, "Rotation", "0°");
    dimmed_row(ui, "Opacity",  "100%");
}

// ── Asset inspector ───────────────────────────────────────────────────────────

fn show_asset_inspector(ui: &mut egui::Ui, asset: &vidcut_core::MediaAsset) {
    section_header(ui, "Media Info");
    info_row(ui, "Name", &asset.name);
    info_row(ui, "Type", &format!("{:?}", asset.asset_type));
    if asset.duration_secs > 0.0 {
        info_row(ui, "Duration", &format_tc(asset.duration_secs));
    }
    if let (Some(w), Some(h)) = (asset.width, asset.height) {
        info_row(ui, "Resolution", &format!("{w}×{h}"));
    }
    if let Some(fps) = asset.fps {
        info_row(ui, "FPS", &format!("{fps:.3}"));
    }
    ui.add_space(8.0);
    ui.separator();
    section_header(ui, "File Path");
    ui.horizontal_wrapped(|ui| {
        ui.add_space(12.0);
        ui.label(
            RichText::new(asset.path.to_string_lossy().as_ref())
                .color(Color32::from_rgb(0x60, 0x68, 0xa0))
                .size(10.0),
        );
    });
}

// ── Empty state ───────────────────────────────────────────────────────────────

fn empty_state(ui: &mut egui::Ui) {
    ui.add_space(40.0);
    ui.vertical_centered(|ui| {
        ui.label(RichText::new("🔍").size(36.0));
        ui.add_space(8.0);
        ui.label(
            RichText::new("No selection.\n\nClick an asset in the\nMedia Browser, or click\na clip on the timeline.")
                .color(theme::TEXT_MUTED)
                .size(12.0),
        );
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

fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add_space(12.0);
        ui.label(
            RichText::new(title)
                .color(Color32::from_rgb(0x8a, 0x9a, 0xff))
                .size(11.0)
                .strong(),
        );
    });
    ui.add_space(4.0);
}

fn info_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.add_space(12.0);
        ui.label(RichText::new(label).color(theme::TEXT_MUTED).size(11.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(12.0);
            ui.label(RichText::new(value).color(theme::TEXT_PRIMARY).size(11.0));
        });
    });
}

fn dimmed_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.add_space(12.0);
        ui.label(RichText::new(label).color(theme::TEXT_MUTED).size(11.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(12.0);
            ui.label(
                RichText::new(value)
                    .color(Color32::from_rgb(0x50, 0x58, 0x80))
                    .size(11.0),
            );
        });
    });
}

fn format_tc(secs: f64) -> String {
    let h = (secs / 3600.0) as u32;
    let m = ((secs % 3600.0) / 60.0) as u32;
    let s = (secs % 60.0) as u32;
    let f = (secs.fract() * 30.0) as u32;
    format!("{h:02}:{m:02}:{s:02}:{f:02}")
}
