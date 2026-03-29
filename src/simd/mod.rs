//! SIMD-accelerated audio processing kernels.
//!
//! Provides platform-specific SIMD implementations for hot-path audio operations.
//! Falls back to scalar code when SIMD is not available or the `simd` feature is disabled.
//!
//! - **x86_64**: SSE2 (baseline, 4 f32/op) + AVX2 (runtime-detected, 8 f32/op)
//! - **aarch64**: NEON (baseline, 4 f32/op)

#[cfg(target_arch = "x86_64")]
#[allow(unused_unsafe, clippy::needless_range_loop)]
mod x86;

#[cfg(target_arch = "aarch64")]
mod aarch64;

// ── Platform dispatch ───────────────────────────────────────────────

#[cfg(target_arch = "x86_64")]
#[inline]
pub fn add_buffers(dst: &mut [f32], src: &[f32]) {
    x86::add_buffers(dst, src)
}
#[cfg(target_arch = "aarch64")]
#[inline]
pub fn add_buffers(dst: &mut [f32], src: &[f32]) {
    aarch64::add_buffers(dst, src)
}
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
#[inline]
pub fn add_buffers(dst: &mut [f32], src: &[f32]) {
    add_buffers_scalar(dst, src)
}

#[cfg(target_arch = "x86_64")]
#[inline]
pub fn apply_gain(samples: &mut [f32], gain: f32) {
    x86::apply_gain(samples, gain)
}
#[cfg(target_arch = "aarch64")]
#[inline]
pub fn apply_gain(samples: &mut [f32], gain: f32) {
    aarch64::apply_gain(samples, gain)
}
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
#[inline]
pub fn apply_gain(samples: &mut [f32], gain: f32) {
    apply_gain_scalar(samples, gain)
}

#[cfg(target_arch = "x86_64")]
#[inline]
pub fn clamp(samples: &mut [f32], min: f32, max: f32) {
    x86::clamp(samples, min, max)
}
#[cfg(target_arch = "aarch64")]
#[inline]
pub fn clamp(samples: &mut [f32], min: f32, max: f32) {
    aarch64::clamp(samples, min, max)
}
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
#[inline]
pub fn clamp(samples: &mut [f32], min: f32, max: f32) {
    clamp_scalar(samples, min, max)
}

#[cfg(target_arch = "x86_64")]
#[must_use]
#[inline]
pub fn peak_abs(samples: &[f32]) -> f32 {
    x86::peak_abs(samples)
}
#[cfg(target_arch = "aarch64")]
#[must_use]
#[inline]
pub fn peak_abs(samples: &[f32]) -> f32 {
    aarch64::peak_abs(samples)
}
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
#[must_use]
#[inline]
pub fn peak_abs(samples: &[f32]) -> f32 {
    peak_abs_scalar(samples)
}

#[cfg(target_arch = "x86_64")]
#[must_use]
#[inline]
pub fn sum_of_squares(samples: &[f32]) -> f64 {
    x86::sum_of_squares(samples)
}
#[cfg(target_arch = "aarch64")]
#[must_use]
#[inline]
pub fn sum_of_squares(samples: &[f32]) -> f64 {
    aarch64::sum_of_squares(samples)
}
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
#[must_use]
#[inline]
pub fn sum_of_squares(samples: &[f32]) -> f64 {
    sum_of_squares_scalar(samples)
}

#[cfg(target_arch = "x86_64")]
#[inline]
pub fn noise_gate(samples: &mut [f32], threshold: f32) {
    x86::noise_gate(samples, threshold)
}
#[cfg(target_arch = "aarch64")]
#[inline]
pub fn noise_gate(samples: &mut [f32], threshold: f32) {
    aarch64::noise_gate(samples, threshold)
}
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
#[inline]
pub fn noise_gate(samples: &mut [f32], threshold: f32) {
    noise_gate_scalar(samples, threshold)
}

#[cfg(target_arch = "x86_64")]
#[inline]
pub fn i16_to_f32(src: &[i16], dst: &mut [f32]) {
    x86::i16_to_f32(src, dst)
}
#[cfg(target_arch = "aarch64")]
#[inline]
pub fn i16_to_f32(src: &[i16], dst: &mut [f32]) {
    aarch64::i16_to_f32(src, dst)
}
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
#[inline]
pub fn i16_to_f32(src: &[i16], dst: &mut [f32]) {
    i16_to_f32_scalar(src, dst)
}

