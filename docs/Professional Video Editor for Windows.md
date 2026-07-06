# VidCut — Professional Video Editor for Windows (Final Cut Pro Alternative)

**Tech Stack**: Rust + egui/wgpu + FFmpeg (via ffmpeg-next) + windows-rs
**Approach**: Build từ scratch, native Windows, 100% Rust — không C++, không Qt
**Phase 1 Goal**: GitHub-ready project scaffold + boilerplate

VidCut là một open-source, native Windows video editor lấy cảm hứng từ Final Cut Pro của Apple. Built hoàn toàn bằng Rust để đạt hiệu suất tối đa, memory safety, và zero-cost abstractions.

---

## Tại sao Rust?

| Tiêu chí | C++/Qt6 | Rust |
|---|---|---|
| Memory Safety | ❌ Manual (UB, dangling ptrs) | ✅ Compile-time guarantees |
| Concurrency | ⚠️ Error-prone | ✅ Fearless concurrency |
| Build system | CMake (phức tạp) | Cargo (đơn giản, powerful) |
| Windows native | Qt abstraction layer | `windows-rs` — direct Win32/COM |
| Package mgmt | vcpkg | crates.io (hàng trăm nghìn crates) |
| Binary size | Lớn (Qt runtime) | Nhỏ, self-contained |
| Performance | Tốt | Tương đương/nhanh hơn |

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     VidCut Architecture (Rust)               │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              UI Layer (egui + wgpu)                    │   │
│  │  - App struct (eframe::App)                            │   │
│  │  - TimelinePanel (custom egui widget)                  │   │
│  │  - PreviewPanel (wgpu texture render)                  │   │
│  │  - MediaBrowser (egui TreeView)                        │   │
│  │  - InspectorPanel (egui side panel)                    │   │
│  └───────────────────────┬──────────────────────────────┘   │
│                           │ Rust message passing (mpsc)       │
│  ┌────────────────────────▼─────────────────────────────┐   │
│  │           Core Engine (vidcut-core crate)              │   │
│  │  - ProjectManager (save/load .vidcut JSON)             │   │
│  │  - TimelineEngine (non-linear, non-destructive)        │   │
│  │  - ClipManager (media pool)                            │   │
│  │  - UndoRedo (Command Pattern)                          │   │
│  │  - RenderPipeline (frame compositor)                   │   │
│  └───────────────────────┬──────────────────────────────┘   │
│                           │ FFI / unsafe bindings             │
│  ┌────────────────────────▼─────────────────────────────┐   │
│  │           Media Layer (vidcut-media crate)             │   │
│  │  - MediaDecoder (ffmpeg-next: avcodec + avformat)      │   │
│  │  - FrameCache (LRU cache for decoded frames)           │   │
│  │  - ThumbnailGenerator (async Tokio tasks)              │   │
│  │  - ExportEncoder (render to output file)               │   │
│  │  - AudioDecoder (swresample + waveform gen)            │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │           Windows Native Layer (windows-rs)            │   │
│  │  - File dialogs (IFileOpenDialog)                      │   │
│  │  - Taskbar progress (ITaskbarList3)                    │   │
│  │  - System tray, notifications                          │   │
│  │  - Hardware acceleration (DXGI, D3D11)                 │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## Workspace Structure (Cargo Workspace)

