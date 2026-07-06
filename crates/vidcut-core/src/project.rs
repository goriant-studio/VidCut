//! Project — top-level container for a VidCut editing session.
//!
//! A [`Project`] holds all tracks, clips, and media assets. It can be
//! serialised to / deserialised from a `.vidcut` JSON file via
//! [`Project::save`] and [`Project::load`].

use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{MediaAsset, Timeline};

// ─── ProjectSettings ─────────────────────────────────────────────────────────

/// Output settings for a VidCut project (resolution, frame-rate, audio).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    /// Output frame-rate (frames per second). Default: 30.
    pub fps: u32,
    /// Output width in pixels. Default: 1920.
    pub width: u32,
    /// Output height in pixels. Default: 1080.
    pub height: u32,
    /// Audio sample rate in Hz. Default: 48 000.
    pub sample_rate: u32,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            fps: 30,
            width: 1920,
            height: 1080,
            sample_rate: 48_000,
        }
    }
}

// ─── Project ─────────────────────────────────────────────────────────────────

/// A VidCut project: the root of the editing session.
///
/// Serialised as JSON with a `.vidcut` extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Human-readable project name.
    pub name: String,
    /// Output / composition settings.
    pub settings: ProjectSettings,
    /// The master timeline containing all tracks and clips.
    pub timeline: Timeline,
    /// Media pool: all assets imported into this project.
    pub media_pool: Vec<MediaAsset>,
}

impl Project {
    /// Create a new empty project with the given name and default settings.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            settings: ProjectSettings::default(),
            timeline: Timeline::default(),
            media_pool: Vec::new(),
        }
    }

    /// Serialise the project to a `.vidcut` JSON file at `path`.
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Deserialise a project from a `.vidcut` JSON file at `path`.
    pub fn load(path: &Path) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let project: Self = serde_json::from_str(&json)?;
        Ok(project)
    }
}
