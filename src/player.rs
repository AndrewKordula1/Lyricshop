//! Chord playback: synthesized tones through rodio.
//! Port of the playback half of `ChordPlayer.kt`, but instead of the JVM's
//! MIDI synth it synthesizes each chord as mixed sine waves (one per note)
//! and plays them with rodio. No MIDI output, no audio files.
//!
//! Behaviour matches the Kotlin app:
//! - One bar per chord at the song's BPM (bars split on `|`, chords in a
//!   bar share it evenly; no `|` means every chord gets its own bar).
//! - Playback loops until stop, or until another section starts.
//! - Transpose -6..+6 semitones, playback-only (written chords unchanged).
//! - Power chords ('5') play root–fifth–octave (see `chords.rs`).
//! - The waveform is a listening preference, not saved. There are no
//!   sampled instruments here — the synth only produces three
//!   waveforms (sine, triangle, sawtooth), so the selector lists
//!   those by name instead of pretending to be a piano or a guitar.

use crate::chords::{midi_to_freq, parse_bars, quarter_beats_per_bar, transpose_notes};
use crate::song::SongSection;
use rodio::{OutputStream, Sink};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub const SAMPLE_RATE: u32 = 44100;

/// A playback timbre (waveform). These are the only sounds the synth
/// produces, so the waveform selector lists them by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timbre {
    /// Pure sine wave, soft and round.
    Sine,
    /// Triangle wave, plucked feel.
    Triangle,
    /// Sawtooth via additive harmonics, bright and sustained.
    Sawtooth,
}

impl Timbre {
    /// Every waveform the synth can produce, in selector order.
    pub const ALL: [Timbre; 3] = [Timbre::Sine, Timbre::Triangle, Timbre::Sawtooth];

    /// Display name for the waveform selector.
    pub fn name(self) -> &'static str {
        match self {
            Timbre::Sine => "Sine Wave",
            Timbre::Triangle => "Triangle Wave",
            Timbre::Sawtooth => "Sawtooth Wave",
        }
    }
}

/// Transpose label for the header dropdown, matching the Kotlin app:
/// 0 is "Standard", -2 is "-2 (D std.)".
pub fn transpose_label(semitones: i32) -> String {
    match semitones {
        0 => "Standard".to_string(),
        -2 => "-2 (D std.)".to_string(),
        n if n > 0 => format!("+{}", n),
        n => format!("{}", n),
    }
}

/// One waveform sample for a note at frequency `freq`, time `t` seconds.
fn wave_sample(timbre: Timbre, freq: f64, t: f64) -> f32 {
    let phase = 2.0 * std::f64::consts::PI * freq * t;
    match timbre {
        Timbre::Sine => phase.sin() as f32,
        Timbre::Triangle => (2.0 / std::f64::consts::PI * phase.sin().asin()) as f32,
        Timbre::Sawtooth => {
            // Additive sawtooth: fundamental + a few harmonics, quieter.
            let mut s = 0.0;
            for h in 1..=6 {
                s += (h as f64 * phase).sin() / h as f64;
            }
            (s * 0.5) as f32
        }
    }
}

/// Mix a chord's notes into one mono buffer of `duration_ms` milliseconds.
/// Pure function (no audio device needed) so it is unit-testable.
/// Each note is a sine-family wave with an exponential decay envelope and
/// a short attack ramp to avoid clicks; notes are mixed and normalized so
/// the chord never clips.
pub fn chord_samples(midi_notes: &[i32], duration_ms: u64, timbre: Timbre) -> Vec<f32> {
    let total = (duration_ms as f64 * SAMPLE_RATE as f64 / 1000.0) as usize;
    let mut out = vec![0.0f32; total];
    if midi_notes.is_empty() || total == 0 {
        return out;
    }
    let freqs: Vec<f64> = midi_notes.iter().map(|n| midi_to_freq(*n)).collect();
    let n = freqs.len() as f32;
    for i in 0..total {
        let t = i as f64 / SAMPLE_RATE as f64;
        let phase = i as f64 / total as f64;
        // Attack ramp (first 5ms) + exponential decay, piano-like.
        let attack = (t / 0.005).min(1.0);
        let decay = (-2.5 * phase).exp();
        let env = attack * decay;
        let mut s = 0.0f32;
        for &f in &freqs {
            s += wave_sample(timbre, f, t);
        }
        out[i] = (s / n) * env as f32 * 0.9;
    }
    out
}

struct PlaybackState {
    is_playing: bool,
    playing_section_id: Option<String>,
    unavailable: bool,
}

pub struct ChordPlayer {
    /// Kept alive for the app's lifetime; rodio needs the stream.
    _stream: Option<OutputStream>,
    sink: Option<Arc<Sink>>,
    state: Arc<Mutex<PlaybackState>>,
    generation: Arc<AtomicU64>,
    stop_requested: Arc<AtomicBool>,
    pub timbre: Timbre,
    pub transpose_semitones: i32,
}

impl ChordPlayer {
    /// Create the player. If no audio device is available (headless or
    /// minimal install), the player is marked unavailable instead of
    /// failing — the play button greys out, like the Kotlin app.
    pub fn new() -> Self {
        let mut unavailable = false;
        let (stream, sink) = match rodio::OutputStreamBuilder::open_default_stream() {
            Ok(stream) => {
                let sink = rodio::Sink::connect_new(stream.mixer());
                (Some(stream), Some(Arc::new(sink)))
            }
            Err(_) => {
                unavailable = true;
                (None, None)
            }
        };
        ChordPlayer {
            _stream: stream,
            sink,
            state: Arc::new(Mutex::new(PlaybackState {
                is_playing: false,
                playing_section_id: None,
                unavailable,
            })),
            generation: Arc::new(AtomicU64::new(0)),
            stop_requested: Arc::new(AtomicBool::new(false)),
            timbre: Timbre::Sine,
            transpose_semitones: 0,
        }
    }