#[cfg(target_arch = "x86_64")]
#[inline]
pub fn f32_to_i16(src: &[f32], dst: &mut [i16]) {
    x86::f32_to_i16(src, dst)
}
#[cfg(target_arch = "aarch64")]
#[inline]
pub fn f32_to_i16(src: &[f32], dst: &mut [i16]) {
    aarch64::f32_to_i16(src, dst)
}
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
#[inline]
pub fn f32_to_i16(src: &[f32], dst: &mut [i16]) {
    f32_to_i16_scalar(src, dst)
}

/// Weighted dot product: sum(samples[i] * weights[i]) for pre-computed sinc kernels.
/// Returns (weighted_sum, weight_sum) for normalization.
#[cfg(target_arch = "x86_64")]
#[must_use]
#[inline]
pub fn weighted_sum(samples: &[f32], weights: &[f32]) -> (f32, f32) {
    x86::weighted_sum(samples, weights)
}
#[cfg(target_arch = "aarch64")]
#[must_use]
#[inline]
pub fn weighted_sum(samples: &[f32], weights: &[f32]) -> (f32, f32) {
    aarch64::weighted_sum(samples, weights)
}
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
#[must_use]
#[inline]
pub fn weighted_sum(samples: &[f32], weights: &[f32]) -> (f32, f32) {
    weighted_sum_scalar(samples, weights)
}

// ── Scalar fallbacks ────────────────────────────────────────────────

#[allow(dead_code)]
fn add_buffers_scalar(dst: &mut [f32], src: &[f32]) {
    let len = dst.len().min(src.len());
    for i in 0..len {
        dst[i] += src[i];
    }
}

#[allow(dead_code)]
fn apply_gain_scalar(samples: &mut [f32], gain: f32) {
    for s in samples.iter_mut() {
        *s *= gain;
    }
}

#[allow(dead_code)]
fn clamp_scalar(samples: &mut [f32], min: f32, max: f32) {
    for s in samples.iter_mut() {
        *s = s.clamp(min, max);
    }
}

#[allow(dead_code)]
fn peak_abs_scalar(samples: &[f32]) -> f32 {
    samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max)
}

#[allow(dead_code)]
fn sum_of_squares_scalar(samples: &[f32]) -> f64 {
    samples.iter().map(|s| (*s as f64) * (*s as f64)).sum()
}

#[allow(dead_code)]
fn noise_gate_scalar(samples: &mut [f32], threshold: f32) {
    for s in samples.iter_mut() {
        if s.abs() < threshold {
            *s = 0.0;
        }
    }
}

#[allow(dead_code)]
fn i16_to_f32_scalar(src: &[i16], dst: &mut [f32]) {
    let len = src.len().min(dst.len());
    for i in 0..len {
        dst[i] = src[i] as f32 / 32768.0;
    }
}

#[allow(dead_code)]
fn weighted_sum_scalar(samples: &[f32], weights: &[f32]) -> (f32, f32) {
    let len = samples.len().min(weights.len());
    let mut sum = 0.0f32;
    let mut weight_sum = 0.0f32;
    for i in 0..len {
        sum += samples[i] * weights[i];
        weight_sum += weights[i];
    }
    (sum, weight_sum)
}

