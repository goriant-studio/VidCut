//! UI panels for the VidCut application.
//!
//! Each sub-module owns one panel of the 5-panel layout:
//! - [`toolbar`] — top bar with transport and file controls
//! - [`media_browser`] — left panel with the media pool
//! - [`inspector`] — right panel with clip properties
//! - [`preview`] — central panel with the video preview
//! - [`timeline`] — bottom panel with track rows
//! - [`theme`] — dark theme setup

pub mod export_dialog;
pub mod inspector;
pub mod media_browser;
pub mod preview;
pub mod theme;
pub mod timeline;
pub mod toolbar;