```
vidcut/
├── Cargo.toml                    ← workspace root
├── Cargo.lock
├── .cargo/
│   └── config.toml               ← Windows target, linker flags
├── crates/
│   ├── vidcut-app/               ← binary: entry point + eframe UI
│   │   ├── Cargo.toml
│   │   ├── build.rs              ← embed manifest + icon + version info
│   │   └── src/
│   │       ├── main.rs
│   │       ├── app.rs            ← eframe::App impl
│   │       └── panels/
│   │           ├── mod.rs
│   │           ├── theme.rs      ← dark theme (FCP-inspired palette)
│   │           ├── toolbar.rs
│   │           ├── timeline.rs
│   │           ├── preview.rs
│   │           ├── media_browser.rs
│   │           └── inspector.rs
│   ├── vidcut-core/              ← lib: core engine (no UI, no FFmpeg)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── project.rs        ← Project + ProjectSettings
│   │       ├── timeline.rs       ← Timeline struct
│   │       ├── clip.rs           ← Clip (start, end, track_id, asset_id)
│   │       ├── track.rs          ← Track (video/audio)
│   │       ├── media_asset.rs    ← MediaAsset (path, duration, type)
│   │       └── commands/
│   │           ├── mod.rs        ← UndoRedo trait + CommandHistory
│   │           ├── add_clip.rs
│   │           └── trim_clip.rs
│   └── vidcut-media/             ← lib: FFmpeg wrappers (unsafe ok here)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── decoder.rs        ← MediaDecoder (probe + decode frames)
│           ├── encoder.rs        ← ExportEncoder (render to MP4/MOV)
│           ├── thumbnail.rs      ← async thumbnail generation
│           ├── audio.rs          ← AudioDecoder + waveform data
│           └── frame_cache.rs    ← LRU cache for decoded frames
├── resources/
│   ├── icons/
│   │   └── vidcut.ico            ← 256x256 app icon
│   └── vidcut.manifest           ← Windows app manifest (DPI PerMonitorV2)
├── README.md
├── LICENSE                       ← MIT
├── CONTRIBUTING.md
├── .github/
│   └── workflows/
│       └── windows-build.yml     ← CI: fmt + clippy + build + test
├── .gitignore
├── .editorconfig
└── rustfmt.toml
```

---

## Phase 1 — Scaffold & Boilerplate

### Checklist:
- [ ] Cargo workspace setup (3 crates)
- [ ] `vidcut-app`: eframe window với 5-panel layout
- [ ] `vidcut-core`: Project + Timeline + Clip + Track structs (full fields, no stubs)
- [ ] `vidcut-media`: FFmpeg decode wrapper stubs (todo! với comment Phase 2)
- [ ] Dark theme (egui Visuals — FCP-inspired palette)
- [ ] Windows manifest (DPI PerMonitorV2, Win10/11 compat)
- [ ] `build.rs` nhúng manifest + icon + version info vào binary
- [ ] README.md (professional, badges, build instructions)
- [ ] LICENSE (MIT 2026 Goriant Studio)
- [ ] CONTRIBUTING.md (dev setup + coding guidelines)
- [ ] GitHub Actions CI (Windows MSVC + Rust)
- [ ] .gitignore, .editorconfig, rustfmt.toml

---

## Proposed Changes (Full File Specs)

### Root Workspace

#### [NEW] `Cargo.toml`
```toml
[workspace]
resolver = "2"
members = [
    "crates/vidcut-app",
    "crates/vidcut-core",
    "crates/vidcut-media",
]

[workspace.dependencies]
# UI
eframe          = { version = "0.30", features = ["wgpu"] }
egui            = "0.30"
egui_extras     = "0.30"

# Async
tokio           = { version = "1", features = ["full"] }

# Media
ffmpeg-next     = "7"

# Windows native
windows         = { version = "0.58", features = [
    "Win32_UI_Shell",
    "Win32_UI_WindowsAndMessaging",
    "Win32_Graphics_Dxgi",
    "Win32_System_Com",
] }

# Serialization / utils
serde           = { version = "1", features = ["derive"] }
serde_json      = "1"
thiserror       = "1"
anyhow          = "1"
tracing         = "0.1"
tracing-subscriber = "0.3"
parking_lot     = "0.12"
uuid            = { version = "1", features = ["v4"] }
```

#### [NEW] `.cargo/config.toml`
```toml
[build]
target = "x86_64-pc-windows-msvc"

[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-cpu=native"]
```

---

### App Crate (UI)

#### [NEW] `crates/vidcut-app/Cargo.toml`
```toml
[package]
name = "vidcut-app"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "vidcut"
path = "src/main.rs"

[dependencies]
eframe.workspace = true
egui.workspace = true
egui_extras.workspace = true
tokio.workspace = true
vidcut-core = { path = "../vidcut-core" }
vidcut-media = { path = "../vidcut-media" }
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
windows.workspace = true

[build-dependencies]
winres = "0.1"

[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = true
```

