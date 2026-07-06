# VidCut — TODO

> **Tech Stack**: Rust + egui/wgpu + ffmpeg-next + windows-rs  
> **Target**: x86_64-pc-windows-msvc  
> **Last updated**: 2026-07-06

---

## Phase 1 — Scaffold & Boilerplate
> **Goal**: GitHub-ready project. Window mở được, 5-panel layout hiển thị, dark theme đúng màu.  
> **Est. time**: 1–2 ngày

### 1.1 Workspace Setup
- [x] Tạo root `Cargo.toml` (workspace, resolver = "2", `[workspace.dependencies]`)
- [x] Tạo `.cargo/config.toml` (target msvc, `target-cpu=native`)
- [x] Tạo 3 crate skeleton: `vidcut-app`, `vidcut-core`, `vidcut-media`
- [x] Verify `cargo check --workspace` không lỗi cấu trúc

### 1.2 Core Data Structures (`vidcut-core`)
- [x] `project.rs` — `Project`, `ProjectSettings` (fps, width, height, sample_rate)
- [x] `timeline.rs` — `Timeline` (tracks, duration_secs)
- [x] `track.rs` — `Track`, `TrackType` enum (Video/Audio)
- [x] `clip.rs` — `Clip` (timeline_start/end, source_start/end, asset_id, track_id)
- [x] `media_asset.rs` — `MediaAsset`, `AssetType` enum (Video/Audio/Image)
- [x] `commands/mod.rs` — `Command` trait + `CommandHistory` (push/undo/redo)
- [x] `commands/add_clip.rs` — `AddClipCommand` impl
- [x] `commands/trim_clip.rs` — `TrimClipCommand` impl
- [x] Tất cả types: derive `Serialize`, `Deserialize`, `Debug`, `Clone`
- [x] `Project::save(&Path)` và `Project::load(&Path)` qua `serde_json`

### 1.3 Media Layer Stubs (`vidcut-media`)
- [x] `decoder.rs` — `MediaInfo` struct + `MediaDecoder` (stub, `todo!("Phase 2")`)
- [x] `encoder.rs` — `ExportEncoder` (stub)
- [x] `thumbnail.rs` — `ThumbnailGenerator` (stub)
- [x] `audio.rs` — `AudioDecoder` (stub)
- [x] `frame_cache.rs` — `FrameCache` (stub, LRU concept)
- [x] Tất cả stubs có `#[allow(dead_code)]` và `//!` doc comment

