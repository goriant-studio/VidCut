//! Encoder — renders the timeline to an output video file.
//!
//! **Phase 1 stub.** Full implementation in Phase 2 via `ffmpeg-next`
//! (libx264 video + AAC audio encoding).

use std::path::Path;

use anyhow::Result;

/// Encodes an edited timeline to an MP4 / MOV output file.
///
/// Phase 2: wraps an `ffmpeg_next` format output context, video + audio
/// codec contexts, and a progress-reporting channel.
#[allow(dead_code)]
pub struct ExportEncoder {
    _private: (),
}

impl ExportEncoder {
    /// Begin an export session to `output_path`.
    ///
    /// # Phase 2
    /// Will open the output container, configure libx264 + AAC codecs,
    /// and start writing packets from the decoded frame stream.
    pub fn begin(_output_path: &Path) -> Result<Self> {
        todo!("Phase 2: open output container + configure codecs")
    }

    /// Write a decoded RGBA video frame to the output stream.
    ///
    /// # Phase 2
    /// Will convert RGBA → YUV420P via libswscale, encode with libx264,
    /// and mux into the container.
    pub fn write_video_frame(&mut self, _rgba: &[u8], _pts_secs: f64) -> Result<()> {
        todo!("Phase 2: encode video frame")
    }

    /// Write a block of f32 PCM audio samples to the output stream.
    ///
    /// # Phase 2
    /// Will resample to the target sample rate via libswresample and encode
    /// with the AAC codec.
    pub fn write_audio_samples(&mut self, _samples: &[f32]) -> Result<()> {
        todo!("Phase 2: encode audio samples")
    }

    /// Flush and close the output file.
    ///
    /// # Phase 2
    /// Will flush encoder buffers, write the trailer, and close the I/O context.
    pub fn finish(self) -> Result<()> {
        todo!("Phase 2: flush + close output file")
    }
}