#### [NEW] `crates/vidcut-app/src/main.rs`
```rust
#![windows_subsystem = "windows"]  // No console window on release

use eframe::NativeOptions;
use tracing_subscriber::EnvFilter;

mod app;
mod panels;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let options = NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("VidCut")
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1024.0, 600.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "VidCut",
        options,
        Box::new(|cc| Ok(Box::new(app::VidCutApp::new(cc)))),
    )?;

    Ok(())
}

fn load_icon() -> eframe::egui::IconData {
    // TODO Phase 1: parse ICO bytes → RGBA via image crate
    todo!("embed icon from resources/icons/vidcut.ico")
}
```

#### [NEW] `crates/vidcut-app/src/app.rs`
```rust
use eframe::egui;
use vidcut_core::Project;

pub struct VidCutApp {
    project: Option<Project>,
    show_media_browser: bool,
    show_inspector: bool,
}

impl VidCutApp {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        crate::panels::theme::apply_dark_theme(&cc.egui_ctx);
        Self {
            project: None,
            show_media_browser: true,
            show_inspector: true,
        }
    }
}

impl eframe::App for VidCutApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        crate::panels::toolbar::show(ctx, self);
        crate::panels::media_browser::show(ctx, self);
        crate::panels::inspector::show(ctx, self);
        crate::panels::preview::show(ctx, self);
        crate::panels::timeline::show(ctx, self);
    }
}
```

#### [NEW] `crates/vidcut-app/src/panels/theme.rs`
```rust
use eframe::egui::{self, Color32, Rounding, Stroke, Visuals};

/// Apply VidCut dark theme — Final Cut Pro inspired palette
pub fn apply_dark_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    // VidCut dark palette
    visuals.window_fill         = Color32::from_rgb(0x1a, 0x1a, 0x1f); // #1a1a1f
    visuals.panel_fill          = Color32::from_rgb(0x24, 0x24, 0x30); // #242430
    visuals.faint_bg_color      = Color32::from_rgb(0x1e, 0x1e, 0x2a); // #1e1e2a
    visuals.extreme_bg_color    = Color32::from_rgb(0x12, 0x12, 0x18); // #121218
    visuals.selection.bg_fill   = Color32::from_rgb(0x6c, 0x7b, 0xff); // #6c7bff accent
    visuals.hyperlink_color     = Color32::from_rgb(0x6c, 0x7b, 0xff);
    visuals.override_text_color = Some(Color32::from_rgb(0xe8, 0xe8, 0xf0));
    visuals.window_rounding     = Rounding::same(8.0);
    visuals.window_stroke       = Stroke::new(1.0, Color32::from_rgb(0x35, 0x35, 0x45));

    ctx.set_visuals(visuals);

    // Typography
    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
    );
    ctx.set_style(style);
}
```

#### Panels layout:
- **toolbar.rs**: `egui::TopBottomPanel::top` — buttons: New, Open, Save, Export, Play/Pause/Stop, Undo/Redo
- **media_browser.rs**: `egui::SidePanel::left` — width 250px — list media assets
- **inspector.rs**: `egui::SidePanel::right` — width 280px — clip properties
- **preview.rs**: `egui::CentralPanel` upper portion — wgpu texture area (placeholder dark bg + "Preview" label)
- **timeline.rs**: `egui::TopBottomPanel::bottom` — height 200px — track rows placeholder

---

### Core Engine Crate

#### [NEW] `crates/vidcut-core/Cargo.toml`
```toml
[package]
name = "vidcut-core"
version = "0.1.0"
edition = "2021"

[dependencies]
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
thiserror.workspace = true
uuid.workspace = true
```

#### Data Structures (full fields, all derive Serialize/Deserialize/Debug/Clone)

**`project.rs`**:
```rust
pub struct ProjectSettings {
    pub fps: u32,           // default: 30
    pub width: u32,         // default: 1920
    pub height: u32,        // default: 1080
    pub sample_rate: u32,   // default: 48000
}

pub struct Project {
    pub name: String,
    pub settings: ProjectSettings,
    pub timeline: Timeline,
    pub media_pool: Vec<MediaAsset>,
}
// impl: new(), save(&Path), load(&Path)
```

