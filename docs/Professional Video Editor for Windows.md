# VidCut — Professional Video Editor for Windows (Final Cut Pro Alternative)

**Tech Stack**: Native C++ 20 + Qt6 + FFmpeg  
**Approach**: Build from scratch, clean architecture  
**Phase 1 Goal**: GitHub-ready project scaffold + boilerplate

VidCut là một open-source, native Windows video editor lấy cảm hứng từ Final Cut Pro của Apple. Built với C++ và Qt6 để đạt hiệu suất tối đa.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                     VidCut Architecture                   │
│                                                           │
│  ┌──────────────────────────────────────────────────┐   │
│  │              UI Layer (Qt6 + QML)                  │   │
│  │  - MainWindow (Qt Widgets / QML)                   │   │
│  │  - Timeline Widget (custom QWidget)                │   │
│  │  - Preview Widget (QOpenGLWidget / QVideoWidget)   │   │
│  │  - Media Browser (QTreeView)                       │   │
│  │  - Inspector Panel (QDockWidget)                   │   │
│  └───────────────────┬──────────────────────────────┘   │
│                       │ Qt Signals/Slots                   │
│  ┌────────────────────▼─────────────────────────────┐   │
│  │           Core Engine (libvidcut)                  │   │
│  │  - Project Manager (save/load .vidcut XML)         │   │
│  │  - Timeline Engine (non-linear, non-destructive)   │   │
│  │  - Clip Manager (media pool)                       │   │
│  │  - Undo/Redo (Command Pattern)                     │   │
│  │  - Render Pipeline (frame compositor)              │   │
│  └───────────────────┬──────────────────────────────┘   │
│                       │ C API calls                       │
│  ┌────────────────────▼─────────────────────────────┐   │
│  │           Media Layer (FFmpeg wrappers)            │   │
│  │  - MediaDecoder (libavcodec + libavformat)         │   │
│  │  - FrameCache (LRU cache for decoded frames)       │   │
│  │  - ThumbnailGenerator (async frame extraction)     │   │
│  │  - ExportEncoder (render to output file)           │   │
│  │  - AudioDecoder (libswresample + waveform gen)     │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

---

## Phase 1 — Scaffold & Boilerplate

### What we're building NOW:
- ✅ Complete project directory structure
- ✅ CMakeLists.txt (root + subdirs)
- ✅ Qt6 MainWindow skeleton (compiles and runs)
- ✅ Placeholder UI zones (5 panels)
- ✅ FFmpeg integration layer stubs
- ✅ Core engine stubs (libvidcut)
- ✅ README.md (professional, with badges)
- ✅ LICENSE (MIT)
- ✅ CONTRIBUTING.md
- ✅ GitHub Actions CI (Windows MSVC build)
- ✅ .gitignore, .clang-format, .editorconfig
- ✅ vcpkg.json (dependency manifest)

---

## Proposed Changes

### Root Project
#### [NEW] `vidcut/CMakeLists.txt`
- CMake 3.25+ minimum
- C++20 standard
- Qt6 package find (Widgets, OpenGL, Multimedia)
- FFmpeg via vcpkg
- Subdirectories: `src/`, `libs/libvidcut/`

#### [NEW] `vidcut/vcpkg.json`
Dependency manifest:
- `ffmpeg` (avcodec, avformat, avfilter, swscale, swresample)
- `qt6-base`, `qt6-multimedia`
- `spdlog` (logging)
- `nlohmann-json` (project file serialization)

---

### Core Engine Library
#### [NEW] `vidcut/libs/libvidcut/`
- `CMakeLists.txt`
- `include/vidcut/` — public headers
  - `Project.h` — project data model
  - `Timeline.h` — timeline engine
  - `Clip.h` — clip data model
  - `Track.h` — track data model
  - `MediaAsset.h` — media pool item
  - `CommandManager.h` — undo/redo
- `src/` — implementation stubs

#### [NEW] `vidcut/libs/libvidcut/include/vidcut/Project.h`
```cpp
namespace VidCut {
  struct ProjectSettings { int fps; int width; int height; };
  class Project {
  public:
    static Project create(const QString& name, ProjectSettings settings);
    bool save(const QString& path) const;
    static Project load(const QString& path);
    QString name() const;
    Timeline& timeline();
    MediaPool& mediaPool();
  };
}
```

---

