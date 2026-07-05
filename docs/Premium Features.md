# VidCut — Premium Features & Implementation Priority

## Overview

Dựa trên architecture hiện tại (C++20 + Qt6 + FFmpeg) và Phase 1 scaffold đã hoàn thành,
đây là danh sách các **premium features** và roadmap implementation.

---

## 🏆 Premium Features — Full List

### 🎬 Timeline & Editing
| Feature | Complexity |
|---------|---------------|------------|
| Magnetic Timeline (auto-ripple) | ⭐⭐⭐⭐⭐ |
| Multi-track non-linear editing | Primary/Secondary Storyline | ⭐⭐⭐⭐ |
| Blade / Razor cut tool | Blade Tool (B) | ⭐⭐ |
| Ripple, Roll, Slip, Slide trims | Trim tools | ⭐⭐⭐⭐ |
| Connected Clips (B-roll attach) | Connected Clips | ⭐⭐⭐⭐⭐ |
| Auditions (clip alternatives) | Auditions | ⭐⭐⭐⭐⭐ |
| Compound Clips (nested timeline) | Compound Clips | ⭐⭐⭐⭐ |
| Multicam Clip editing | Multicam Editing | ⭐⭐⭐⭐⭐ |
| Range-based selections | Range selection | ⭐⭐⭐ |
| Skimmer preview (hover scrub) | Skimmer | ⭐⭐⭐ |

### 🎨 Color & Effects
| Feature | FCP Equivalent | Complexity |
|---------|---------------|------------|
| Color Wheels (Lift/Gamma/Gain) | Color Board / Color Wheels | ⭐⭐⭐⭐ |
| Color Curves (RGB + Luma) | Color Curves | ⭐⭐⭐⭐ |
| Hue/Saturation Curves | Hue/Sat Curves | ⭐⭐⭐⭐ |
| LUT support (3D LUT import) | Custom LUT | ⭐⭐⭐ |
| Scopes (Waveform, Vectorscope, Histogram) | Video Scopes | ⭐⭐⭐⭐ |
| Color Match (AI auto-match) | Color Match | ⭐⭐⭐⭐⭐ |
| Keyframeable color grades | Per-clip keyframes | ⭐⭐⭐⭐ |
| Built-in video effects library | Effects Browser | ⭐⭐⭐⭐ |
| Motion Blur | Motion Blur | ⭐⭐⭐⭐ |
| Chroma Key (Green Screen) | Keyer effect | ⭐⭐⭐⭐⭐ |

### 🔊 Audio
| Feature | FCP Equivalent | Complexity |
|---------|---------------|------------|
| Waveform visualization on clips | Audio waveform | ⭐⭐ |
| Per-clip audio gain & pan | Audio inspector | ⭐⭐ |
| Multi-channel audio mixer | Audio Mixer | ⭐⭐⭐⭐ |
| Audio roles (Dialogue/Music/FX) | Roles | ⭐⭐⭐⭐ |
| Noise reduction | Noise Reduction effect | ⭐⭐⭐⭐⭐ |
| Loudness normalization | Loudness settings | ⭐⭐⭐ |
| Audio crossfades | Audio transitions | ⭐⭐ |
| Sync audio to video (waveform match) | Synchronize Clips | ⭐⭐⭐⭐⭐ |

### 📁 Media Management
| Feature | FCP Equivalent | Complexity |
|---------|---------------|------------|
| Media import with proxy workflow | Proxy Media | ⭐⭐⭐⭐ |
| Smart Collections (auto-organize) | Smart Collections | ⭐⭐⭐⭐ |
| Keywords & tagging | Keywords | ⭐⭐⭐ |
| Media relink (offline media) | Media relink | ⭐⭐⭐ |
| Thumbnail scrubbing in browser | Browser skimming | ⭐⭐⭐ |
| Favorites & Rejected marking | Rating system | ⭐⭐ |

### 🚀 Rendering & Export
| Feature | FCP Equivalent | Complexity |
|---------|---------------|------------|
| Background rendering | Background render | ⭐⭐⭐⭐ |
| Hardware-accelerated encoding (NVENC/QSV) | HW acceleration | ⭐⭐⭐⭐ |
| Export to multiple formats | Share menu | ⭐⭐⭐ |
| Chapter markers | Chapter markers | ⭐⭐ |
| Batch export | Batch share | ⭐⭐⭐ |
| High DPI / 4K / 8K support | High resolution | ⭐⭐⭐ |

### ✨ Titles & Motion Graphics
| Feature | FCP Equivalent | Complexity |
|---------|---------------|------------|
| Built-in title templates | Titles browser | ⭐⭐⭐⭐ |
| Animated lower thirds | Built-in generators | ⭐⭐⭐⭐ |
| Custom text animation keyframes | Text keyframing | ⭐⭐⭐⭐ |
| 3D text rendering | 3D text | ⭐⭐⭐⭐⭐ |

### 🤖 AI Features
| Feature | FCP Equivalent | Complexity |
|---------|---------------|------------|
| Scene detection (auto-split) | Scene detection | ⭐⭐⭐ |
| Auto transcription & subtitles | Captions (AI) | ⭐⭐⭐⭐ |
| Smart Conform (reframe for aspect ratio) | Smart Conform | ⭐⭐⭐⭐⭐ |
| Object tracker | Object tracker | ⭐⭐⭐⭐⭐ |
| Silence detection | Remove Silence | ⭐⭐⭐ |

