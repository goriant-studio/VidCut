//! Audio decoder — decodes audio streams and generates waveform data.
//!
//! **Phase 1 stub.** Phase 2 will implement real decoding using `ffmpeg-next`
//! and `libswresample` to resample audio to f32 PCM, then generate a compact
//! waveform representation for the timeline display.

use std::path::Path;

use anyhow::Result;

/// Decodes audio streams and produces waveform data.
///
/// Phase 2: wraps an `ffmpeg_next` audio codec context + `libswresample`
/// resampler to produce f32 PCM samples.
#[allow(dead_code)]
pub struct AudioDecoder {
    _private: (),
}

impl AudioDecoder {
    /// Open an audio (or video+audio) file for decoding.
    ///
    /// # Phase 2
    /// Will open the container with `ffmpeg-next`, locate the best audio
    /// stream, and initialise the resampler.
    pub fn open(_path: &Path) -> Result<Self> {
        todo!("Phase 2: ffmpeg-next open audio stream + swresample init")
    }

    /// Decode all audio samples from the file as interleaved f32 PCM.
    ///
    /// # Phase 2
    /// Will drain the decoder, resample to 48 kHz stereo f32, and collect
    /// all samples into the returned `Vec`.
    pub fn decode_all(&mut self) -> Result<Vec<f32>> {
        todo!("Phase 2: decode + resample all audio frames to f32 PCM")
    }

    /// Generate a compact waveform summary (peak amplitude per N samples).
    ///
    /// `bins` controls how many data points to produce (one per pixel width
    /// of the timeline clip block is a typical value).
    ///
    /// # Phase 2
    /// Will call [`decode_all`](Self::decode_all) then reduce to `bins`
    /// peak-amplitude values in `[0.0, 1.0]`.
    pub fn waveform_peaks(&mut self, _bins: usize) -> Result<Vec<f32>> {
        todo!("Phase 2: generate waveform peak data for timeline display")
    }
}
