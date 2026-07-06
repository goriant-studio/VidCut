//! Theme — VidCut dark theme, Final Cut Pro-inspired colour palette.
//!
//! Call [`apply_dark_theme`] once during app initialisation (inside
//! `VidCutApp::new`) to set the global egui [`Visuals`] and typography.

use eframe::egui::{self, Color32, FontId, Rounding, Stroke, Visuals};

// ─── Palette constants ────────────────────────────────────────────────────────

// Allow unused palette constants — defined as the complete VidCut design system
// for future panels and widgets.

/// Deepest background — window chrome.
pub const BG_DEEP: Color32 = Color32::from_rgb(0x12, 0x12, 0x18);
/// Primary panel background.
pub const BG_PANEL: Color32 = Color32::from_rgb(0x1a, 0x1a, 0x1f);
/// Secondary surface (cards, timeline track rows).
pub const BG_SURFACE: Color32 = Color32::from_rgb(0x24, 0x24, 0x30);
/// Subtle tint for alternating rows / faint areas.
pub const BG_FAINT: Color32 = Color32::from_rgb(0x1e, 0x1e, 0x2a);

/// Primary accent — indigo (#6c7bff).
pub const ACCENT: Color32 = Color32::from_rgb(0x6c, 0x7b, 0xff);
/// Accent at reduced opacity for hover states.
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0x85, 0x92, 0xff);
/// Danger / destructive action colour.
#[allow(dead_code)]
pub const DANGER: Color32 = Color32::from_rgb(0xff, 0x5c, 0x5c);

/// Primary text colour.
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xe8, 0xe8, 0xf0);
/// Muted / secondary text colour.
pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x8a, 0x8a, 0xa0);

/// Panel border / separator colour.
pub const BORDER: Color32 = Color32::from_rgb(0x35, 0x35, 0x45);

// ─── apply_dark_theme ─────────────────────────────────────────────────────────

/// Apply the VidCut dark theme to the given egui context.
///
/// Must be called once in [`VidCutApp::new`] before the first frame.
pub fn apply_dark_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    visuals.window_fill = BG_PANEL;
    visuals.panel_fill = BG_PANEL;
    visuals.faint_bg_color = BG_FAINT;
    visuals.extreme_bg_color = BG_DEEP;

    visuals.selection.bg_fill = ACCENT;
    visuals.selection.stroke = Stroke::new(1.0, ACCENT_HOVER);

    visuals.hyperlink_color = ACCENT;
    visuals.override_text_color = Some(TEXT_PRIMARY);

    visuals.window_rounding = Rounding::same(8.0);
    visuals.window_stroke = Stroke::new(1.0, BORDER);

    visuals.widgets.noninteractive.bg_fill = BG_SURFACE;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_MUTED);

    visuals.widgets.inactive.bg_fill = BG_SURFACE;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);

    visuals.widgets.hovered.bg_fill = ACCENT_HOVER;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, TEXT_PRIMARY);
    visuals.widgets.hovered.expansion = 1.0;

    visuals.widgets.active.bg_fill = ACCENT;
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::WHITE);

    ctx.set_visuals(visuals);

    // ── Typography ────────────────────────────────────────────────────────────
    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Body,
        FontId::new(14.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        FontId::new(13.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        FontId::new(11.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        FontId::new(18.0, egui::FontFamily::Proportional),
    );

    // Comfortable spacing between widgets.
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);

    ctx.set_style(style);
}
