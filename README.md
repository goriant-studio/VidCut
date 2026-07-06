# VidCut

> **Professional Video Editor for Windows — built entirely in Rust**

[![Windows Build](https://github.com/goriant-studio/VidCut/actions/workflows/windows-build.yml/badge.svg)](https://github.com/goriant-studio/VidCut/actions/workflows/windows-build.yml)
[![Rust Version](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

VidCut is an open-source, native Windows video editor inspired by Final Cut Pro.
Built 100% in Rust — no C++, no Qt, no Electron — delivering maximum performance,
memory safety, and a modern native feel via **egui + wgpu**.

---

## Features (Phase 1 — Scaffold)

- 🖤 **Dark theme** — Final Cut Pro-inspired palette (#1a1a1f background, #6c7bff accent)
- 🖼️ **5-panel layout** — Toolbar · Media Browser · Inspector · Preview · Timeline
- 📐 **Windows DPI-aware** — PerMonitorV2 manifest for crisp rendering on HiDPI displays
- 🦀 **Pure Rust workspace** — `vidcut-app` / `vidcut-core` / `vidcut-media`
- 🔄 **Undo/Redo** — Command pattern with full history
- 💾 **Project serialisation** — `.vidcut` JSON format via `serde_json`

### Roadmap

| Phase | Feature | Status |
|-------|---------|--------|
| 1 | Scaffold & boilerplate | ✅ **Done** |
| 2 | FFmpeg import, timeline editing, preview playback | 🔜 Planned |
| 3 | Multi-track, transitions, color grading, audio mixer | 🔜 Planned |
| 4 | AI features, plugin system | 🔜 Planned |

---

## Tech Stack

| Crate | Purpose |
|-------|---------|
| `eframe` + `egui` | Immediate mode GUI (wgpu backend) |
| `egui_extras` | Extra widgets |
| `ffmpeg-next` | FFmpeg bindings (Phase 2) |
| `windows-rs` | Win32/COM APIs — dialogs, taskbar, DXGI |
| `tokio` | Async runtime for decode/export tasks |
| `wgpu` | GPU rendering for preview |
| `serde` + `serde_json` | Project file serialisation |
| `anyhow` + `thiserror` | Error handling |
| `parking_lot` | Fast Mutex/RwLock |
| `tracing` | Structured logging |
| `winres` | Embed manifest + icon at build time |
| `uuid` | Unique IDs for tracks, clips, assets |

---

## Prerequisites

- **Rust stable** (MSVC toolchain): [rustup.rs](https://rustup.rs)
- **Visual Studio 2022** (or Build Tools) with the **Desktop C++ workload**
  - Provides `link.exe`, `rc.exe`, and the Windows SDK
- **Windows 10 / 11** (x86_64)

> **Phase 2 prerequisite**: `vcpkg install ffmpeg:x64-windows` for FFmpeg bindings.

---

## Building

```powershell
# Add the MSVC target (one-time)
rustup target add x86_64-pc-windows-msvc

# Check all crates
cargo check --workspace

# Run in development mode
cargo run -p vidcut-app

# Release build  →  target/release/vidcut.exe
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
│   └── vidcut-media/           ← library: FFmpeg wrappers (stubs in Phase 1)
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
