//! Export dialog — modal overlay for configuring and monitoring an export.
//!
//! Shows:
//! - Output path picker
//! - Format selector (MP4 / MOV / MKV)
//! - Quality preset (High / Medium / Small)
//! - Progress bar + status line during export
//! - Cancel button
//!
//! The dialog is shown as an `egui::Window` (modal-like, centred).
//! It reads/writes [`ExportDialogState`] from [`VidCutApp`].

use eframe::egui::{self, Color32, RichText};

use crate::{app::VidCutApp, panels::theme};
use vidcut_media::{ffmpeg_available, OutputFormat, QualityPreset};

// ─── show ────────────────────────────────────────────────────────────────────

/// Render the export dialog if it is open. Called every frame from
/// [`VidCutApp::update`].
pub fn show(ctx: &egui::Context, app: &mut VidCutApp) {
    if !app.export_dialog_open {
        return;
    }

    // Dim background (poor-man's modal overlay).
    let screen = ctx.screen_rect();
    ctx.layer_painter(egui::LayerId::new(
        egui::Order::Background,
        egui::Id::new("export_modal_bg"),
    ))
    .rect_filled(
        screen,
        0.0,
        Color32::from_rgba_premultiplied(0, 0, 0, 160),
    );

    let mut open = true;
    egui::Window::new("Export")
        .id(egui::Id::new("export_dialog"))
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .fixed_size([440.0, 320.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(egui::Color32::from_rgb(0x1a, 0x1a, 0x2a))
                .stroke(egui::Stroke::new(1.0, theme::BORDER)),
        )
        .show(ctx, |ui| {
            dialog_contents(ctx, ui, app);
        });

    // If the user clicked the ✕ of the window.
    if !open {
        app.action_cancel_export();
        app.export_dialog_open = false;
    }
}

// ─── Contents ────────────────────────────────────────────────────────────────

fn dialog_contents(ctx: &egui::Context, ui: &mut egui::Ui, app: &mut VidCutApp) {
    // Poll export progress from background thread.
    app.poll_export_progress(ctx);

    let is_exporting = app.export_progress.is_some();

    // ── Header ────────────────────────────────────────────────────────────────
    ui.add_space(4.0);
    ui.label(
        RichText::new("Export Video")
            .color(Color32::WHITE)
            .size(16.0)
            .strong(),
    );
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    // ── FFmpeg check ──────────────────────────────────────────────────────────
    if !ffmpeg_available() {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new("⚠  ffmpeg not found in PATH.")
                    .color(Color32::from_rgb(0xff, 0xcc, 0x44))
                    .size(12.0),
            );
        });
        ui.add_space(4.0);
        ui.label(
            RichText::new(
                "Install ffmpeg and ensure it is on your system PATH to enable export.",
            )
            .color(theme::TEXT_MUTED)
            .size(11.0),
        );
        ui.add_space(12.0);
        if ui
            .add(
                egui::Button::new(RichText::new("Close").size(13.0))
                    .min_size(egui::vec2(100.0, 28.0)),
            )
            .clicked()
        {
            app.export_dialog_open = false;
        }
        return;
    }

    // ── Output path ───────────────────────────────────────────────────────────
    ui.add_enabled_ui(!is_exporting, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Output").color(theme::TEXT_MUTED).size(12.0));
            ui.add_space(8.0);

            let path_str = app
                .export_output_path
                .as_deref()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| "No file selected".to_owned());

            ui.add(
                egui::TextEdit::singleline(&mut path_str.as_str())
                    .desired_width(280.0)
                    .interactive(false)
                    .text_color(if app.export_output_path.is_some() {
                        theme::TEXT_PRIMARY
                    } else {
                        theme::TEXT_MUTED
                    }),
            );

            if ui
                .add(
                    egui::Button::new(RichText::new("Browse…").size(12.0))
                        .min_size(egui::vec2(70.0, 24.0)),
                )
                .clicked()
            {
                let format = app.export_format;
                let default_name = format!("export.{}", format.extension());
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name(&default_name)
                    .add_filter(format.display_name(), &[format.extension()])
                    .set_title("Save Export As")
                    .save_file()
                {
                    app.export_output_path = Some(path);
                }
            }
        });

        ui.add_space(8.0);

        // ── Format ────────────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label(RichText::new("Format").color(theme::TEXT_MUTED).size(12.0));
            ui.add_space(8.0);
            egui::ComboBox::from_id_salt("export_format")
                .selected_text(app.export_format.display_name())
                .width(220.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut app.export_format,
                        OutputFormat::Mp4,
                        OutputFormat::Mp4.display_name(),
                    );
                    ui.selectable_value(
                        &mut app.export_format,
                        OutputFormat::Mov,
                        OutputFormat::Mov.display_name(),
                    );
                    ui.selectable_value(
                        &mut app.export_format,
                        OutputFormat::Mkv,
                        OutputFormat::Mkv.display_name(),
                    );
                });
        });

        ui.add_space(8.0);

        // ── Quality ───────────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label(RichText::new("Quality").color(theme::TEXT_MUTED).size(12.0));
            ui.add_space(8.0);
            egui::ComboBox::from_id_salt("export_quality")
                .selected_text(app.export_quality.display_name())
                .width(220.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut app.export_quality,
                        QualityPreset::High,
                        QualityPreset::High.display_name(),
                    );
                    ui.selectable_value(
                        &mut app.export_quality,
                        QualityPreset::Medium,
                        QualityPreset::Medium.display_name(),
                    );
                    ui.selectable_value(
                        &mut app.export_quality,
                        QualityPreset::Small,
                        QualityPreset::Small.display_name(),
                    );
                });
        });
    });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);

    // ── Progress area ─────────────────────────────────────────────────────────
    if let Some((fraction, msg)) = &app.export_progress {
        let fraction = *fraction;
        ui.label(
            RichText::new("Exporting…")
                .color(theme::ACCENT)
                .size(12.0)
                .strong(),
        );
        ui.add_space(6.0);
        ui.add(
            egui::ProgressBar::new(fraction)
                .show_percentage()
                .desired_width(ui.available_width()),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new(msg.as_str())
                .color(theme::TEXT_MUTED)
                .size(10.0)
                .monospace(),
        );
        ui.add_space(8.0);
        if ui
            .add(
                egui::Button::new(
                    RichText::new("Cancel")
                        .color(Color32::from_rgb(0xff, 0x55, 0x55))
                        .size(13.0),
                )
                .min_size(egui::vec2(100.0, 28.0)),
            )
            .clicked()
        {
            app.action_cancel_export();
        }
        return;
    }

    if let Some(status) = &app.export_status.clone() {
        let (color, label) = if status.starts_with("✓") {
            (Color32::from_rgb(0x44, 0xdd, 0x88), status.as_str())
        } else if status.starts_with("✗") || status.starts_with("⚠") {
            (Color32::from_rgb(0xff, 0x55, 0x55), status.as_str())
        } else {
            (theme::TEXT_MUTED, status.as_str())
        };
        ui.label(RichText::new(label).color(color).size(12.0));
        ui.add_space(8.0);
    }

    // ── Action buttons ────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        let can_start = app.export_output_path.is_some()
            && app.project.as_ref().map(|p| !p.timeline.tracks.is_empty()).unwrap_or(false);

        ui.add_enabled_ui(can_start, |ui| {
            if ui
                .add(
                    egui::Button::new(
                        RichText::new("⬆  Export")
                            .color(Color32::WHITE)
                            .size(13.0)
                            .strong(),
                    )
                    .fill(theme::ACCENT)
                    .min_size(egui::vec2(110.0, 30.0)),
                )
                .clicked()
            {
                app.action_start_export();
            }
        });

        ui.add_space(8.0);

        if ui
            .add(
                egui::Button::new(RichText::new("Close").size(13.0))
                    .min_size(egui::vec2(80.0, 30.0)),
            )
            .clicked()
        {
            app.export_dialog_open = false;
        }

        if app.export_output_path.is_none() {
            ui.add_space(8.0);
            ui.label(
                RichText::new("← Select an output file to enable export")
                    .color(theme::TEXT_MUTED)
                    .size(11.0),
            );
        }
    });
}