**`timeline.rs`**:
```rust
pub struct Timeline {
    pub tracks: Vec<Track>,
    pub duration_secs: f64,
}
```

**`track.rs`**:
```rust
pub enum TrackType { Video, Audio }

pub struct Track {
    pub id: Uuid,
    pub name: String,
    pub track_type: TrackType,
    pub clips: Vec<Clip>,
    pub muted: bool,
    pub locked: bool,
}
```

**`clip.rs`**:
```rust
pub struct Clip {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub track_id: Uuid,
    pub timeline_start: f64,  // secs on timeline
    pub timeline_end: f64,
    pub source_start: f64,    // secs in source file
    pub source_end: f64,
}
```

**`media_asset.rs`**:
```rust
pub enum AssetType { Video, Audio, Image }

pub struct MediaAsset {
    pub id: Uuid,
    pub path: PathBuf,
    pub name: String,
    pub duration_secs: f64,
    pub asset_type: AssetType,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<f64>,
}
```

**`commands/mod.rs`**:
```rust
pub trait Command {
    fn execute(&mut self, timeline: &mut Timeline);
    fn undo(&mut self, timeline: &mut Timeline);
}

pub struct CommandHistory {
    history: Vec<Box<dyn Command>>,
    cursor: usize,
}
// impl: push(), undo(), redo(), can_undo(), can_redo()
```

---

### Media Layer Crate

#### [NEW] `crates/vidcut-media/Cargo.toml`
```toml
[package]
name = "vidcut-media"
version = "0.1.0"
edition = "2021"

[dependencies]
ffmpeg-next.workspace = true
tokio.workspace = true
anyhow.workspace = true
thiserror.workspace = true
parking_lot.workspace = true
serde.workspace = true
```

#### Stubs (Phase 2 sẽ implement thật):

**`decoder.rs`**:
```rust
pub struct MediaInfo { pub width: u32, pub height: u32, pub fps: f64,
                       pub duration_secs: f64, pub has_audio: bool }
pub struct MediaDecoder { /* ffmpeg context */ }
impl MediaDecoder {
    pub fn open(path: &Path) -> Result<(Self, MediaInfo)> {
        todo!("Phase 2: ffmpeg-next probe + open")
    }
    pub fn decode_frame(&mut self, timestamp_secs: f64) -> Result<Vec<u8>> {
        todo!("Phase 2: seek + decode → RGBA bytes")
    }
}
```

**`encoder.rs`**, **`thumbnail.rs`**, **`audio.rs`**, **`frame_cache.rs`** — tương tự, struct + stub methods có `todo!("Phase 2: ...")`.

---

### Windows Manifest & Build

#### [NEW] `resources/vidcut.manifest`
```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity version="1.0.0.0" name="goriant.VidCut" type="win32"/>
  <description>VidCut — Professional Video Editor</description>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
    </windowsSettings>
  </application>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <!-- Windows 10/11 -->
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
    </application>
  </compatibility>
</assembly>
```

#### [NEW] `crates/vidcut-app/build.rs`
```rust
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_manifest_file("../../resources/vidcut.manifest");
        res.set_icon("../../resources/icons/vidcut.ico");
        res.set("ProductName", "VidCut");
        res.set("FileDescription", "VidCut — Professional Video Editor");
        res.set("LegalCopyright", "Copyright (c) 2026 Goriant Studio");
        res.compile().unwrap();
    }
}
```

---

### GitHub Actions CI

#### [NEW] `.github/workflows/windows-build.yml`
```yaml
name: Windows Build (Rust)

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  build:
    runs-on: windows-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-pc-windows-msvc
          components: clippy, rustfmt

      - name: Cache Cargo registry
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Install FFmpeg (via vcpkg)
        run: |
          vcpkg install ffmpeg:x64-windows
          echo "VCPKG_ROOT=$env:VCPKG_ROOT" >> $env:GITHUB_ENV

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Clippy lint
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Build (Debug)
        run: cargo build --workspace

      - name: Build (Release)
        run: cargo build --workspace --release

      - name: Run tests
        run: cargo test --workspace

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: vidcut-windows-x64
          path: target/release/vidcut.exe
```

