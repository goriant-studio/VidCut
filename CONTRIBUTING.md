# Contributing to VidCut

Thank you for considering contributing to VidCut! This document covers the dev
setup, code style, and contribution workflow.

---

## Development Setup

### 1. Prerequisites

- **Rust stable** (MSVC toolchain): [rustup.rs](https://rustup.rs)
- **Visual Studio 2022** with the **Desktop C++ workload**
  - Provides `link.exe`, `rc.exe` (for `winres`), and the Windows SDK
- **Windows 10 / 11** (x86_64)

```powershell
# Install Rust
winget install Rustlang.Rustup

# Add MSVC target
rustup target add x86_64-pc-windows-msvc

# Install clippy + rustfmt if not already present
rustup component add clippy rustfmt
```

### 2. Clone & Build

```powershell
git clone https://github.com/goriant-studio/VidCut.git
cd VidCut
cargo check --workspace
cargo run -p vidcut-app
```

### 3. Phase 2+ (FFmpeg)

```powershell
# Install vcpkg (one-time global setup)
git clone https://github.com/microsoft/vcpkg $env:USERPROFILE\vcpkg
& "$env:USERPROFILE\vcpkg\bootstrap-vcpkg.bat"
$env:VCPKG_ROOT = "$env:USERPROFILE\vcpkg"

# Install FFmpeg static libs
vcpkg install ffmpeg:x64-windows
```

---

## Code Style

- **Formatter**: `rustfmt` with settings in `rustfmt.toml` (max 100 columns)
- **Linter**: `clippy` — zero warnings policy (`-D warnings`)
- **Naming**: follow Rust API guidelines (snake_case functions, CamelCase types)
- **Doc comments**: every public item must have `///` docs; every module must
  have a `//!` module-level comment explaining its purpose
- **`todo!()`**: every `todo!()` must include a comment identifying which Phase
  will implement it, e.g. `todo!("Phase 2: ffmpeg-next open stream")`
- **`unsafe`**: only permitted in `vidcut-media`; every `unsafe` block must
  have a `// SAFETY:` comment

### Before You Push

```powershell
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
```

---

## Workspace Layout

| Crate | Purpose | May use `unsafe`? |
|-------|---------|-------------------|
| `vidcut-core` | Data model, undo/redo, serialisation | ❌ No |
| `vidcut-media` | FFmpeg wrappers, frame cache | ✅ Yes (FFI only) |
| `vidcut-app` | UI (egui panels), entry point | ❌ No |

**No C++, Qt, Tauri, Electron, or web technology is permitted.**

---

## Pull Request Checklist

- [ ] `cargo fmt --all` — no formatting changes
- [ ] `cargo clippy --all-targets -- -D warnings` — 0 warnings
- [ ] `cargo test --workspace` — all tests pass
- [ ] New public items have `///` doc comments
- [ ] New modules have `//!` module-level comments
- [ ] `todo!()` calls include Phase labels
- [ ] PR description explains *why* the change is needed

---

## Reporting Issues

Use [GitHub Issues](https://github.com/goriant-studio/VidCut/issues) with:
- OS version and GPU
- Steps to reproduce
- Relevant `RUST_LOG=debug` output

---

## License

By contributing you agree that your changes will be licensed under the
[MIT License](LICENSE).
