//! De-esser — sibilance reduction using a biquad sidechain detector.
//!
//! Detects energy in the sibilant frequency range (typically 4–9 kHz) and
//! applies dynamic gain reduction when it exceeds the threshold.

use serde::{Deserialize, Serialize};

use crate::buffer::AudioBuffer;
use crate::dsp::biquad::{BiquadFilter, FilterType};
use crate::dsp::db_to_amplitude;

/// De-esser parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DeEsserParams {
    /// Center frequency of the sibilance band in Hz (typically 5000–8000).
    pub freq_hz: f32,
    /// Threshold in dB — sibilance above this is reduced.
    pub threshold_db: f32,
    /// Maximum gain reduction in dB (positive value, e.g., 6.0).
    pub reduction_db: f32,
    /// Q factor / bandwidth of the detection and reduction band.
    pub q: f32,
}

impl Default for DeEsserParams {
    fn default() -> Self {
        Self {
            freq_hz: 6000.0,
            threshold_db: -20.0,
            reduction_db: 6.0,
            q: 2.0,
        }
    }
}

/// Sibilance reduction processor.
#[derive(Debug, Clone)]
pub struct DeEsser {
    params: DeEsserParams,
    /// Band-pass filter for detecting sibilant energy.
    detector: BiquadFilter,
    sample_rate: u32,
    channels: u32,
}

impl DeEsser {
    /// Create a new de-esser.
    pub fn new(params: DeEsserParams, sample_rate: u32, channels: u32) -> Self {
        let detector = BiquadFilter::new(
            FilterType::BandPass,
            params.freq_hz,
            params.q,
            sample_rate,
            channels,
        );
        Self {
            params,
            detector,
            sample_rate,
            channels,
        }
    }

    /// Process an audio buffer in-place.
    pub fn process(&mut self, buf: &mut AudioBuffer) {
        let ch = buf.channels as usize;
        // We need a copy for sidechain detection
        let mut sidechain = buf.clone();
        self.detector.process(&mut sidechain);

        let threshold_lin = db_to_amplitude(self.params.threshold_db);
        let max_reduction_lin = db_to_amplitude(-self.params.reduction_db);

        for frame in 0..buf.frames {
            // Detect sidechain peak across channels
            let mut sidechain_peak = 0.0f32;
            for c in 0..ch {
                let idx = frame * ch + c;
                sidechain_peak = sidechain_peak.max(sidechain.samples[idx].abs());
            }

            // If sidechain exceeds threshold, compute gain reduction
            if sidechain_peak > threshold_lin {
                let excess_ratio = sidechain_peak / threshold_lin;
                // Proportional reduction: more excess = more reduction
                let reduction = (1.0 / excess_ratio).max(max_reduction_lin);

                for c in 0..ch {
                    let idx = frame * ch + c;
                    buf.samples[idx] *= reduction;
                }
            }
        }
    }

    /// Reset filter state.
    pub fn reset(&mut self) {
        self.detector.reset();
    }

    /// Update parameters.
    pub fn set_params(&mut self, params: DeEsserParams) {
        self.detector = BiquadFilter::new(
            FilterType::BandPass,
            params.freq_hz,
            params.q,
            self.sample_rate,
            self.channels,
        );
        self.params = params;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sine(freq: f32, amplitude: f32, frames: usize) -> AudioBuffer {
        let samples: Vec<f32> = (0..frames)
            .map(|i| {
                amplitude * (2.0 * std::f32::consts::PI * freq * i as f32 / 44100.0).sin()
            })
            .collect();
        AudioBuffer::from_interleaved(samples, 1, 44100).unwrap()
    }

    #[test]
    fn low_frequency_unaffected() {
        let params = DeEsserParams {
            freq_hz: 6000.0,
            threshold_db: -30.0,
            reduction_db: 12.0,
            q: 2.0,
        };
        let mut deesser = DeEsser::new(params, 44100, 1);
        let mut buf = make_sine(200.0, 0.8, 4096);
        let original_rms = buf.rms();
        deesser.process(&mut buf);
        // Low frequency should pass through mostly unchanged
        assert!(
            (buf.rms() - original_rms).abs() < original_rms * 0.1,
            "200Hz should not trigger de-esser at 6kHz"
        );
    }

    #[test]
    fn sibilance_reduced() {
        let params = DeEsserParams {
            freq_hz: 6000.0,
            threshold_db: -30.0,
            reduction_db: 12.0,
            q: 1.0,
        };
        let mut deesser = DeEsser::new(params, 44100, 1);
        let mut buf = make_sine(6000.0, 0.8, 4096);
        let original_rms = buf.rms();
        deesser.process(&mut buf);
        assert!(
            buf.rms() < original_rms * 0.8,
            "6kHz signal should be reduced by de-esser"
        );
    }

    #[test]
    fn below_threshold_passthrough() {
        let params = DeEsserParams {
            freq_hz: 6000.0,
            threshold_db: 0.0, // Very high threshold
            reduction_db: 12.0,
            q: 2.0,
        };
        let mut deesser = DeEsser::new(params, 44100, 1);
        let mut buf = make_sine(6000.0, 0.1, 4096);
        let original_rms = buf.rms();
        deesser.process(&mut buf);
        // Signal too quiet to trigger de-esser
        assert!(
            (buf.rms() - original_rms).abs() < original_rms * 0.15,
            "Below-threshold signal should be near-unchanged"
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut deesser = DeEsser::new(DeEsserParams::default(), 44100, 1);
        let mut buf = make_sine(6000.0, 0.8, 256);
        deesser.process(&mut buf);
        deesser.reset();
        // Should not panic and detector should be clean
        let out = deesser.detector.process_sample(0.0, 0);
        assert!(out.abs() < f32::EPSILON);
    }
}