### 1.4 UI Layer (`vidcut-app`)
- [x] `panels/theme.rs` — `apply_dark_theme()` với FCP palette (#1a1a1f, #6c7bff)
- [x] `panels/toolbar.rs` — `TopBottomPanel::top`, buttons: New/Open/Save/Export/Play/Undo/Redo
- [x] `panels/media_browser.rs` — `SidePanel::left` 250px, placeholder asset list
- [x] `panels/inspector.rs` — `SidePanel::right` 280px, placeholder properties
- [x] `panels/preview.rs` — `CentralPanel` upper, dark bg + "Preview" label
- [x] `panels/timeline.rs` — `TopBottomPanel::bottom` 200px, placeholder track rows
- [x] `app.rs` — `VidCutApp` struct + `eframe::App` impl
- [x] `main.rs` — init tracing, NativeOptions 1440×900, `eframe::run_native`

### 1.5 Windows Native & Build
- [x] `resources/vidcut.manifest` — DPI PerMonitorV2, Win10/11 compat GUID
- [x] `resources/icons/vidcut.ico` — placeholder ICO file (1x1 transparent)
- [x] `build.rs` — `winres`: embed manifest + icon + ProductName/Copyright
- [x] `main.rs`: `#![windows_subsystem = "windows"]`

### 1.6 Config & CI
- [x] `rustfmt.toml` (edition 2021, max_width 100, imports_granularity Crate)
- [x] `.editorconfig` (utf-8, lf, 4 spaces, trim whitespace)
- [x] `.gitignore` (/target/, *.pdb, .env, .idea/)
- [x] `README.md` — badges (build, license, Rust version), features, build instructions
- [x] `LICENSE` — MIT 2026 Goriant Studio
- [x] `CONTRIBUTING.md` — dev setup, coding guidelines
- [x] `.github/workflows/windows-build.yml` — fmt → clippy → build → test → upload artifact

### 1.7 Phase 1 Verification
- [x] `cargo check --workspace` — 0 errors
- [x] `cargo clippy --all-targets` — 0 warnings
- [x] Window mở, title "VidCut"
- [x] 5-panel layout visible
- [x] Dark theme đúng màu
- [ ] Windows DPI aware (không mờ trên HiDPI)
- [ ] GitHub Actions CI pass

---

## Phase 2 — Core Editing Features
> **Goal**: Import media, kéo clip lên timeline, xem preview, export cơ bản.  
> **Est. time**: 9–12 tuần

### 2.1 FFmpeg Integration (`vidcut-media`)
- [ ] Setup FFmpeg via `vcpkg` cho Windows (x64-windows static)
- [ ] `decoder.rs` — implement `MediaDecoder::open()` (ffmpeg-next probe)
- [ ] `decoder.rs` — implement `MediaDecoder::decode_frame()` (seek + decode → RGBA)
- [ ] `audio.rs` — implement `AudioDecoder` (swresample → f32 PCM)
- [ ] `audio.rs` — waveform data generation cho timeline display
- [ ] `frame_cache.rs` — LRU cache (parking_lot RwLock, configurable capacity)
- [ ] `thumbnail.rs` — async thumbnail generation (tokio spawn_blocking)

### 2.2 Media Import & Browser
- [ ] Windows file dialog (`IFileOpenDialog` via windows-rs) — filter video/audio
- [ ] `MediaAsset` probe khi import (ffmpeg: width, height, fps, duration)
- [ ] Media browser panel — hiển thị danh sách assets với thumbnail
- [ ] Thumbnail async load + cache vào `egui::TextureHandle`
- [ ] Drag media từ browser → timeline

### 2.3 Timeline Engine (`vidcut-core`)
- [ ] `TimelineEngine` — add/remove/move clip logic
- [ ] Snap-to-grid (frame-accurate)
- [ ] Overlap detection & resolution
- [ ] Playhead position tracking
- [ ] `CommandHistory` fully working (undo/redo add, remove, move, trim)

### 2.4 Timeline Widget (`vidcut-app`)
- [ ] Custom egui widget: track rows + clip blocks
- [ ] Clip drag & drop (horizontal move, cross-track)
- [ ] Clip trim (drag left/right edge)
- [ ] Playhead scrubbing
- [ ] Zoom in/out (scroll wheel)
- [ ] Waveform display cho audio clips
- [ ] Thumbnail strip cho video clips

### 2.5 Preview Playback
- [ ] wgpu texture streaming từ decoded frames
- [ ] Real-time playback (tokio task decode → mpsc → UI thread)
- [ ] Play / Pause / Stop controls
- [ ] Frame-accurate seeking (click trên timeline)
- [ ] Playback speed control (0.25x, 0.5x, 1x, 2x)

### 2.6 Basic Export
- [ ] `encoder.rs` — implement `ExportEncoder` (ffmpeg-next: libx264 + AAC)
- [ ] Export dialog: chọn output path, format (MP4/MOV), quality preset
- [ ] Progress reporting qua `mpsc` channel → UI progress bar
- [ ] Taskbar progress (ITaskbarList3 via windows-rs)
- [ ] Cancel export

### 2.7 Inspector Panel
- [ ] Clip properties: name, start/end time, speed, opacity
- [ ] Media info: resolution, fps, codec, file size
- [ ] Basic clip transforms: position, scale, rotation (placeholder cho Phase 3)

### 2.8 Phase 2 Verification
- [ ] Import MP4/MOV/WAV/MP3 thành công
- [ ] Clip hiển thị trên timeline với thumbnail
- [ ] Drag, trim, undo/redo hoạt động đúng
- [ ] Preview playback mượt ≥ 30fps (1080p)
- [ ] Export MP4 ra file đúng thời lượng
- [ ] No memory leaks (frame cache bounded)

---

## Phase 3 — Advanced Editing
> **Goal**: Professional editing: multi-track, transitions, color grading, audio mixer.  
> **Est. time**: 3–4 tháng

### 3.1 Multi-track & Compositing
- [ ] Unlimited video/audio tracks
- [ ] Track header controls: mute, solo, lock, rename, reorder
- [ ] Video compositing: alpha blend, blend modes
- [ ] `RenderPipeline` — composite multiple tracks per frame (wgpu compute)
- [ ] Track grouping

### 3.2 Transitions & Effects
- [ ] Transition system: cut, dissolve, wipe, fade-in/out
- [ ] Transition widget trên timeline (draggable duration)
- [ ] Video filters: brightness, contrast, saturation, sharpen, blur
- [ ] Effect stack per clip (ordered, enable/disable)
- [ ] Keyframe animation cho effect parameters
- [ ] Keyframe editor trên timeline

### 3.3 Color Grading
- [ ] Waveform monitor (luma) + Vectorscope
- [ ] Color wheels (Lift/Gamma/Gain) — egui custom widget
- [ ] Curves editor (RGB + individual channels)
- [ ] LUT import/apply (.cube format)
- [ ] wgpu compute shader cho real-time color pipeline

### 3.4 Audio Mixer
- [ ] WASAPI output via windows-rs (IMMDeviceEnumerator)
- [ ] Per-track volume + pan controls
- [ ] Master volume
- [ ] Audio meters (VU meter, peak hold)
- [ ] Fade in/out per clip
- [ ] Audio sync / waveform alignment

### 3.5 Project Management
- [ ] New / Open / Save / Save As (`.vidcut` JSON format)
- [ ] Recent projects list
- [ ] Auto-save (mỗi 5 phút)
- [ ] Project settings dialog (resolution, fps, sample rate)
- [ ] Media relink (khi file bị move)

### 3.6 Performance & Polish
- [ ] GPU-accelerated decode (DXGI / D3D11VA hardware decode)
- [ ] Background render cache (pre-decode frames xung quanh playhead)
- [ ] Multi-threaded export (tokio parallelism)
- [ ] Proxy workflow (low-res proxy cho edit, full-res khi export)
- [ ] Memory usage HUD (debug overlay)

### 3.7 Phase 3 Verification
- [ ] 4+ video tracks compositing real-time
- [ ] Transitions render đúng
- [ ] Color wheels thay đổi output đúng
- [ ] Audio playback qua WASAPI không glitch
- [ ] Save/load project round-trip hoàn hảo
- [ ] Export 4K H.264 trong thời gian hợp lý

---

## Phase 4 — AI Features & Plugin System
> **Goal**: AI-powered editing, extensible plugin architecture.  
> **Est. time**: 3–5 tháng

### 4.1 AI Scene Detection
- [ ] Research: ONNX Runtime Rust bindings vs. tch-rs (PyTorch)
- [ ] Scene cut detection (histogram diff / ML model)
- [ ] Auto-split clip tại scene boundaries
- [ ] Shot type classification (wide / medium / close-up)
- [ ] Silence detection cho audio (auto-cut dead air)

### 4.2 AI Auto-Cut
- [ ] Highlight detection (fast motion, loud audio peaks)
- [ ] Smart reframe (face/subject tracking → auto crop for 9:16)
- [ ] Auto-transcription (Whisper via ONNX hoặc Windows Speech API)
- [ ] Caption generation từ transcript
- [ ] B-roll suggestion (match keyword → stock footage placeholder)

### 4.3 AI Upscaling & Enhancement
- [ ] Real-ESRGAN upscaling (ONNX Runtime, wgpu compute)
- [ ] Noise reduction (temporal denoising)
- [ ] Frame interpolation (RIFE / DAIN via ONNX)
- [ ] Background removal (rembg model)
- [ ] Face enhancement (GFPGAN)

### 4.4 Plugin System
- [ ] Plugin API thiết kế: traits `VideoEffect`, `AudioEffect`, `Importer`, `Exporter`
- [ ] Rust dylib plugins (`cdylib` crate type, dynamic loading)
- [ ] WASM plugin sandbox (wasmtime) cho third-party plugins an toàn
- [ ] Plugin manager UI: install, enable/disable, settings
- [ ] Plugin marketplace stub (local directory scan)
- [ ] Example plugin: `vidcut-plugin-lut` (LUT importer)
- [ ] Example plugin: `vidcut-plugin-text` (title generator)

### 4.5 Export Presets & Sharing
- [ ] Export preset library: YouTube 4K, Instagram Reels, TikTok, Twitter
- [ ] Custom preset save/load
- [ ] Batch export (multiple clips/projects)
- [ ] Chapter markers → YouTube description generator
- [ ] Direct upload placeholder (YouTube API / OAuth2 stub)

### 4.6 Accessibility & Internationalization
- [ ] i18n framework (fluent-rs hoặc rust-i18n)
- [ ] Vietnamese + English UI strings
- [ ] Keyboard shortcut customization
- [ ] Keyboard shortcut reference overlay (? key)
- [ ] High contrast mode

### 4.7 Phase 4 Verification
- [ ] Scene detection chạy trong < 2s cho 10 phút video
- [ ] WASM plugin sandbox load + execute an toàn
- [ ] Upscaling 1080p → 4K chạy trên GPU
- [ ] Export preset YouTube 4K ra file đúng spec
- [ ] i18n switch language không restart app

---

## Backlog / Nice-to-have

- [ ] Multicam editing (sync multiple camera angles)
- [ ] 360° video support
- [ ] HDR (HDR10, HLG) display + export
- [ ] Motion tracking (manual point tracking)
- [ ] Collaboration (project share via cloud storage stub)
- [ ] Dark/Light theme toggle
- [ ] Custom workspace layout save/restore
- [ ] Crash recovery (auto-backup trước khi crash)
- [ ] Telemetry opt-in (tracing → remote Jaeger/OpenTelemetry)

---

_Generated from [Professional Video Editor for Windows.md](./Professional%20Video%20Editor%20for%20Windows.md)_
