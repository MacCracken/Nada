# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] — 2026-03-29

### Changed — BREAKING

#### API Hardening
- `Compressor::set_params()`, `EnvelopeLimiter::set_params()`, `DeEsser::set_params()`, `Envelope::set_params()`, `Reverb::set_params()` now return `Result` — parameters are validated on update (previously bypassed constructor checks)
- `#[non_exhaustive]` added to all public parameter structs (`CompressorParams`, `LimiterParams`, `DeEsserParams`, `ReverbParams`, `AdsrParams`, `ModulatedDelayParams`, `EqBandConfig`, `GainSmootherParams`, `GraphicEqSettings`) and several data structs (`DynamicsAnalysis`, `Spectrogram`, `R128Loudness`, `OnsetResult`, `WaveformData`, `NoteEvent`, `ControlChange`, `MidiClip`, `CcMapping`, `Connection`, `CaptureConfig`, `OutputConfig`, `AudioDevice`)
- `#[non_exhaustive]` added to `EnvelopeState`, `VoiceState`, `CrossfadeType`, `FadeCurve` enums
- `AudioBuffer::from_interleaved()` now rejects sample counts not divisible by channel count (returns `LengthMismatch`)
- Builder constructors added: `CompressorParams::new().with_threshold().with_ratio()...`, `ReverbParams::new().with_room_size()...`, `LimiterParams::new().with_ceiling()...`, `EqBandConfig::new()`
- `NoteEvent::new()` and `ControlChange::new()` constructors added

#### API Encapsulation — v1.0 Freeze
- `DynamicsAnalysis` fields now private — use `peak()`, `peak_db()`, `true_peak()`, `true_peak_db()`, `rms()`, `rms_db()`, `crest_factor_db()`, `lufs()`, `dynamic_range_db()`, `frame_count()`, `channel_count()` accessors
- `R128Loudness` fields now private — use `integrated_lufs()`, `range_lu()`, `short_term_lufs()`, `momentary_lufs()` accessors
- `OnsetResult` fields now private — use `positions()`, `strengths()`, `count()` accessors
- `NoteEvent` fields now private — use `position()`, `duration()`, `note()`, `velocity()`, `channel()` accessors
- `ControlChange` fields now private — use `position()`, `controller()`, `value()`, `channel()` accessors
- `MidiClip` fields now private — use `name()`, `notes()`, `control_changes()`, `timeline_pos()`, `duration()` accessors
- `Connection` fields now private — use `from()`, `to()` accessors
- `NodeId` inner field now private — use `value()` accessor
- `LevelMeter` fields now private — use `peak()`, `rms()`, `lufs()` accessors (plus existing `peak_db()`, `rms_db()`, `peak_hold()`)

### Fixed

#### Correctness
- **SVF Peak/Shelf modes** — Peak mode now correctly boosts/cuts the bandpass component around cutoff (`input + (A²-1) * BP`) with modified `k = 1/(Q*A)` per Cytomic spec (was computing `HP * A²`, a scaled high-pass). Shelf modes: removed dead `* 0.0` terms; formulas were correct underneath
- **R128 loudness** — integrated, ungated, and short-term LUFS now averaged in linear power domain per EBU R128 spec (was incorrectly averaging in dB domain)
- **True peak detection** — upgraded from linear interpolation (which always equaled sample peak) to 4-point cubic Hermite interpolation for proper inter-sample peak detection
- **`AudioClock::set_tempo()`** — now clamps negative and NaN values to 0 (was storing invalid values)
- **`Lfo::set_rate()`** — now clamps negative values to 0 (was allowing negative phase drift)
- **`GainSmoother::new()`** — now clamps attack/release to 0.0–1.0 (was accepting any value)
- **FFI `nada_buffer_silence()`** — now returns null for `channels == 0` or `sample_rate == 0`
- **Oscillator triangle waveform** — now uses PolyBLEP-integrated leaky integrator for proper anti-aliasing (was using naive formula despite docs claiming PolyBLEP)
- **STFT `hop_size.max(1)`** dead code removed (error check already handled `hop_size == 0`)
- **`resample_linear()`** — early return for empty buffers and upper bound validation on target_rate