    pub fn is_playing(&self) -> bool {
        self.state.lock().unwrap().is_playing
    }

    pub fn playing_section_id(&self) -> Option<String> {
        self.state.lock().unwrap().playing_section_id.clone()
    }

    pub fn unavailable(&self) -> bool {
        self.state.lock().unwrap().unavailable
    }

    /// Start looping playback of a section's progression. Starting a new
    /// playback cancels any older one (generation bump).
    pub fn play(&self, section: &SongSection, bpm: i32) {
        if self.unavailable() {
            return;
        }
        let Some(sink) = self.sink.clone() else {
            return;
        };
        let bars = parse_bars(section.chords.as_deref().unwrap_or(""));
        if bars.is_empty() {
            return;
        }
        let my_generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        self.stop_requested.store(false, Ordering::SeqCst);
        let bpm = bpm.clamp(20, 400);
        let beat_ms = 60000.0 / bpm as f64;
        let bar_ms = (beat_ms * quarter_beats_per_bar(section.time_signature.as_deref())) as u64;
        let timbre = self.timbre;
        let transpose = self.transpose_semitones;
        let section_id = section.id.clone();

        {
            let mut st = self.state.lock().unwrap();
            st.is_playing = true;
            st.playing_section_id = Some(section_id.clone());
        }

        let state = self.state.clone();
        let generation = self.generation.clone();
        let stop_requested = self.stop_requested.clone();
        std::thread::spawn(move || {
            // Loop until stop or a newer playback starts.
            while generation.load(Ordering::SeqCst) == my_generation
                && !stop_requested.load(Ordering::SeqCst)
            {
                for bar in &bars {
                    if generation.load(Ordering::SeqCst) != my_generation
                        || stop_requested.load(Ordering::SeqCst)
                    {
                        break;
                    }
                    let chord_ms = bar_ms / bar.len().max(1) as u64;
                    for notes in bar {
                        if generation.load(Ordering::SeqCst) != my_generation
                            || stop_requested.load(Ordering::SeqCst)
                        {
                            break;
                        }
                        let shifted = transpose_notes(notes, transpose);
                        let samples = chord_samples(&shifted, chord_ms, timbre);
                        let buf = rodio::buffer::SamplesBuffer::new(1, SAMPLE_RATE, samples);
                        sink.append(buf);
                        // Sleep most of the chord so the queue stays short
                        // and a stop lands promptly (same idea as the
                        // Kotlin sleepInterruptible).
                        sleep_interruptible(chord_ms, &stop_requested, &generation, my_generation);
                    }
                }
            }
            sink.stop();
            if generation.load(Ordering::SeqCst) == my_generation {
                let mut st = state.lock().unwrap();
                st.is_playing = false;
                st.playing_section_id = None;
            }
        });
    }

    /// Only flags the stop. It must NOT bump the generation: the playback
    /// thread's cleanup only runs when its generation still matches, so
    /// bumping it here would leave the button stuck on stop (the Kotlin
    /// bug that was fixed the same way).
    pub fn stop(&self) {
        self.stop_requested.store(true, Ordering::SeqCst);
        if let Some(sink) = &self.sink {
            sink.stop();
        }
    }
}

fn sleep_interruptible(
    ms: u64,
    stop_requested: &AtomicBool,
    generation: &AtomicU64,
    my_generation: u64,
) {
    let mut remaining = ms;
    while remaining > 0
        && !stop_requested.load(Ordering::SeqCst)
        && generation.load(Ordering::SeqCst) == my_generation
    {
        let chunk = remaining.min(50);
        std::thread::sleep(Duration::from_millis(chunk));
        remaining = remaining.saturating_sub(chunk);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chord_samples_have_right_length_and_no_clip() {
        let samples = chord_samples(&[69, 76, 81], 500, Timbre::Sine);
        assert_eq!(samples.len(), 500 * 44100 / 1000);
        assert!(samples.iter().all(|s| s.abs() <= 1.0));
        // Starts near silence (attack ramp) and decays.
        assert!(samples[0].abs() < 0.01);
        let mid = samples[samples.len() / 2].abs();
        let end = samples[samples.len() - 1].abs();
        assert!(end <= mid + 0.01);
    }

    #[test]
    fn empty_chord_is_silence() {
        let samples = chord_samples(&[], 200, Timbre::Sine);
        assert!(samples.iter().all(|s| *s == 0.0));
    }

    #[test]
    fn timbres_differ() {
        let sine = chord_samples(&[69], 100, Timbre::Sine);
        let tri = chord_samples(&[69], 100, Timbre::Triangle);
        let saw = chord_samples(&[69], 100, Timbre::Sawtooth);
        assert_ne!(sine, tri);
        assert_ne!(sine, saw);
    }

    #[test]
    fn transpose_labels() {
        assert_eq!(transpose_label(0), "Standard");
        assert_eq!(transpose_label(-2), "-2 (D std.)");
        assert_eq!(transpose_label(3), "+3");
        assert_eq!(transpose_label(-6), "-6");
    }

    #[test]
    fn timbres_are_the_three_waveforms() {
        assert_eq!(Timbre::ALL.len(), 3);
        assert_eq!(Timbre::ALL[0].name(), "Sine Wave");
        assert_eq!(Timbre::ALL[1].name(), "Triangle Wave");
        assert_eq!(Timbre::ALL[2].name(), "Sawtooth Wave");
    }
}
