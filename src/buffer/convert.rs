//! Format conversion — sample format, layout, and channel mapping.

use crate::NadaError;
use crate::buffer::AudioBuffer;

/// Convert i16 interleaved samples to f32 (-1.0 to ~1.0).
pub fn i16_to_f32(samples: &[i16]) -> Vec<f32> {
    let mut dst = vec![0.0f32; samples.len()];
    #[cfg(feature = "simd")]
    {
        crate::simd::i16_to_f32(samples, &mut dst);
    }
    #[cfg(not(feature = "simd"))]
    {
        for (i, &s) in samples.iter().enumerate() {
            dst[i] = s as f32 / 32768.0;
        }
    }
    dst
}

/// Convert f32 samples to i16 with clamping.
pub fn f32_to_i16(samples: &[f32]) -> Vec<i16> {
    let mut dst = vec![0i16; samples.len()];
    #[cfg(feature = "simd")]
    {
        crate::simd::f32_to_i16(samples, &mut dst);
    }
    #[cfg(not(feature = "simd"))]
    {
        for (i, &s) in samples.iter().enumerate() {
            let clamped = s.clamp(-1.0, 1.0);
            dst[i] = (clamped * 32767.0) as i16;
        }
    }
    dst
}

/// Convert i32 interleaved samples to f32 (-1.0 to ~1.0).
pub fn i32_to_f32(samples: &[i32]) -> Vec<f32> {
    samples
        .iter()
        .map(|&s| s as f32 / 2_147_483_648.0)
        .collect()
}

/// Convert f32 samples to i32 with clamping.
pub fn f32_to_i32(samples: &[f32]) -> Vec<i32> {
    samples
        .iter()
        .map(|&s| {
            let clamped = s.clamp(-1.0, 1.0);
            (clamped as f64 * 2_147_483_647.0) as i32
        })
        .collect()
}

/// Convert an interleaved AudioBuffer to planar representation.
///
/// Returns one `Vec<f32>` per channel.
pub fn interleaved_to_planar(buf: &AudioBuffer) -> Vec<Vec<f32>> {
    let ch = buf.channels as usize;
    let mut planes: Vec<Vec<f32>> = (0..ch).map(|_| Vec::with_capacity(buf.frames)).collect();

    for frame in 0..buf.frames {
        for (c, plane) in planes.iter_mut().enumerate() {
            plane.push(buf.samples[frame * ch + c]);
        }
    }

    planes
}

/// Convert planar channels back to an interleaved AudioBuffer.
///
/// All channels must have the same length.
pub fn planar_to_interleaved(
    channels: &[Vec<f32>],
    sample_rate: u32,
) -> Result<AudioBuffer, NadaError> {
    if channels.is_empty() {
        return Err(NadaError::InvalidChannels(0));
    }

    let frames = channels[0].len();
    for (i, ch) in channels.iter().enumerate() {
        if ch.len() != frames {
            return Err(NadaError::Conversion(format!(
                "channel {} has {} frames, expected {}",
                i,
                ch.len(),
                frames
            )));
        }
    }

    let ch_count = channels.len();
    let mut samples = Vec::with_capacity(frames * ch_count);
    for frame in 0..frames {
        for ch in channels {
            samples.push(ch[frame]);
        }
    }

    AudioBuffer::from_interleaved(samples, ch_count as u32, sample_rate)
}

/// Duplicate mono buffer to stereo.
pub fn mono_to_stereo(buf: &AudioBuffer) -> Result<AudioBuffer, NadaError> {
    if buf.channels != 1 {
        return Err(NadaError::Conversion(format!(
            "expected mono (1 channel), got {}",
            buf.channels
        )));
    }

    let mut samples = Vec::with_capacity(buf.frames * 2);
    for &s in &buf.samples {
        samples.push(s);
        samples.push(s);
    }

    AudioBuffer::from_interleaved(samples, 2, buf.sample_rate)
}

/// Mix stereo buffer down to mono (average of L and R).
pub fn stereo_to_mono(buf: &AudioBuffer) -> Result<AudioBuffer, NadaError> {
    if buf.channels != 2 {
        return Err(NadaError::Conversion(format!(
            "expected stereo (2 channels), got {}",
            buf.channels
        )));
    }

    let mut samples = Vec::with_capacity(buf.frames);
    for frame in 0..buf.frames {
        let l = buf.samples[frame * 2];
        let r = buf.samples[frame * 2 + 1];
        samples.push((l + r) * 0.5);
    }

    AudioBuffer::from_interleaved(samples, 1, buf.sample_rate)
}

