//! Media Browser panel — left sidebar showing the imported media pool.
//!
//! Phase 2: Lists assets with type icon, name, duration.
//! - Click → select asset (highlighted)
//! - Double-click → add to timeline
//! - "Add to Timeline" button when asset is selected

use eframe::egui::{self, Color32, RichText, Sense};

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

            // ── "Add to Timeline" (shown when an asset is selected) ────────────
            if let Some(sel_id) = app.selected_asset_id {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("▶ Add to Timeline")
                                    .color(Color32::from_rgb(0xff, 0xff, 0xff))
                                    .size(12.0),
                            )
                            .fill(theme::ACCENT)
                            .min_size(egui::vec2(230.0, 26.0)),
                        )
                        .clicked()
                    {
                        app.action_add_asset_to_timeline(sel_id);
                    }
                });
            }

            ui.add_space(8.0);
            ui.separator();

            // ── Asset list ─────────────────────────────────────────────────────
            let pool_snapshot: Vec<_> = app.media_pool().to_vec();
            let selected_id = app.selected_asset_id;

            if pool_snapshot.is_empty() {
                empty_state(ui, "No media imported.\nClick \"+ Import Media\" to begin.\n\nTip: double-click an asset\nto add it to the timeline.");
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for asset in &pool_snapshot {
                        let is_selected = selected_id == Some(asset.id);

                        // Compute row background.
                        let row_bg = if is_selected {
                            Color32::from_rgba_premultiplied(0x6c, 0x7b, 0xff, 0x40)
                        } else {
                            Color32::TRANSPARENT
                        };

                        let (row_rect, row_resp) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), 48.0),
                            Sense::click(),
                        );

                        // Draw background.
                        if ui.is_rect_visible(row_rect) {
                            ui.painter().rect_filled(row_rect, 4.0, row_bg);
                        }

                        // Render contents inside the allocated row.
                        let mut child_ui = ui.new_child(
                            egui::UiBuilder::new()
                                .max_rect(row_rect)
                                .layout(egui::Layout::left_to_right(egui::Align::Center)),
                        );
                        child_ui.add_space(8.0);

                        let type_icon = match asset.asset_type {
                            vidcut_core::AssetType::Video => "🎬",
                            vidcut_core::AssetType::Audio => "🎵",
                            vidcut_core::AssetType::Image => "🖼",
                        };
                        child_ui.label(RichText::new(type_icon).size(18.0));
                        child_ui.add_space(6.0);

                        child_ui.vertical(|ui| {
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new(&asset.name)
                                    .color(if is_selected { Color32::WHITE } else { theme::TEXT_PRIMARY })
                                    .size(12.0)
                                    .strong(),
                            );
                            let dur = asset.duration_secs;
                            let extra = if let (Some(w), Some(h)) = (asset.width, asset.height) {
                                format!("{:02}:{:02}  {}×{}", (dur / 60.0) as u32, (dur % 60.0) as u32, w, h)
                            } else {
                                format!("{:02}:{:02}", (dur / 60.0) as u32, (dur % 60.0) as u32)
                            };
                            ui.label(
                                RichText::new(extra)
                                    .color(theme::TEXT_MUTED)
                                    .size(10.0),
                            );
                        });

                        // Handle click / double-click.
                        let asset_id = asset.id;
                        if row_resp.double_clicked() {
                            app.selected_asset_id = Some(asset_id);
                            app.action_add_asset_to_timeline(asset_id);
                        } else if row_resp.clicked() {
                            app.selected_asset_id = Some(asset_id);
                        }

                        // Selected left accent bar.
                        if is_selected {
                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(row_rect.min, egui::vec2(3.0, 48.0)),
                                0.0,
                                theme::ACCENT,
                            );
                        }
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
