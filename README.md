# VidCut

![Build](https://github.com/goriant-studio/VidCut/actions/workflows/windows-build.yml/badge.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![C++20](https://img.shields.io/badge/C%2B%2B-20-blue.svg)
![Qt6](https://img.shields.io/badge/Qt-6-green.svg)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey.svg)

**A professional, open-source video editor for Windows — inspired by Final Cut Pro.**

Built with native C++20, Qt6, and FFmpeg for maximum performance and a premium editing experience.

---

## Features (Roadmap)

| Status | Feature |
|--------|---------|
| 🚧 | Project scaffold & architecture |
| 📋 | Import MP4/MOV/MKV media |
| 📋 | Non-linear timeline with drag & drop |
| 📋 | Real-time preview (OpenGL) |
| 📋 | Trim, cut, split clips |
| 📋 | Multi-track video + audio |
| 📋 | Transitions & effects |
| 📋 | Color grading |
| 📋 | Audio mixer |
| 📋 | Export (H.264/H.265/ProRes) |
| 📋 | AI-assisted features |

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Language | C++20 |
| UI Framework | Qt6 (Widgets + QML) |
| Video/Audio | FFmpeg (libavcodec, libavformat, libswscale) |
| Rendering | OpenGL via QOpenGLWidget |
| Logging | spdlog |
| Build | CMake 3.25+ + vcpkg |
| CI | GitHub Actions (Windows MSVC) |

## Building

### Prerequisites

- Windows 10/11
- [Visual Studio 2022](https://visualstudio.microsoft.com/) (Desktop C++ workload)
- [CMake 3.25+](https://cmake.org/)
- [vcpkg](https://github.com/microsoft/vcpkg)

### Steps

```powershell
# 1. Clone
git clone https://github.com/goriant-studio/VidCut.git
cd VidCut

# 2. Configure (vcpkg installs dependencies automatically)
cmake -B build -S . `
  -DCMAKE_TOOLCHAIN_FILE="$env:VCPKG_ROOT/scripts/buildsystems/vcpkg.cmake" `
  -DCMAKE_BUILD_TYPE=Release

# 3. Build
cmake --build build --config Release --parallel

# 4. Run
.\build\src\Release\VidCut.exe
```

## Architecture

```
VidCut/
├── src/
│   ├── main.cpp
│   ├── ui/              # Qt6 widgets (MainWindow, Timeline, Preview…)
│   └── media/           # FFmpeg wrappers (Decoder, Encoder, Thumbnails…)
├── libs/
│   └── libvidcut/       # Core engine (Timeline, Clip, Project, Undo/Redo)
└── resources/           # Icons, QSS theme, Windows resources
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT — see [LICENSE](LICENSE).