### Media Layer
#### [NEW] `vidcut/src/media/`
- `MediaDecoder.h/.cpp` — FFmpeg decode wrapper (stub)
- `ThumbnailGenerator.h/.cpp` — async frame extraction (stub)
- `ExportEncoder.h/.cpp` — render pipeline (stub)
- `AudioDecoder.h/.cpp` — audio decode + waveform (stub)

---

### UI Layer
#### [NEW] `vidcut/src/ui/`
- `MainWindow.h/.cpp` — QMainWindow with docks
- `TimelineWidget.h/.cpp` — custom QWidget (stub)
- `PreviewWidget.h/.cpp` — QOpenGLWidget (stub)
- `MediaBrowserWidget.h/.cpp` — QTreeView (stub)
- `InspectorWidget.h/.cpp` — QDockWidget (stub)
- `ToolBar.h/.cpp` — main toolbar

#### [NEW] `vidcut/src/ui/MainWindow.cpp`
Layout skeleton:
```
┌─────────────────────────────────────────┐
│  Toolbar                                 │
├─────────────────┬───────────────────────┤
│  Media Browser  │  Preview Player        │
│  (QDockWidget)  │  (QOpenGLWidget)       │
├─────────────────┴───────────────────────┤
│  Timeline (custom QWidget, scrollable)   │
├──────────────────────────────────────────┤
│  Inspector (QDockWidget, right side)     │
└──────────────────────────────────────────┘
```

---

### Style / Theme
#### [NEW] `vidcut/src/ui/styles/dark_theme.qss`
Qt Stylesheet — Final Cut Pro inspired dark theme:
- Background: `#1a1a1f`
- Surface: `#242430`
- Accent: `#6c7bff` (indigo)
- Text primary: `#e8e8f0`
- Timeline track bg: `#1e1e2a`

---

### Resources
#### [NEW] `vidcut/resources/`
- `vidcut.rc` — Windows resource file (app icon, version info)
- `icons/` — app icon (SVG + ICO)
- `app.qrc` — Qt resource file

---

### GitHub / Open Source
#### [NEW] `vidcut/README.md`
Professional README với:
- Project logo + tagline
- Feature list (roadmap-based)
- Screenshots placeholder → sẽ add sau khi UI ready
- Build instructions (Windows với vcpkg + CMake)
- Tech stack badges

#### [NEW] `vidcut/.github/workflows/windows-build.yml`
GitHub Actions:
- Trigger: push + PR to main
- Runner: `windows-latest`
- Steps: vcpkg install → CMake configure → CMake build
- Cache vcpkg for fast builds

#### [NEW] `vidcut/CONTRIBUTING.md`
- Code style (clang-format config)
- Branching strategy (main + feature branches)
- Issue labels, PR process
- Module ownership guide

#### [NEW] `vidcut/LICENSE` — MIT License

#### [NEW] `vidcut/.clang-format` — Google style base, custom adjustments

#### [NEW] `vidcut/.editorconfig` — UTF-8, LF, 4-space indent for C++

#### [NEW] `vidcut/.gitignore` — C++/Qt/CMake/Windows ignores

---

## Verification Plan

### Build Check
```bash
# Prerequisites: Visual Studio 2022, CMake, vcpkg
cmake -B build -S . -DCMAKE_TOOLCHAIN_FILE="%VCPKG_ROOT%/scripts/buildsystems/vcpkg.cmake"
cmake --build build --config Release
```

### Sanity Check
- [ ] CMake configures without errors
- [ ] Qt6 MainWindow compiles and launches
- [ ] 5-zone layout visible (even if empty)
- [ ] Dark theme applied
- [ ] Window title shows "VidCut"
- [ ] GitHub Actions CI passes

---

## Future Phases (Reference)

| Phase | Feature | Est. Time |
|-------|---------|-----------|
| 2 | Import MP4/MOV, Media Browser | 2–3 tuần |
| 2 | Basic Timeline + Clip drag | 3–4 tuần |
| 2 | Preview playback (QVideoWidget) | 2 tuần |
| 2 | Trim clips, basic export | 2–3 tuần |
| 3 | Multi-track, transitions | 1–2 tháng |
| 3 | Color grading UI | 1 tháng |
| 3 | Audio mixer | 1 tháng |
| 4 | AI features, multicam | 2–3 tháng |
| 4 | Plugin system | 1–2 tháng |
