//! Encoder — renders the timeline to an output video file.
//!
//! **Phase 2**: Drives the system `ffmpeg` CLI executable to mux + transcode
//! clips from their source files. No C FFmpeg bindings are required at this
//! phase; the encoder builds a `filter_complex` concat graph from the clip
//! list and spawns `ffmpeg` as a child process.
//!
//! **Supported formats**: MP4 (H.264 + AAC), MOV (H.264 + AAC), MKV (H.264 + Vorbis).
//!
//! **Phase 3** will replace this with a full in-process `ffmpeg-next` pipeline
//! for per-frame colour grading, transitions, and GPU acceleration.

use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::mpsc,
    thread,
};

use anyhow::{bail, Context, Result};
use tracing::{debug, info, warn};

// ─── Public types ─────────────────────────────────────────────────────────────

/// A clip segment to be included in the export.
///
/// Created by [`ExportJob`] from the project timeline.
#[derive(Debug, Clone)]
pub struct ExportSegment {
    /// Absolute path to the source media file.
    pub source_path: PathBuf,
    /// In-point within the source file, in seconds.
    pub source_start: f64,
    /// Duration of the segment, in seconds.
    pub duration: f64,
    /// Whether this segment has a video stream.
    pub has_video: bool,
    /// Whether this segment has an audio stream.
    pub has_audio: bool,
}

/// Output container format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Mp4,
    Mov,
    Mkv,
}

impl OutputFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Mov => "mov",
            Self::Mkv => "mkv",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Mp4 => "MP4 (H.264 + AAC)",
            Self::Mov => "MOV (H.264 + AAC)",
            Self::Mkv => "MKV (H.264 + Vorbis)",
        }
    }

    fn audio_codec(self) -> &'static str {
        match self {
            Self::Mp4 | Self::Mov => "aac",
            Self::Mkv => "libvorbis",
        }
    }
}

/// Quality preset mapped to a CRF value for libx264.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityPreset {
    /// High quality — CRF 18.
    High,
    /// Medium quality — CRF 23 (ffmpeg default).
    Medium,
    /// Small file size — CRF 28.
    Small,
}

impl QualityPreset {
    pub fn crf(self) -> u32 {
        match self {
            Self::High => 18,
            Self::Medium => 23,
            Self::Small => 28,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::High => "High quality (CRF 18)",
            Self::Medium => "Medium (CRF 23)",
            Self::Small => "Small file (CRF 28)",
        }
    }
}

/// A progress update sent from the encoder thread to the UI thread.
#[derive(Debug, Clone)]
pub enum ExportProgress {
    /// Export is in progress. `fraction` is in `[0.0, 1.0]`.
    Progress { fraction: f32, message: String },
    /// Export completed successfully.
    Done { output_path: PathBuf },
    /// Export failed with an error message.
    Failed { message: String },
    /// Export was cancelled by the user.
    Cancelled,
}

// ─── ExportJob ────────────────────────────────────────────────────────────────

/// Parameters for a single export run.
#[derive(Debug, Clone)]
pub struct ExportJob {
    /// Ordered list of clip segments to concatenate.
    pub segments: Vec<ExportSegment>,
    /// Destination file path (including extension).
    pub output_path: PathBuf,
    /// Container format.
    pub format: OutputFormat,
    /// Quality preset.
    pub quality: QualityPreset,
    /// Project frame rate.
    pub fps: u32,
    /// Output width in pixels.
    pub width: u32,
    /// Output height in pixels.
    pub height: u32,
}

// ─── ExportEncoder ────────────────────────────────────────────────────────────

/// Encodes an edited timeline to an MP4 / MOV / MKV output file.
///
/// Spawns `ffmpeg` as a child process.  Progress updates are sent over an
/// [`mpsc`] channel so the UI can update a progress bar without blocking.
///
/// # Usage
///
/// ```no_run
/// let (encoder, rx) = ExportEncoder::begin(job)?;
/// // poll rx each frame for ExportProgress updates
/// encoder.cancel(); // if the user hits cancel
/// ```
pub struct ExportEncoder {
    child: Child,
    _thread: thread::JoinHandle<()>,
}