---

## 🎯 Implementation Priority Roadmap

### Phase 2 — Core Editing (Bắt đầu ngay, ~2–3 tháng)
> **Goal**: Usable editor với basic editing workflow

#### Priority 1 — CRITICAL (làm trước tiên)
- [ ] **Media Import** — drag & drop MP4/MOV/MKV vào media browser
  - FFmpeg MediaDecoder thực sự (không phải stub)
  - Async thumbnail generation
- [ ] **Timeline Clip Placement** — kéo clip từ browser xuống timeline
  - Clip rendering trên timeline (colored blocks + thumbnail)
  - Snap-to-grid, snap-to-clip
- [ ] **Preview Playback** — play/pause/seek với QOpenGLWidget + FFmpeg decode
  - Frame-accurate seeking
  - Audio sync (QAudioSink)
- [ ] **Blade Tool** — cut clip tại vị trí playhead
  - Ripple delete sau khi cut
- [ ] **Basic Trim** — drag clip edges để trim in/out points
- [ ] **Undo/Redo** — Command pattern (CommandManager đã có stub)
- [ ] **Export** — FFmpeg encode ra MP4 (H.264 + AAC)

#### Priority 2 — IMPORTANT (~1 tháng tiếp)
- [ ] **Audio Waveform** — visualize trên clip trong timeline
- [ ] **Multi-track** — video tracks + audio tracks riêng biệt
- [ ] **Transitions** — Cut, Dissolve, Fade In/Out
- [ ] **Ripple Trim** — ripple edit mode
- [ ] **Skimmer** — hover preview trong media browser
- [ ] **Keyboard shortcuts** — JKL playback, I/O points, B blade...

### Phase 3 — Professional Features (~3–4 tháng)
> **Goal**: Compete với CapCut/DaVinci Resolve basic tier

#### Priority 3 — HIGH VALUE
- [ ] **Color Wheels** (Lift/Gamma/Gain) — Inspector panel
- [ ] **LUT Import** — drag .cube file, apply 3D LUT
- [ ] **Video Scopes** — Waveform + Histogram panel
- [ ] **Keyframing system** — animate any parameter over time
  - Bezier curve editor
- [ ] **Built-in effects library** — blur, sharpen, stabilize
- [ ] **Compound Clips** — nest timeline sequences
- [ ] **Audio Mixer** — multi-track gain/pan/mute/solo
- [ ] **Hardware encoding** — NVENC (NVIDIA) / QSV (Intel)
- [ ] **Proxy workflow** — import + auto-generate proxy files

#### Priority 4 — NICE TO HAVE
- [ ] **Chroma Key (Green Screen)** — color-based keyer
- [ ] **Title templates** — built-in lower thirds, end cards
- [ ] **Color Curves** — RGB + Luma curves editor
- [ ] **Smart Collections** — auto-organize by metadata
- [ ] **Background rendering** — render while editing

### Phase 4 — AI & Advanced (~3–6 tháng)
> **Goal**: Feature parity với Final Cut Pro

#### Priority 5 — ADVANCED
- [ ] **Auto Transcription** — Whisper.cpp integration → subtitles
- [ ] **Scene Detection** — FFmpeg scene filter → auto-split clips
- [ ] **Silence Removal** — detect & remove silence gaps
- [ ] **Multicam Editing** — sync multiple cameras by audio waveform
- [ ] **Object Tracker** — track subject, attach titles/masks
- [ ] **Smart Conform** — AI reframe 16:9 → 9:16 for Shorts/Reels
- [ ] **Color Match** — AI match color grade across clips
- [ ] **Magnetic Timeline** — auto-ripple like FCP

---

## 📊 Priority Summary Matrix

| Priority | Features | Why First | Est. Time |
|----------|---------|-----------|-----------|
| **P1** | Import, Timeline, Playback, Cut, Trim, Export | Core loop — unusable without these | 6–8 tuần |
| **P2** | Audio waveform, Multi-track, Transitions, Shortcuts | Makes it feel like a real editor | 4 tuần |
| **P3** | Color grading, Keyframes, Effects, HW encode | Premium feel, differentiator | 8–12 tuần |
| **P4** | Compound clips, Proxy, Mixer, Chroma Key | Pro workflow | 4–6 tuần |
| **P5** | AI features, Multicam, Magnetic timeline | WOW features, major differentiator | 3–6 tháng |

---

## 🔑 Recommended Starting Point (Ngay tuần tới)

Theo thứ tự implement:

1. **`MediaDecoder.cpp`** — FFmpeg decode thật: open file → read packets → decode frames
2. **`ThumbnailGenerator.cpp`** — async seek to 1s, extract frame, convert to QImage
3. **`MediaBrowserWidget.cpp`** — show imported files với thumbnail grid
4. **`TimelineEngine`** — Track + Clip placement data model
5. **`TimelineWidget.cpp`** — render clips as colored rectangles, handle drag & drop
6. **`PreviewWidget.cpp`** — OpenGL texture upload từ decoded YUV frames
7. **`ExportEncoder.cpp`** — FFmpeg mux/encode pipeline

> **Tip**: Implement theo đúng thứ tự này vì mỗi bước depend on bước trước.
> Khi playback hoạt động, mọi feature khác (trim, color, effects) đều build on top of nó.