#### Performance
- **Graph processor** — output buffers reused across cycles (was allocating fresh `AudioBuffer::silence` per node per cycle)
- **STFT** — scratch `real`/`imag` buffers allocated once and reused across frames (was allocating per frame)
- **Noise reduction** — same scratch buffer reuse optimization
- **R128 K-weighting** — filters samples directly instead of cloning the entire `AudioBuffer` (avoids duplicating metadata + allocation overhead)
- **LevelMeter peak hold** — decay now scales by frame count (`powi(frames)`) for buffer-size-independent rate

### Added

#### Annotations
- `#[must_use]` on all public structs and pure functions across the crate (~50+ types), including `BiquadFilter::process_sample()`, `SvfFilter::process_sample()`
- `#[inline]` on hot-path functions: all DSP `process()` methods (BiquadFilter, Compressor, DeEsser, DelayLine, ModulatedDelay, EnvelopeLimiter, ParametricEq, GraphicEq, Reverb), `BiquadState::process`, `BiquadFilter::process_sample`, `Oscillator::sample`, `Envelope::tick`, `Lfo::tick`, `StereoPanner::process`, `GainSmoother::smooth`, SIMD dispatch functions, clock accessors, MIDI translate functions, meter store/load
- `AudioBuffer::silence()` now debug-asserts `channels > 0` and `sample_rate > 0`
- `// SAFETY:` comments on all unsafe blocks in NEON biquad_stereo (aarch64.rs)

#### Tracing
- `tracing::debug!` on all DSP effect constructors: BiquadFilter, SvfFilter, DelayLine, ModulatedDelay, StereoPanner, GainSmoother, GraphicEq, NoiseReducer (plus existing Compressor, Limiter, DeEsser, ParametricEq, Reverb)
- `tracing::debug!` on all analysis entry points (measure_r128, analyze_dynamics, stft, chromagram, detect_onsets)
- `tracing::warn!` on all error paths in conversion functions and FFI null returns
- `tracing::debug!` on noise_reduce, resample_sinc, Graph::compile

#### DSP
- **`ConvolutionReverb`** — FFT-based partitioned overlap-add convolution reverb engine. Works with any `&[f32]` impulse response. Supports mono/stereo, configurable block size, dry/wet mix, IR hot-swap
- **`NoiseReducer`** — stateful spectral noise reducer that reuses Hann window, FFT scratch, and magnitude buffers across calls (avoids 3 large allocations per call). `noise_reduce()` still available as convenience wrapper

