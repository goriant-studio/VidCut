//! `vidcut-media` — FFmpeg wrappers and media utilities for VidCut.
//!
//! **Phase 1**: This crate contains stub types only. All methods that require
//! the FFmpeg C library are marked with `todo!("Phase 2: …")`.
//!
//! **Phase 2**: Replace stubs with real implementations using `ffmpeg-next`.
//!
//! This crate is the *only* place in the workspace where `unsafe` FFI code
//! is permitted.

pub mod audio;
pub mod decoder;
pub mod encoder;
pub mod frame_cache;
pub mod thumbnail;