impl ExportEncoder {
    /// Start exporting `job` using the system `ffmpeg` executable.
    ///
    /// Returns the encoder handle and a receiver for progress updates.
    /// The receiver will eventually receive [`ExportProgress::Done`] or
    /// [`ExportProgress::Failed`].
    ///
    /// Returns an error immediately if `ffmpeg` is not found in PATH or if
    /// the job has no segments.
    pub fn begin(job: ExportJob) -> Result<(Self, mpsc::Receiver<ExportProgress>)> {
        if job.segments.is_empty() {
            bail!("Cannot export: timeline has no clips.");
        }

        // Build the ffmpeg command.
        let mut cmd = build_ffmpeg_command(&job)?;

        info!(
            "Starting export → {:?}  segments={}",
            job.output_path,
            job.segments.len()
        );
        debug!("ffmpeg args: {:?}", cmd.get_args().collect::<Vec<_>>());

        // Capture stderr for progress parsing.
        cmd.stdout(Stdio::null()).stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .context("Failed to spawn ffmpeg. Is ffmpeg installed and in PATH?")?;

        let stderr = child
            .stderr
            .take()
            .expect("stderr was configured as Stdio::piped()");

        let (tx, rx) = mpsc::channel::<ExportProgress>();
        let total_duration: f64 = job.segments.iter().map(|s| s.duration).sum();
        let output_path = job.output_path.clone();

        // Spawn a thread to parse ffmpeg's stderr progress output.
        let thread = thread::spawn(move || {
            parse_ffmpeg_stderr(stderr, tx, total_duration, output_path);
        });

        Ok((
            Self {
                child,
                _thread: thread,
            },
            rx,
        ))
    }

    /// Cancel the running export by killing the ffmpeg process.
    pub fn cancel(mut self) {
        if let Err(e) = self.child.kill() {
            warn!("Could not kill ffmpeg process: {e}");
        }
        // The stderr thread will detect EOF and exit naturally.
    }

    /// Check if the ffmpeg process has finished.
    pub fn is_finished(&mut self) -> bool {
        self.child.try_wait().map(|s| s.is_some()).unwrap_or(true)
    }
}

// ─── Command builder ──────────────────────────────────────────────────────────

fn build_ffmpeg_command(job: &ExportJob) -> Result<Command> {
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y"); // overwrite output without asking

    // ── Input files ──────────────────────────────────────────────────────────
    for seg in &job.segments {
        // Seek before input is the fast path for ffmpeg.
        cmd.args(["-ss", &format!("{:.6}", seg.source_start)]);
        cmd.args(["-t", &format!("{:.6}", seg.duration)]);
        cmd.args(["-i", &seg.source_path.to_string_lossy()]);
    }

    let n = job.segments.len();

    if n == 1 {
        // Simple single-clip copy / transcode — no filter_complex needed.
        let seg = &job.segments[0];
        if seg.has_video {
            cmd.args(["-c:v", "libx264"]);
            cmd.args(["-crf", &job.quality.crf().to_string()]);
            cmd.args(["-preset", "medium"]);
            cmd.args([
                "-vf",
                &format!("scale={}:{}", job.width, job.height),
            ]);
        } else {
            cmd.args(["-vn"]);
        }
        if seg.has_audio {
            cmd.args(["-c:a", job.format.audio_codec()]);
        } else {
            cmd.args(["-an"]);
        }
    } else {
        // Multiple clips: build a filter_complex concat graph.
        // [0:v][0:a][1:v][1:a]...[N-1:v][N-1:a] concat=n=N:v=1:a=1 [v][a]
        let mut filter = String::new();
        let mut has_any_video = false;
        let mut has_any_audio = false;

        for (i, seg) in job.segments.iter().enumerate() {
            if seg.has_video {
                filter.push_str(&format!(
                    "[{i}:v]scale={w}:{h},fps={fps},setsar=1[v{i}];",
                    i = i,
                    w = job.width,
                    h = job.height,
                    fps = job.fps,
                ));
                has_any_video = true;
            }
            if seg.has_audio {
                has_any_audio = true;
            }
        }

        // Concat filter inputs.
        let mut concat_inputs = String::new();
        for (i, seg) in job.segments.iter().enumerate() {
            if seg.has_video {
                concat_inputs.push_str(&format!("[v{i}]"));
            }
            if seg.has_audio {
                concat_inputs.push_str(&format!("[{i}:a]"));
            }
        }

        let v_streams = job.segments.iter().filter(|s| s.has_video).count();
        let a_streams = job.segments.iter().filter(|s| s.has_audio).count();

        // Use the minimum count so concat doesn't fail on mixed streams.
        let concat_n = v_streams.min(a_streams).max(1);
        let v_out = if has_any_video { 1 } else { 0 };
        let a_out = if has_any_audio { 1 } else { 0 };

        filter.push_str(&format!(
            "{inputs}concat=n={n}:v={v}:a={a}[outv][outa]",
            inputs = concat_inputs,
            n = concat_n,
            v = v_out,
            a = a_out,
        ));

        cmd.args(["-filter_complex", &filter]);

        if has_any_video {
            cmd.args(["-map", "[outv]"]);
            cmd.args(["-c:v", "libx264"]);
            cmd.args(["-crf", &job.quality.crf().to_string()]);
            cmd.args(["-preset", "medium"]);
        }
        if has_any_audio {
            cmd.args(["-map", "[outa]"]);
            cmd.args(["-c:a", job.format.audio_codec()]);
        }
    }

    // Progress output via stderr (default) — request time-based stats.
    cmd.args(["-stats", "-loglevel", "error"]);

    cmd.arg(job.output_path.to_string_lossy().as_ref());

    Ok(cmd)
}

