//! Voice synthesis — glottal source, formant filtering, phoneme sequencing, prosody.
//!
//! Re-exports from [`svara`](https://crates.io/crates/svara), the AGNOS vocal synthesis crate.
//! Produces deterministic, real-time voice from phoneme sequences — no neural TTS.
//!
//! # Feature: `voice`
//!
//! Enable with:
//! ```toml
//! dhvani = { version = "0.22", features = ["voice"] }
//! ```
//!
//! # Data Flow
//!
//! ```text
//! text → phoneme sequence → prosody contour → glottal source → vocal tract → AudioBuffer
//! ```

use crate::buffer::AudioBuffer;

// ── Voice profiles ──────────────────────────────────────────────────

/// Speaker voice characteristics (f0, breathiness, vibrato, jitter, shimmer).
pub use svara::voice::VoiceProfile;

// ── Glottal source ──────────────────────────────────────────────────

/// Glottal pulse model selection (Rosenberg, Liljencrants-Fant).
pub use svara::glottal::{GlottalModel, GlottalSource};

// ── Formant synthesis ───────────────────────────────────────────────

/// Single formant resonance (frequency, bandwidth, amplitude).
pub use svara::formant::Formant;

/// Vowel category for formant lookup.
pub use svara::formant::Vowel;

/// Formant frequency/bandwidth targets (F1–F5).
pub use svara::formant::VowelTarget;

/// Parallel bank of biquad filters tuned to formant frequencies.
pub use svara::formant::FormantFilter;

// ── Vocal tract ─────────────────────────────────────────────────────

/// Place of articulation for nasal consonants.
pub use svara::tract::{NasalPlace, VocalTract};

// ── Phonemes ────────────────────────────────────────────────────────

/// IPA-subset phoneme inventory (~50 phonemes).
pub use svara::phoneme::{Phoneme, PhonemeClass};

/// Formant targets for a phoneme.
pub use svara::phoneme::phoneme_formants;

/// Default duration for a phoneme.
pub use svara::phoneme::phoneme_duration;

/// F2 locus equation for stop consonants.
pub use svara::phoneme::f2_locus_equation;

/// Synthesize a single phoneme to samples.
pub use svara::phoneme::synthesize_phoneme;

// ── Prosody ─────────────────────────────────────────────────────────

/// Stress level (primary, secondary, unstressed).
pub use svara::prosody::Stress;

/// Intonation patterns (declarative, interrogative, continuation, exclamatory).
pub use svara::prosody::IntonationPattern;

/// Prosodic contour — f0 trajectory, duration, and amplitude scaling.
pub use svara::prosody::ProsodyContour;

// ── Sequencing ──────────────────────────────────────────────────────

/// Timed phoneme event (phoneme + duration + stress).
pub use svara::sequence::PhonemeEvent;

/// Ordered phoneme sequence with coarticulation and crossfading.
pub use svara::sequence::PhonemeSequence;

// ── Spectral analysis ───────────────────────────────────────────────

/// Spectral analysis for vocal synthesis diagnostics.
pub use svara::spectral;

/// Svara error type.
pub use svara::prelude::SvaraError;

// ── Bridge: svara synthesis → dhvani AudioBuffer ────────────────────

/// Render a [`PhonemeSequence`] to a dhvani [`AudioBuffer`].
///
/// # Errors
///
/// Returns `crate::NadaError::Dsp` if the sequence fails to render.
///
/// # Example
///
/// ```rust,no_run
/// use dhvani::voice_synth::*;
///
/// let voice = VoiceProfile::new_female();
/// let mut seq = PhonemeSequence::new();
/// seq.push(PhonemeEvent::new(Phoneme::VowelA, 0.3, Stress::Primary));
/// seq.push(PhonemeEvent::new(Phoneme::NasalN, 0.1, Stress::Unstressed));
///
/// let buf = render_sequence(&seq, &voice, 44100).unwrap();
/// ```
pub fn render_sequence(
    sequence: &PhonemeSequence,
    voice: &VoiceProfile,
    sample_rate: u32,
) -> crate::Result<AudioBuffer> {
    let samples = sequence
        .render(voice, sample_rate as f32)
        .map_err(|e| crate::NadaError::Dsp(format!("voice synthesis failed: {e}")))?;
    AudioBuffer::from_interleaved(samples, 1, sample_rate).map_err(|e| {
        crate::NadaError::Dsp(format!("failed to create buffer from voice output: {e}"))
    })
}

