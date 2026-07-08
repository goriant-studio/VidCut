# VidCut

> **Professional Video Editor — built entirely in Rust**

[![Rust Version](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

VidCut is an open-source, native video editor inspired by Final Cut Pro.
Built 100% in Rust — no C++, no Qt, no Electron — delivering maximum performance,
memory safety, and a modern native feel via **egui + wgpu**.

Runs on **macOS** and **Windows**.

---

## Features

- 🖤 **Dark theme** — Final Cut Pro-inspired palette (#1a1a1f background, #6c7bff accent)
- 🖼️ **5-panel layout** — Toolbar · Media Browser · Inspector · Preview · Timeline
- 📐 **DPI-aware** — crisp rendering on HiDPI / Retina displays
- 🦀 **Pure Rust workspace** — `vidcut-app` / `vidcut-core` / `vidcut-media`
- 🔄 **Undo/Redo** — Command pattern with full history
- 💾 **Project serialisation** — `.vidcut` JSON format via `serde_json`
- 🎬 **Media import** — Drag video/audio files into the browser (MP4, MOV, MKV, WAV, MP3)
- ✂️ **Timeline editing** — Clip drag, trim, snap-to-grid, overlap detection
- ▶️ **Preview playback** — Real-time frame-accurate playback with J/K/L speed controls
- 🎨 **Inspector** — Clip properties, media info, transforms (position, scale, rotation, opacity)
- 📤 **Export** — MP4/MOV/MKV output via ffmpeg with quality presets and progress reporting

### Roadmap

| Phase | Feature | Status |
|-------|---------|--------|
| 1 | Scaffold & boilerplate | ✅ Done |
| 2 | FFmpeg import, timeline editing, preview playback, export | ✅ Done |
| 3 | Multi-track, transitions, color grading, audio mixer | 🔜 Planned |
| 4 | AI features, plugin system | 🔜 Planned |

---

## Tech Stack

| Crate | Purpose |
|-------|---------|
| `eframe` + `egui` | Immediate-mode GUI (wgpu backend) |
| `egui_extras` | Extra widgets |
| `ffmpeg-sidecar` | Managed ffmpeg CLI binary for decode/encode |
| `mp4` + `symphonia` | Pure-Rust media probing (no C deps) |
| `rfd` | Native OS file dialogs (no C deps) |
| `tokio` | Async runtime for decode/export tasks |
| `serde` + `serde_json` | Project file serialisation |
| `anyhow` + `thiserror` | Error handling |
| `parking_lot` | Fast Mutex/RwLock |
| `tracing` | Structured logging |
| `uuid` | Unique IDs for tracks, clips, assets |
| `windows` *(Windows only)* | Win32/COM APIs — dialogs, taskbar, DXGI |

---

## Prerequisites

- **Rust stable**: [rustup.rs](https://rustup.rs)

### macOS

- **Xcode Command Line Tools**: `xcode-select --install`
- **macOS 12+** (Apple Silicon or Intel)

### Windows

- **Visual Studio 2022** (or Build Tools) with the **Desktop C++ workload**
  - Provides `link.exe`, `rc.exe`, and the Windows SDK
- **Windows 10 / 11** (x86_64)

> **Note**: ffmpeg is managed automatically via `ffmpeg-sidecar` — no manual ffmpeg installation required.

---

## Building

```bash
# Check all crates
cargo check --workspace

# Run in development mode
cargo run -p vidcut-app

# Release build
cargo build --workspace --release
```

### Environment variables

| Variable | Purpose |
|----------|---------|
| `RUST_LOG` | Log filter, e.g. `RUST_LOG=debug cargo run -p vidcut-app` |
| `RUST_BACKTRACE` | Set to `1` for full backtraces on panic |

---

## Project Structure

```
vidcut/
├── Cargo.toml                  ← workspace root
├── crates/
│   ├── vidcut-app/             ← binary: UI + eframe entry point
│   ├── vidcut-core/            ← library: project model, timeline, commands
│   └── vidcut-media/           ← library: FFmpeg wrappers, probing, export
├── docs/                       ← design docs, todo, presentation
├── resources/
│   ├── icons/vidcut.ico        ← app icon
│   └── vidcut.manifest         ← Windows DPI + compat manifest
└── .github/workflows/          ← GitHub Actions CI
```

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for dev setup and coding guidelines.

---

## License

[MIT](LICENSE) © 2026 Goriant Studio
