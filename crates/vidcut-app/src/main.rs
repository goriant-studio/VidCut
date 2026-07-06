//! VidCut — entry point.
//!
//! Initialises logging, constructs the eframe window with a 1440×900 viewport,
//! and runs the [`VidCutApp`] event loop.

// Hide the console window in release builds on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::IconData;
use tracing_subscriber::EnvFilter;

mod app;
mod panels;

fn main() -> anyhow::Result<()> {
    // ── Logging ───────────────────────────────────────────────────────────────
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tracing::info!("VidCut starting");

    // ── Window options ────────────────────────────────────────────────────────
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("VidCut")
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1024.0, 600.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    // ── Run ───────────────────────────────────────────────────────────────────
    eframe::run_native(
        "VidCut",
        options,
        Box::new(|cc| Ok(Box::new(app::VidCutApp::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {e}"))?;

    Ok(())
}

/// Load the application icon.
///
/// Phase 1: returns a 1×1 transparent icon so the build never panics.
/// Phase 2: parse `resources/icons/vidcut.ico` bytes via the `image` crate
/// and return the real RGBA pixel data.
fn load_icon() -> IconData {
    // 1×1 transparent RGBA pixel — valid enough for eframe.
    IconData {
        rgba: vec![0, 0, 0, 0],
        width: 1,
        height: 1,
    }
}