#### Acoustics Integration (feature: `acoustics`)
- **`acoustics` module** — bridges [`goonj`](https://crates.io/crates/goonj) 1.1.1 acoustic simulation into dhvani's processing pipeline
- `convolution_from_ir()` / `convolution_from_dhvani_ir()` — create `ConvolutionReverb` from goonj impulse responses
- `convolve_multiband()` — frequency-dependent reverb from `MultibandIr` (8 octave bands)
- `process_fdn()` — FDN late reverb processing into `AudioBuffer` with mono-sum input
- `decode_bformat_stereo()` — decode 1st-order Ambisonics B-Format IR to stereo (W+Y / W-Y cardioid)
- `export_ir_wav()` — export IR as 16-bit PCM WAV
- `generate_ir()` — convenience wrapper for `generate_dhvani_ir` using `(f32, f32, f32)` tuples (no hisab dep needed)
- `acoustics::presets` — 6 curated room presets: studio, rehearsal room, concert hall, bathroom, cathedral, outdoor
- Re-exports: `AcousticRoom`, `AcousticMaterial`, `ImpulseResponse`, `MultibandIr`, `DhvaniIr`, `Fdn`, `FdnConfig`, `BFormatIr`, `HoaIr`, room acoustics analysis functions

#### Documentation
- Redundant explicit rustdoc link targets fixed in lib.rs

#### Synthesis Integration (feature: `synthesis`)
- **`synthesis` module** — re-exports from [`naad`](https://crates.io/crates/naad) 1.0.0: subtractive, FM, additive, wavetable, granular, physical modeling, drum, vocoder synthesis engines
- `render_to_buffer()` and `render_stereo_to_buffer()` — bridge functions from sample-based synthesis to dhvani `AudioBuffer`
- All naad core primitives available: oscillators, SVF/biquad filters, ADSR/multi-stage envelopes, LFO, noise generators, mod matrix, voice manager, parameter smoothing, tuning, panning, effects (chorus, flanger, phaser, distortion)

#### Voice Synthesis Integration (feature: `voice`)
- **`voice_synth` module** — re-exports from [`svara`](https://crates.io/crates/svara) 1.0.0: glottal source (Rosenberg/LF models), formant filtering, vocal tract modeling, phoneme inventory (~50 IPA phonemes), prosody contours, phoneme sequencing with coarticulation
- `render_sequence()` — render a `PhonemeSequence` to `AudioBuffer`
- `render_phoneme()` — render a single `Phoneme` to `AudioBuffer`
- `render_vocal_tract()` — low-level glottal source + vocal tract rendering
- Voice profiles: male, female, child with breathiness, vibrato, jitter, shimmer control

#### Tests
- 18 DSP reference accuracy tests (0.01 dB tolerance): biquad LP/HP/peaking/shelf/allpass/notch, limiter ceiling, compressor unity, panner constant power, coefficient DC/Nyquist gain, dB roundtrip
- 8 convolution reverb tests (identity, delay, stereo, long IR, reset, hot-swap)
- 9 acoustics integration tests (IR generation, FDN, multiband, B-Format decode, WAV export, room presets)
- 7 SVF peak/shelf correctness tests
- 5 PipeWire lifecycle tests (capture start/stop, output send, device enumeration, idempotent start)
- 4 MIDI accessor tests, 2 error Display tests
- 9 P(-1) hardening tests + 7 synthesis tests + 4 voice tests
- 90.02% line coverage via cargo-llvm-cov (597 unit + 22 doc = 619 total)

### Performance
- stereo_to_mono: 97µs → 54µs (−45%)
- mono_to_stereo: 97µs → 68µs (−30%)
- noise_gate: 17.5µs → 10.9µs (−38%)
- reverb: 1.14ms → 967µs (−15%)
- limiter: 689µs → 614µs (−11%)
- parametric_eq_10band: 2.96ms → 2.56ms (−14%)

### Tooling
- Dependencies updated (proptest 1.11, serde_spanned 1.1, cc 1.2.58, etc.)

## [0.22.4] — 2026-03-22

### Changed — BREAKING

#### API Encapsulation
- `AudioClock` fields (`position_samples`, `sample_rate`, `tempo_bpm`, `running`) are now private — use accessor methods `position_samples()`, `sample_rate()`, `tempo_bpm()`, `is_running()`, `set_tempo()`

### Added

#### Abaco Integration
- Added `abaco` 0.22.4 as dependency — shared DSP math crate for the AGNOS ecosystem
- `dsp::amplitude_to_db`, `db_to_amplitude`, `sanitize_sample` now re-exported from `abaco::dsp`
- Biquad filter design uses `abaco::dsp::{angular_frequency, db_gain_factor}`
- Compressor/limiter time constants use `abaco::dsp::time_constant`
- Oscillator uses `abaco::dsp::poly_blep`
- Panner uses `abaco::dsp::constant_power_pan`
- Crossfade uses `abaco::dsp::equal_power_crossfade`
- MIDI voice frequency uses `abaco::dsp::midi_to_freq`
- MIDI constants (`A4_FREQUENCY`, `A4_MIDI_NOTE`, `SEMITONES_PER_OCTAVE`) re-exported from abaco

#### DSP Consistency
- `set_sample_rate()` on all stateful DSP effects: BiquadFilter, Compressor, EnvelopeLimiter, ParametricEq, DeEsser, Reverb, ModulatedDelay, Envelope, Lfo
- `set_bypass()`/`is_bypassed()` on all dynamic effects: BiquadFilter, Compressor, EnvelopeLimiter, ParametricEq, DeEsser, Reverb, DelayLine, ModulatedDelay
- `ParametricEq::set_params()` — bulk band replacement
- `ModulatedDelay::set_params()` — runtime parameter updates
- `DelayLine::latency_frames()` and `ModulatedDelay::latency_frames()` — latency reporting for PDC
- `AudioClock::set_tempo()` — runtime BPM changes

#### Refactoring
- `dsp::soft_knee_gain()` — shared soft-knee gain computation used by Compressor and EnvelopeLimiter
- Removed duplicated `time_constant()` from Compressor and EnvelopeLimiter (now delegates to `abaco::dsp`)
- Removed inline `poly_blep()` from oscillator (now uses `abaco::dsp`)

#### Tooling
- `scripts/bench-history.sh` — benchmark runner with CSV history + 3-point Markdown tracking

#### Tests
- 15 new tests: bypass, set_sample_rate, soft_knee_gain, clock getters, latency_frames, set_params (431 total)

## [0.21.4] — 2026-03-21

### Changed — BREAKING

#### API Encapsulation
- `AudioBuffer` fields (`samples`, `channels`, `sample_rate`, `frames`) are now `pub(crate)` — use accessor methods `samples()`, `samples_mut()`, `channels()`, `sample_rate()`, `frames()` instead
- `Spectrum` fields are now private — use accessor methods `magnitudes()`, `magnitude_db()`, `freq_resolution()`, `sample_rate()`, `fft_size()`, `peak_frequency()`, `peak_magnitude_db()`
- `Chromagram.chroma` is now private — use `chroma()` accessor
- `Voice` fields are now `pub(crate)` — use accessor methods
- `MidiRoute` fields are now private — construct via `MidiRoute::new()`, use getters

#### Analysis Error Propagation
- `spectrum_fft()` now returns `Result<Spectrum, NadaError>` instead of default Spectrum on error
- `spectrum_dft()` now returns `Result<Spectrum, NadaError>`
- `compute_stft()` now returns `Result<Spectrogram, NadaError>`
- `measure_r128()` now returns `Result<R128Loudness, NadaError>`
- `chromagram()` now returns `Result<Chromagram, NadaError>`
- `detect_onsets()` now returns `Result<OnsetResult, NadaError>`

#### Constructor Validation
- `Compressor::new()`, `Reverb::new()`, `EnvelopeLimiter::new()`, `DeEsser::new()` now return `Result` — parameters are validated on construction

### Added

#### Format Conversion
- `SampleFormat::I24`, `SampleFormat::F64`, `SampleFormat::U8` variants
- `i24_to_f32()` / `f32_to_i24()` — 24-bit (i32-padded) conversion
- `i24_packed_to_f32()` / `f32_to_i24_packed()` — 24-bit packed 3-byte LE conversion
- `f64_to_f32()` / `f32_to_f64()` — double-precision conversion
- `u8_to_f32()` / `f32_to_u8()` — unsigned 8-bit PCM (centered at 128)

#### Dithering
- `buffer::dither::tpdf_dither()` — Triangular PDF dithering for bit-depth reduction
- `buffer::dither::noise_shaped_dither()` — first-order error feedback noise-shaped dithering

#### Buffer Utilities
- `buffer::ops::crossfade()` — linear and equal-power crossfade between two buffers
- `buffer::ops::fade_in()` / `fade_out()` — linear and exponential fade ramps
- `buffer::ops::normalize_to_lufs()` — normalize to target LUFS using EBU R128 measurement

#### Memory & Allocation
- `AudioBufferRef<'a>` — zero-copy read-only buffer view (borrows samples, no allocation)
- `BufferPool` — reusable buffer arena to reduce allocation pressure in RT paths
- `StftProcessor` — caches Hann window for repeated STFT computations
- `GraphProcessor` now uses Vec-indexed outputs (was HashMap) and pre-allocated input scratch

#### Parameter Validation
- `AdsrParams::validate()`, `ModulatedDelayParams::validate()`, `Oscillator::validate()`, `Lfo::validate()`
- Sample rate ceiling raised from 384 kHz to 768 kHz

#### Trait Derives
- `GraphicEq` now implements `Debug` and `Clone`

#### Robustness
- `dsp::sanitize_sample()` — NaN/Inf → 0.0 helper
- NaN guards added to reverb, delay, de-esser, limiter process paths
- `// SAFETY:` comments on all unsafe blocks in simd/x86.rs, simd/aarch64.rs, ffi.rs, meter/mod.rs

---

## [0.21.3] — 2026-03-21

### Changed

#### Analysis — BREAKING
- `DynamicsAnalysis` — all fields upgraded from scalar `f32` to per-channel `Vec<f32>`: `peak`, `peak_db`, `true_peak`, `true_peak_db`, `rms`, `rms_db`, `crest_factor_db`. Added `lufs: f32`, `frame_count: usize`, `channel_count: u32`. Convenience methods: `max_peak()`, `max_peak_db()`, `max_true_peak()`, `max_true_peak_db()`, `mean_rms()`, `mean_crest_factor_db()` for whole-buffer summaries
- `Spectrum` — added `magnitude_db: Vec<f32>`, `fft_size: usize`, `peak_frequency: f32`, `peak_magnitude_db: f32` fields. All constructed via internal `from_magnitudes()` which computes dB and peak fields automatically

### Added

#### Analysis
- `Spectrum::spectral_centroid()` — weighted mean frequency by magnitude (brightness indicator)
- `Spectrum::spectral_rolloff(threshold)` — frequency below which a given fraction of spectral energy sits (timbral shape descriptor)

#### Metering
- `LevelMeter` — block-accumulating audio level meter with peak, RMS, LUFS, and peak-hold tracking. Accumulates statistics across multiple `process()` calls and computes integrated LUFS using simplified EBU R128 gating (absolute gate at -70 LUFS, relative gate at mean-10 LU). Includes per-channel peak hold with configurable decay coefficient

---

## [0.20.5] — 2026-03-21

Yanked — superseded by 0.21.3 which includes the same features plus breaking API improvements.

---

## [0.20.4] — 2026-03-20

### Added

#### DSP
- `GainSmoother` — exponential moving average with configurable attack/release coefficients for smooth gain transitions. Prevents pumping in volume normalization workflows
- `GainSmootherParams` — serde-compatible parameters (default: attack 0.3, release 0.05)
- `GraphicEq` — 10-band ISO graphic equalizer (31 Hz–16 kHz) wrapping `ParametricEq` with per-band gain control
- `GraphicEqSettings` — settings with 9 named presets (rock, pop, jazz, classical, bass, treble, vocal, electronic, acoustic)
- `ISO_BANDS` constant — standard 10-band center frequencies

#### Analysis
- `suggest_gain(buf, target_rms) → f32` — per-buffer normalization gain suggestion with 0.1–10.0x clamping. Convenience for media player volume normalization

#### Crate Structure
- Feature flags for module-level compilation: `dsp`, `analysis`, `midi`, `graph` (all default-on)
- `analysis` feature implies `dsp` (R128 K-weighting needs biquad, dynamics needs dB conversion)
- `dsp::noise_reduction` gated behind `analysis` feature (needs FFT)
- Core always available: `buffer`, `capture`, `clock`, `ffi`, `error`
- Consumers can now select only what they need (e.g., `default-features = false, features = ["dsp", "simd"]`)

#### Documentation
- Comprehensive documentation audit and cleanup across all docs
- Updated roadmap: collapsed v0.21–v0.23 into 2 dense releases targeting v1.0
- Architecture overview updated with full module tree
- Migration guide updated with planned v0.21.3 breaking changes

### Fixed
- Sanskrit character: नाद (Nāda) → ध्वनि (Dhvani) in README and docs
- README Quick Start: replaced nonexistent `dsp::compress()` with `Compressor` struct
- README: `spectrum_dft` → `spectrum_fft` in examples
- Roadmap: marked already-completed items (oscillator, envelope, LFO, noise_reduction, waveform, anyhow removal, serde_json)
- Stale version references removed from capability table and roadmap

---

## [0.20.3] — 2026-03-20

### Added

#### Core
- `AudioBuffer` — f32 interleaved audio buffer with channels, sample_rate, frames
- `SampleFormat` (F32, I16, I32) and `Layout` (Interleaved, Planar) enums with `#[non_exhaustive]`
- `AudioClock` — sample-accurate transport with position, tempo, beats, PTS, seek
- `NadaError` enum with FormatMismatch, LengthMismatch, InvalidSampleRate, InvalidChannels, Dsp, Capture, InvalidParameter, Conversion variants

#### DSP
- `BiquadFilter` — 8 filter types (LP, HP, BP, notch, all-pass, peaking, shelf) using Bristow-Johnson cookbook
- `ParametricEq` — N-band biquad cascade with per-band enable/disable
- `Reverb` — Schroeder/Freeverb (4 combs + 2 allpasses, stereo decorrelation)
- `DelayLine` + `ModulatedDelay` — fixed and LFO-modulated for chorus/flanger
- `Compressor` — envelope follower with soft knee, attack/release, makeup gain
- `EnvelopeLimiter` — brick-wall limiter with instant attack, soft knee
- `DeEsser` — biquad sidechain sibilance detection with pre-allocated buffer
- `StereoPanner` — constant-power (sin/cos) panning law
- Stateless: noise gate, hard limiter, normalize, amplitude/dB conversion

#### Analysis
- Radix-2 Cooley-Tukey FFT (O(n log n)) + simple DFT for small windows
- STFT spectrograms with configurable window/hop size
- EBU R128 loudness (K-weighting, 400ms blocks, absolute + relative gating, LRA)
- `DynamicsAnalysis` — true peak (4x oversampled), crest factor, dynamic range
- `Chromagram` — 12 pitch classes mapped from FFT bins
- Onset detection via spectral flux with peak-picking
- Simplified LUFS and silence detection

#### MIDI
- MIDI 1.0: `NoteEvent`, `ControlChange`, `MidiEvent` enum, `MidiClip`
- MIDI 2.0 / UMP: `NoteOnV2`, `NoteOffV2`, `ControlChangeV2`, per-note expression, `UmpMessageType`
- Translation: velocity (7↔16 bit), CC (7↔32 bit), pitch bend (14↔32 bit) with roundtrip tests
- `VoiceManager` — polyphonic voice pool with 4 steal modes (Oldest, Quietest, Lowest, None)
- Routing: `VelocityCurve`, `MidiRoute`, `CcMapping`
- `MidiClip` operations: sorted insert, binary search range query, merge, transpose, quantize

#### SIMD
- SSE2 kernels (x86_64): mix, gain, clamp, peak, RMS, noise gate, i16/f32, weighted sum
- AVX2 kernels (x86_64): mix, gain, clamp, peak — runtime-detected
- NEON kernels (aarch64): mix, gain, clamp, peak, RMS, noise gate, weighted sum
- Platform dispatch module with scalar fallback

#### RT Infrastructure
- `PeakMeter` / `MeterBank` / `SharedMeterBank` — lock-free metering via AtomicU32
- `AudioNode` trait + `Graph` + `ExecutionPlan` + `GraphProcessor` (double-buffered swap)
- `RecordManager` / `LoopRecordManager` — ring-buffer recording with take splitting

#### Capture
- PipeWire capture/output (`PwCapture`, `PwOutput`, `enumerate_devices`)
- Device types, config structs, `CaptureEvent` hot-plug notifications

#### Format Conversion
- i16 ↔ f32, i32 ↔ f32 with clamping
- Interleaved ↔ planar
- Mono → stereo, stereo → mono
- 5.1 → stereo downmix (ITU-R BS.775)
- Sinc resampling (Blackman-Harris window, Draft/Good/Best quality)

#### Crate Quality
- FFI module — C-compatible `nada_buffer_*` API
- CONTRIBUTING.md, SECURITY.md, CODE_OF_CONDUCT.md, deny.toml
- Fuzz targets (mix, resample, DSP chain)
- CI: cargo-vet, cargo-semver-checks, test-minimal, fuzz, bench jobs
- 265+ tests, 7 benchmark suites, 94%+ line coverage