/// Render a single [`Phoneme`] to a dhvani [`AudioBuffer`].
///
/// # Errors
///
/// Returns `crate::NadaError::Dsp` if synthesis fails.
pub fn render_phoneme(
    phoneme: &Phoneme,
    voice: &VoiceProfile,
    sample_rate: u32,
    duration: f32,
) -> crate::Result<AudioBuffer> {
    let samples = synthesize_phoneme(phoneme, voice, sample_rate as f32, duration)
        .map_err(|e| crate::NadaError::Dsp(format!("phoneme synthesis failed: {e}")))?;
    AudioBuffer::from_interleaved(samples, 1, sample_rate).map_err(|e| {
        crate::NadaError::Dsp(format!("failed to create buffer from phoneme output: {e}"))
    })
}

/// Render a [`GlottalSource`] through a [`VocalTract`] for the given number of frames.
///
/// This is the lowest-level voice rendering function — for when you need
/// direct control over the glottal source and tract parameters.
pub fn render_vocal_tract(
    source: &mut GlottalSource,
    tract: &mut VocalTract,
    frames: usize,
    sample_rate: u32,
) -> AudioBuffer {
    let mut samples = Vec::with_capacity(frames);
    for _ in 0..frames {
        let glottal = source.next_sample();
        samples.push(tract.process_sample(glottal));
    }
    AudioBuffer::from_interleaved(samples, 1, sample_rate)
        .unwrap_or_else(|_| AudioBuffer::silence(1, frames, sample_rate.max(1)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_vowel_a() {
        let voice = VoiceProfile::new_male();
        let buf = render_phoneme(&Phoneme::VowelA, &voice, 44100, 0.2).unwrap();
        assert_eq!(buf.channels(), 1);
        assert!(buf.frames() > 0);
        assert!(buf.rms() > 0.0);
        assert!(buf.samples().iter().all(|s| s.is_finite()));
    }

    #[test]
    fn render_phoneme_sequence() {
        let voice = VoiceProfile::new_female();
        let mut seq = PhonemeSequence::new();
        seq.push(PhonemeEvent::new(Phoneme::VowelA, 0.2, Stress::Primary));
        seq.push(PhonemeEvent::new(Phoneme::NasalN, 0.1, Stress::Unstressed));
        seq.push(PhonemeEvent::new(Phoneme::VowelI, 0.2, Stress::Unstressed));

        let buf = render_sequence(&seq, &voice, 44100).unwrap();
        assert!(buf.frames() > 0);
        assert!(buf.rms() > 0.0);
    }

    #[test]
    fn voice_profiles() {
        let male = VoiceProfile::new_male();
        let female = VoiceProfile::new_female();
        let child = VoiceProfile::new_child();

        // Female f0 > male f0 > 0
        assert!(female.base_f0 > male.base_f0);
        assert!(child.base_f0 > female.base_f0);
        assert!(male.base_f0 > 0.0);
    }

    #[test]
    fn glottal_source_renders() {
        let voice = VoiceProfile::new_male();
        let mut source = voice.create_glottal_source(44100.0).unwrap();
        let mut tract = VocalTract::new(44100.0);

        let buf = render_vocal_tract(&mut source, &mut tract, 4410, 44100);
        assert_eq!(buf.frames(), 4410);
        assert!(buf.samples().iter().all(|s| s.is_finite()));
    }
}