// ─── Progress parsing ─────────────────────────────────────────────────────────

fn parse_ffmpeg_stderr(
    stderr: impl std::io::Read,
    tx: mpsc::Sender<ExportProgress>,
    total_duration: f64,
    output_path: PathBuf,
) {
    let reader = BufReader::new(stderr);
    let mut last_fraction = 0.0f32;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        // ffmpeg -stats writes lines like:
        // frame=  120 fps= 60 q=23.0 size=    512kB time=00:00:04.00 bitrate=1048.6kbits/s speed=2.00x
        if let Some(fraction) = parse_time_fraction(&line, total_duration) {
            last_fraction = fraction;
            let _ = tx.send(ExportProgress::Progress {
                fraction,
                message: line.trim().to_owned(),
            });
        } else if !line.is_empty() {
            // Log any other stderr as warnings.
            warn!("ffmpeg: {line}");
        }
    }

    // EOF on stderr means ffmpeg exited.  We don't know the exit code here,
    // but if progress reached ~100% we treat it as success.
    if last_fraction >= 0.99 {
        let _ = tx.send(ExportProgress::Done { output_path });
    } else {
        // Either cancelled or failed.  We send Cancelled; the caller can
        // distinguish by checking whether it triggered a cancel.
        let _ = tx.send(ExportProgress::Cancelled);
    }
}

/// Parse a `time=HH:MM:SS.cc` field from an ffmpeg stats line and convert
/// it to a progress fraction in `[0.0, 1.0]`.
fn parse_time_fraction(line: &str, total_duration: f64) -> Option<f32> {
    if total_duration <= 0.0 {
        return None;
    }
    // Find "time=HH:MM:SS.cc" in the line.
    let idx = line.find("time=")?;
    let rest = line[idx + 5..].trim_start();
    // Expect "HH:MM:SS.cc" format.
    let mut parts = rest.splitn(2, ':');
    let h: f64 = parts.next()?.parse().ok()?;
    let rest = parts.next()?;
    let mut parts = rest.splitn(2, ':');
    let m: f64 = parts.next()?.parse().ok()?;
    let s: f64 = parts.next()?.parse().ok()?;
    let elapsed = h * 3600.0 + m * 60.0 + s;
    Some((elapsed / total_duration).clamp(0.0, 1.0) as f32)
}

// ─── FFmpeg availability check ────────────────────────────────────────────────

/// Returns `true` if `ffmpeg` is available in PATH.
pub fn ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