#[allow(dead_code)]
fn f32_to_i16_scalar(src: &[f32], dst: &mut [i16]) {
    let len = src.len().min(dst.len());
    for i in 0..len {
        dst[i] = (src[i].clamp(-1.0, 1.0) * 32767.0) as i16;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_buffers_basic() {
        let mut dst = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let src = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0];
        add_buffers(&mut dst, &src);
        assert_eq!(
            dst,
            vec![11.0, 22.0, 33.0, 44.0, 55.0, 66.0, 77.0, 88.0, 99.0]
        );
    }

    #[test]
    fn add_buffers_mismatched_len() {
        let mut dst = vec![1.0, 2.0, 3.0];
        let src = vec![10.0, 20.0];
        add_buffers(&mut dst, &src);
        assert_eq!(dst, vec![11.0, 22.0, 3.0]);
    }

    #[test]
    fn apply_gain_basic() {
        let mut samples = vec![0.5, -0.5, 1.0, -1.0, 0.25];
        apply_gain(&mut samples, 2.0);
        assert_eq!(samples, vec![1.0, -1.0, 2.0, -2.0, 0.5]);
    }

    #[test]
    fn clamp_basic() {
        let mut samples = vec![2.0, -2.0, 0.5, -0.5, 1.5, -1.5, 0.0, 0.99, -0.99];
        clamp(&mut samples, -1.0, 1.0);
        assert_eq!(
            samples,
            vec![1.0, -1.0, 0.5, -0.5, 1.0, -1.0, 0.0, 0.99, -0.99]
        );
    }

    #[test]
    fn peak_abs_basic() {
        let samples = vec![0.3, -0.7, 0.5, 0.1, -0.2, 0.6, -0.4, 0.0, 0.69];
        assert!((peak_abs(&samples) - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn peak_abs_empty() {
        assert_eq!(peak_abs(&[]), 0.0);
    }

    #[test]
    fn sum_of_squares_basic() {
        let samples = vec![1.0, 2.0, 3.0];
        let result = sum_of_squares(&samples);
        assert!((result - 14.0).abs() < 1e-6);
    }

    #[test]
    fn noise_gate_basic() {
        let mut samples = vec![0.01, -0.01, 0.5, -0.5, 0.001, 0.8];
        noise_gate(&mut samples, 0.1);
        assert_eq!(samples[0], 0.0);
        assert_eq!(samples[1], 0.0);
        assert!((samples[2] - 0.5).abs() < f32::EPSILON);
        assert!((samples[3] + 0.5).abs() < f32::EPSILON);
        assert_eq!(samples[4], 0.0);
        assert!((samples[5] - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn i16_f32_roundtrip() {
        let src_i16: Vec<i16> = vec![0, 16384, -16384, 32767, -32768];
        let mut f32_buf = vec![0.0f32; 5];
        i16_to_f32(&src_i16, &mut f32_buf);
        let mut back_i16 = vec![0i16; 5];
        f32_to_i16(&f32_buf, &mut back_i16);
        for (a, b) in src_i16.iter().zip(back_i16.iter()) {
            assert!((*a as i32 - *b as i32).abs() <= 1, "{a} != {b}");
        }
    }

    #[test]
    fn weighted_sum_basic() {
        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let weights = vec![0.5, 0.5, 0.5, 0.5, 0.5];
        let (sum, wt) = weighted_sum(&samples, &weights);
        assert!((sum - 7.5).abs() < 1e-4, "weighted_sum={sum}");
        assert!((wt - 2.5).abs() < 1e-4, "weight_sum={wt}");
    }

    #[test]
    fn weighted_sum_unequal_weights() {
        let samples = vec![1.0, 2.0, 3.0];
        let weights = vec![1.0, 0.0, 0.5];
        let (sum, wt) = weighted_sum(&samples, &weights);
        assert!((sum - 2.5).abs() < 1e-4);
        assert!((wt - 1.5).abs() < 1e-4);
    }

    /// Verify SIMD paths match scalar fallbacks for all kernels.
    #[test]
    fn simd_scalar_parity() {
        let data: Vec<f32> = (0..1025).map(|i| (i as f32 * 0.01).sin() * 0.9).collect();

        // peak_abs
        let simd_peak = peak_abs(&data);
        let scalar_peak = super::peak_abs_scalar(&data);
        assert!(
            (simd_peak - scalar_peak).abs() < 1e-6,
            "peak_abs: simd={simd_peak} scalar={scalar_peak}"
        );

        // sum_of_squares
        let simd_sos = sum_of_squares(&data);
        let scalar_sos = super::sum_of_squares_scalar(&data);
        assert!(
            (simd_sos - scalar_sos).abs() < 1e-3,
            "sum_of_squares: simd={simd_sos} scalar={scalar_sos}"
        );

        // add_buffers
        let mut simd_dst = vec![1.0f32; data.len()];
        let mut scalar_dst = simd_dst.clone();
        add_buffers(&mut simd_dst, &data);
        super::add_buffers_scalar(&mut scalar_dst, &data);
        assert_eq!(simd_dst, scalar_dst, "add_buffers mismatch");

        // apply_gain
        let mut simd_gain = data.clone();
        let mut scalar_gain = data.clone();
        apply_gain(&mut simd_gain, 0.75);
        super::apply_gain_scalar(&mut scalar_gain, 0.75);
        for (i, (s, sc)) in simd_gain.iter().zip(scalar_gain.iter()).enumerate() {
            assert!(
                (s - sc).abs() < 1e-6,
                "apply_gain[{i}]: simd={s} scalar={sc}"
            );
        }

        // clamp
        let mut simd_clamp: Vec<f32> = data.iter().map(|s| s * 2.0).collect();
        let mut scalar_clamp = simd_clamp.clone();
        clamp(&mut simd_clamp, -0.5, 0.5);
        super::clamp_scalar(&mut scalar_clamp, -0.5, 0.5);
        assert_eq!(simd_clamp, scalar_clamp, "clamp mismatch");

        // noise_gate
        let mut simd_gate = data.clone();
        let mut scalar_gate = data.clone();
        noise_gate(&mut simd_gate, 0.3);
        super::noise_gate_scalar(&mut scalar_gate, 0.3);
        assert_eq!(simd_gate, scalar_gate, "noise_gate mismatch");

        // i16_to_f32 / f32_to_i16 roundtrip parity
        let i16_data: Vec<i16> = (0..1025).map(|i| ((i * 31) % 65536) as i16).collect();
        let mut simd_f32 = vec![0.0f32; i16_data.len()];
        let mut scalar_f32 = vec![0.0f32; i16_data.len()];
        i16_to_f32(&i16_data, &mut simd_f32);
        super::i16_to_f32_scalar(&i16_data, &mut scalar_f32);
        for (i, (s, sc)) in simd_f32.iter().zip(scalar_f32.iter()).enumerate() {
            assert!(
                (s - sc).abs() < 1e-6,
                "i16_to_f32[{i}]: simd={s} scalar={sc}"
            );
        }

        let mut simd_i16 = vec![0i16; simd_f32.len()];
        let mut scalar_i16 = vec![0i16; scalar_f32.len()];
        f32_to_i16(&simd_f32, &mut simd_i16);
        super::f32_to_i16_scalar(&scalar_f32, &mut scalar_i16);
        // Allow ±1 difference: SIMD uses round-to-nearest, scalar uses truncation
        for (i, (s, sc)) in simd_i16.iter().zip(scalar_i16.iter()).enumerate() {
            assert!(
                (*s as i32 - *sc as i32).abs() <= 1,
                "f32_to_i16[{i}]: simd={s} scalar={sc}"
            );
        }

        // weighted_sum
        let weights: Vec<f32> = (0..data.len()).map(|i| (i as f32 * 0.003).cos()).collect();
        let (simd_ws, simd_wt) = weighted_sum(&data, &weights);
        let (scalar_ws, scalar_wt) = super::weighted_sum_scalar(&data, &weights);
        assert!(
            (simd_ws - scalar_ws).abs() < 1e-2,
            "weighted_sum: simd={simd_ws} scalar={scalar_ws}"
        );
        assert!(
            (simd_wt - scalar_wt).abs() < 1e-2,
            "weight_sum: simd={simd_wt} scalar={scalar_wt}"
        );
    }

    #[test]
    fn various_buffer_sizes() {
        for size in [0, 1, 3, 4, 7, 8, 15, 16, 17] {
            let mut dst = vec![1.0f32; size];
            let src = vec![2.0f32; size];
            add_buffers(&mut dst, &src);
            assert!(
                dst.iter().all(|&s| (s - 3.0).abs() < f32::EPSILON),
                "add size={size}"
            );

            let mut samples = vec![0.5f32; size];
            apply_gain(&mut samples, 2.0);
            assert!(
                samples.iter().all(|&s| (s - 1.0).abs() < f32::EPSILON),
                "gain size={size}"
            );

            let mut samples = vec![2.0f32; size];
            clamp(&mut samples, -1.0, 1.0);
            assert!(
                samples.iter().all(|&s| (s - 1.0).abs() < f32::EPSILON),
                "clamp size={size}"
            );
        }
    }
}