/// Downmix 5.1 surround (6 channels) to stereo using ITU-R BS.775 coefficients.
///
/// Assumed channel order: L, R, C, LFE, Ls, Rs (SMPTE/ITU standard).
///
/// ```text
/// L_out = L + 0.707 * C + 0.707 * Ls
/// R_out = R + 0.707 * C + 0.707 * Rs
/// ```
///
/// LFE is discarded (standard practice for non-bass-managed systems).
pub fn downmix_5_1_to_stereo(buf: &AudioBuffer) -> Result<AudioBuffer, NadaError> {
    if buf.channels != 6 {
        return Err(NadaError::Conversion(format!(
            "expected 5.1 (6 channels), got {}",
            buf.channels
        )));
    }

    let coeff: f32 = std::f32::consts::FRAC_1_SQRT_2; // 0.7071...
    let mut samples = Vec::with_capacity(buf.frames * 2);

    for frame in 0..buf.frames {
        let base = frame * 6;
        let l = buf.samples[base]; // L
        let r = buf.samples[base + 1]; // R
        let c = buf.samples[base + 2]; // Center
        // base + 3 = LFE (discarded)
        let ls = buf.samples[base + 4]; // Left surround
        let rs = buf.samples[base + 5]; // Right surround

        samples.push(l + coeff * c + coeff * ls);
        samples.push(r + coeff * c + coeff * rs);
    }

    AudioBuffer::from_interleaved(samples, 2, buf.sample_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i16_f32_roundtrip() {
        let original: Vec<i16> = vec![0, 16384, -16384, 32767, -32768];
        let f32s = i16_to_f32(&original);
        let back = f32_to_i16(&f32s);
        for (a, b) in original.iter().zip(back.iter()) {
            assert!((*a as i32 - *b as i32).abs() <= 1, "{a} != {b}");
        }
    }

    #[test]
    fn i32_f32_roundtrip() {
        let original: Vec<i32> = vec![0, 1_073_741_824, -1_073_741_824];
        let f32s = i32_to_f32(&original);
        let back = f32_to_i32(&f32s);
        for (a, b) in original.iter().zip(back.iter()) {
            // Allow some precision loss due to f32 intermediate
            let tolerance = 256;
            assert!((*a as i64 - *b as i64).abs() <= tolerance, "{a} != {b}");
        }
    }

    #[test]
    fn f32_to_i16_clamps() {
        let samples = vec![2.0, -2.0, 0.5];
        let result = f32_to_i16(&samples);
        assert_eq!(result[0], 32767);
        assert_eq!(result[1], -32767);
    }

    #[test]
    fn f32_to_i32_clamps() {
        let samples = vec![2.0, -2.0];
        let result = f32_to_i32(&samples);
        assert_eq!(result[0], i32::MAX);
        assert_eq!(result[1], -i32::MAX);
    }

    #[test]
    fn interleaved_planar_roundtrip() {
        let buf = AudioBuffer::from_interleaved(vec![1.0, 2.0, 3.0, 4.0], 2, 44100).unwrap();
        let planes = interleaved_to_planar(&buf);
        assert_eq!(planes.len(), 2);
        assert_eq!(planes[0], vec![1.0, 3.0]); // L
        assert_eq!(planes[1], vec![2.0, 4.0]); // R

        let back = planar_to_interleaved(&planes, 44100).unwrap();
        assert_eq!(back.samples, vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(back.channels, 2);
    }

    #[test]
    fn planar_rejects_mismatched_lengths() {
        let planes = vec![vec![1.0, 2.0], vec![3.0]];
        assert!(planar_to_interleaved(&planes, 44100).is_err());
    }

    #[test]
    fn planar_rejects_empty() {
        let planes: Vec<Vec<f32>> = vec![];
        assert!(planar_to_interleaved(&planes, 44100).is_err());
    }

    #[test]
    fn mono_to_stereo_duplicates() {
        let buf = AudioBuffer::from_interleaved(vec![0.5, -0.5], 1, 44100).unwrap();
        let stereo = mono_to_stereo(&buf).unwrap();
        assert_eq!(stereo.channels, 2);
        assert_eq!(stereo.frames, 2);
        assert_eq!(stereo.samples, vec![0.5, 0.5, -0.5, -0.5]);
    }

    #[test]
    fn mono_to_stereo_rejects_non_mono() {
        let buf = AudioBuffer::from_interleaved(vec![0.5, -0.5], 2, 44100).unwrap();
        assert!(mono_to_stereo(&buf).is_err());
    }

    #[test]
    fn stereo_to_mono_averages() {
        let buf = AudioBuffer::from_interleaved(vec![1.0, 0.0, 0.0, 1.0], 2, 44100).unwrap();
        let mono = stereo_to_mono(&buf).unwrap();
        assert_eq!(mono.channels, 1);
        assert_eq!(mono.frames, 2);
        assert!((mono.samples[0] - 0.5).abs() < f32::EPSILON);
        assert!((mono.samples[1] - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn stereo_to_mono_rejects_non_stereo() {
        let buf = AudioBuffer::from_interleaved(vec![0.5], 1, 44100).unwrap();
        assert!(stereo_to_mono(&buf).is_err());
    }

    #[test]
    fn downmix_5_1_basic() {
        // All channels at 1.0 — check coefficient application
        let samples = vec![1.0f32; 6 * 2]; // 2 frames of 6 channels
        let buf = AudioBuffer::from_interleaved(samples, 6, 48000).unwrap();
        let stereo = downmix_5_1_to_stereo(&buf).unwrap();
        assert_eq!(stereo.channels, 2);
        assert_eq!(stereo.frames, 2);
        let coeff = std::f32::consts::FRAC_1_SQRT_2;
        let expected = 1.0 + coeff + coeff; // L + 0.707*C + 0.707*Ls
        assert!((stereo.samples[0] - expected).abs() < 0.001);
    }

    #[test]
    fn downmix_5_1_rejects_non_6ch() {
        let buf = AudioBuffer::from_interleaved(vec![0.0; 4], 2, 44100).unwrap();
        assert!(downmix_5_1_to_stereo(&buf).is_err());
    }
}