---

### Config Files

#### [NEW] `rustfmt.toml`
```toml
edition = "2021"
max_width = 100
tab_spaces = 4
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
```

#### [NEW] `.editorconfig`
```ini
root = true

[*]
charset = utf-8
end_of_line = lf
indent_style = space
indent_size = 4
trim_trailing_whitespace = true
insert_final_newline = true

[*.toml]
indent_size = 4

[*.md]
trim_trailing_whitespace = false
```

#### [NEW] `.gitignore`
```
/target/
Cargo.lock
*.pdb
*.ilk
.env
.idea/
.vscode/settings.json
*.user
FFmpeg/
```

---

## Constraints (Bắt buộc tuân theo)

- ❌ **KHÔNG dùng C++, Qt, hoặc bất kỳ C++ framework nào**
- ❌ **KHÔNG dùng Tauri, Electron, hay web technology**
- ✅ Code phải pass `cargo check --workspace` sau khi setup FFmpeg
- ✅ Tất cả `todo!()` phải có comment giải thích Phase nào sẽ implement
- ✅ Dùng `#[allow(dead_code)]` cho stubs để tránh warnings làm CI fail
- ✅ Mỗi file phải có module-level doc comment (`//!`) giải thích purpose
- ✅ Tất cả public types phải có `///` doc comments

---

## Verification Plan

### Build Check
```powershell
# Prerequisites: Rust stable (msvc toolchain), Visual Studio 2022, FFmpeg via vcpkg
rustup target add x86_64-pc-windows-msvc
vcpkg install ffmpeg:x64-windows

# Check toàn bộ workspace
cargo check --workspace

# Run (dev)
cargo run -p vidcut-app

# Release build
cargo build --workspace --release
# Binary tại: target/release/vidcut.exe
```

### Sanity Checklist
- [ ] `cargo check --workspace` không có errors
- [ ] `cargo clippy` clean (0 warnings)
- [ ] Window mở, title "VidCut"
- [ ] 5-panel layout visible (dù còn empty)
- [ ] Dark theme áp dụng đúng màu (#1a1a1f background, #6c7bff accent)
- [ ] Windows DPI aware (không bị mờ trên HiDPI)
- [ ] GitHub Actions CI pass

---

## Future Phases (Reference)

| Phase | Feature | Est. Time |
|-------|---------|-----------|
| 2 | Import MP4/MOV via ffmpeg-next, Media Browser | 2–3 tuần |
| 2 | Basic Timeline + Clip drag (egui custom widget) | 3–4 tuần |
| 2 | Preview playback (wgpu texture streaming) | 2 tuần |
| 2 | Trim clips, basic export (FFmpeg encoder) | 2–3 tuần |
| 3 | Multi-track, transitions, effects | 1–2 tháng |
| 3 | Color grading UI (wgpu compute shaders) | 1 tháng |
| 3 | Audio mixer (WASAPI via windows-rs) | 1 tháng |
| 4 | AI features (scene detection, auto-cut) | 2–3 tháng |
| 4 | Plugin system (Rust dylib / WASM plugins) | 1–2 tháng |

---

## Key Crates Reference

| Crate | Mục đích |
|-------|---------|
| `eframe` + `egui` | Immediate mode GUI, cross-platform, wgpu backend |
| `egui_extras` | Extra widgets (table, date picker, image) |
| `ffmpeg-next` | Safe Rust bindings to FFmpeg C library |
| `windows` (windows-rs) | Win32/COM APIs: dialogs, taskbar, DXGI |
| `tokio` | Async runtime cho background decode/export tasks |
| `wgpu` | Modern GPU API (D3D12/Vulkan) cho preview render |
| `serde` + `serde_json` | Project file serialization (.vidcut format) |
| `anyhow` + `thiserror` | Error handling |
| `parking_lot` | Fast Mutex/RwLock cho shared media state |
| `tracing` | Structured logging + performance tracing |
| `winres` | Build-time: embed manifest, icon, version info |
| `uuid` | Unique IDs cho Track, Clip, MediaAsset |
