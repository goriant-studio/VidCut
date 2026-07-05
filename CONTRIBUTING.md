# Contributing to VidCut

Thank you for your interest in contributing!

## Code Style

- **C++20** standard throughout
- Format all code with `clang-format` (config in `.clang-format`)
- Run `clang-format -i <files>` before committing

## Branching

| Branch | Purpose |
|--------|---------|
| `main` | Stable, CI-passing code |
| `feature/<name>` | New features |
| `fix/<name>` | Bug fixes |

## Pull Request Process

1. Fork the repo and create your branch from `main`
2. Ensure the build passes (`cmake --build`)
3. Add a clear PR description explaining what and why
4. Link any related issues

## Module Ownership

| Module | Path | Description |
|--------|------|-------------|
| Core engine | `libs/libvidcut/` | Project, Timeline, Undo/Redo |
| Media layer | `src/media/` | FFmpeg wrappers |
| UI layer | `src/ui/` | Qt6 widgets |

## Reporting Issues

Use GitHub Issues. Include:
- OS version and build toolchain
- Steps to reproduce
- Expected vs actual behavior
