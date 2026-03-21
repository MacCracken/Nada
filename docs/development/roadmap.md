# Dhvani Roadmap

> **Principle**: Correctness first, then SIMD, then capture backends. Every consumer gets the same audio math.

---

## v0.20.3 (current) — Complete Engine

Everything below was implemented in the initial build-out.

### Core
- [x] AudioBuffer (f32 interleaved), SampleFormat, Layout, mixing, linear + sinc resampling
- [x] AudioClock (position, tempo, beats, PTS, seek)
- [x] NadaError with `#[non_exhaustive]`, Result type alias

### DSP
- [x] Biquad filter (8 types — Bristow-Johnson cookbook)
- [x] Parametric EQ (N-band cascade), StereoPanner (constant-power)
- [x] Reverb (Schroeder/Freeverb), Delay (fixed + modulated chorus/flanger)
- [x] Compressor (envelope follower, soft knee, makeup), EnvelopeLimiter (brick-wall)
- [x] De-esser, noise gate, hard limiter, normalize, dB conversions

### Analysis
- [x] Radix-2 FFT (O(n log n)), STFT spectrograms
- [x] EBU R128 loudness (K-weighting + gating + LRA)
- [x] DynamicsAnalysis (true peak 4x oversampled, crest factor, dynamic range)
- [x] Chromagram (12 pitch classes), onset detection (spectral flux)

### MIDI
- [x] MIDI 1.0: NoteEvent, ControlChange, MidiEvent enum, MidiClip (sorted insert, binary search, merge, transpose, quantize)
- [x] MIDI 2.0 / UMP: NoteOnV2, NoteOffV2, ControlChangeV2, per-note expression, UmpMessageType
- [x] Translation: velocity/CC/pitch bend 1.0↔2.0 with roundtrip tests
- [x] Voice management: VoiceManager, 4 steal modes, polyphonic pool
- [x] Routing: VelocityCurve, MidiRoute, CcMapping

### SIMD
- [x] SSE2/AVX2/NEON kernels: mix, gain, clamp, peak, RMS, noise gate, i16/f32, weighted sum
- [x] Platform dispatch with scalar fallback, runtime AVX2 detection

### RT Infrastructure
- [x] Lock-free metering (PeakMeter, MeterBank, SharedMeterBank via AtomicU32)
- [x] Audio graph (AudioNode trait, Graph → ExecutionPlan → GraphProcessor with double-buffered swap)
- [x] Ring-buffer recording (RecordManager, LoopRecordManager with take splitting)

### Capture
- [x] PipeWire capture/output (PwCapture, PwOutput, enumerate_devices, hot-plug events)
- [x] Device types, config structs, CaptureEvent

### Crate Quality
- [x] Format conversion: i16/i32/f32, interleaved/planar, mono/stereo, 5.1 downmix
- [x] FFI module (C-compatible AudioBuffer API)
- [x] CONTRIBUTING.md, SECURITY.md, deny.toml
- [x] Fuzz targets (mix, resample, DSP), cargo-vet/semver-checks in CI
- [x] 265+ tests, 7 benchmark suites, zero clippy warnings

---

## v0.21.3 — Hardening, Safety & API Freeze

Everything needed to make the public API production-safe and v1.0-ready.

### Panic & safety elimination

- [ ] **FFT panic removal**: Replace `assert!(n.is_power_of_two())` in `fft_in_place()` with graceful `Result` return
- [ ] **Unchecked indexing audit**: Add bounds guards in `interleaved_to_planar()`, `stft()`, `resample_linear()`, `dynamics` loops (~8 sites)
- [ ] **`// SAFETY:` comments on all 106 unsafe blocks**: x86.rs (SSE2/AVX2), aarch64.rs (NEON), ffi.rs, meter/mod.rs — document memory layout, alignment, feature detection invariants
- [ ] **Analysis error propagation**: Change `spectrum_fft()`, `stft()`, `measure_r128()` from silent-default-on-failure to `Result<T, NadaError>` so callers can distinguish silence from error

### API encapsulation

- [ ] **AudioBuffer**: Make `samples`, `channels`, `sample_rate`, `frames` private; accessor methods already exist
- [ ] **Spectrum / Chromagram**: Private `magnitudes`, `chroma`; add read-only accessors
- [ ] **Voice / VoiceManager**: Private state fields; mutation only through `note_on()`/`note_off()`
- [ ] **MidiRoute**: Private fields; validated setters
- [ ] **GraphProcessor**: Private `current_plan`, `pending_plan`, `node_outputs`; mutation only through `swap_handle()`

### Format conversion

- [ ] **24-bit audio (i24 ↔ f32)**: Packed 3-byte and i32-padded variants — the standard pro recording format
- [ ] **f64 ↔ f32**: Double-precision for mastering buses and scientific analysis
- [ ] **u8 ↔ f32**: Unsigned 8-bit PCM (WAV 8-bit, legacy formats). Center at 128, range 0–255
- [ ] Add `I24`, `F64`, `U8` to `SampleFormat` enum

### Parameter validation

- [ ] **Remaining validate() methods**: `AdsrParams`, `ModulatedDelayParams`, `Oscillator`, `Lfo` — all constructors enforce valid ranges
- [ ] **Existing validate() tightening**: `CompressorParams`, `ReverbParams`, `LimiterParams`, `DeEsserParams` — call `validate()` in constructors, not just expose it
- [ ] **Sample rate ceiling**: Raise to 768kHz, guard integer overflow in frame/sample calculations

### Memory & allocation

- [ ] **DeEsser sidechain pre-allocation**: Reusable buffer in struct instead of `buf.clone()` per call
- [ ] **Graph processor Vec-indexed outputs**: Replace `HashMap<NodeId, AudioBuffer>` with `Vec<AudioBuffer>`
- [ ] **Graph input gather pre-allocation**: Pre-allocate input Vec per node
- [ ] **Buffer pool**: Reusable `AudioBuffer` arena — effects borrow instead of allocating
- [ ] **Zero-copy buffer views**: `AudioBufferRef<'a>` for read-only DSP (analysis, metering)
- [ ] **STFT window caching**: Pre-compute Hann window once, reuse across calls

### Robustness

- [ ] **NaN propagation audit**: All DSP effects handle NaN input → 0.0; add `debug_assert!(is_finite())` in test builds
- [ ] **Reverb dead field**: Remove unused `_sample_rate` from Reverb struct
- [ ] Audit `tracing` — instrument key functions or remove

### Completed in v0.20.3

- [x] Remove `anyhow` dep — `NadaError::Other` now uses `Box<dyn Error>`
- [x] `serde_json` already in dev-dependencies
- [x] Spectral noise reduction, waveform extraction, oscillator, ADSR envelope, LFO

---

## v0.22.3 — Testing, SIMD Completeness, Docs & Consumer Integration

Ship-quality validation, close SIMD gaps, documentation for v1.0, and get consumers on board.

### SIMD completeness

- [ ] **AVX2 kernels**: `sum_of_squares`, `noise_gate` — currently SSE2-only on x86_64
- [ ] **NEON kernels**: `i16_to_f32`, `f32_to_i16` — currently scalar fallback on aarch64
- [ ] **SIMD for new formats**: 24-bit and u8 conversion kernels (SSE2 + NEON)
- [ ] **SIMD biquad cross-channel**: Process stereo L+R in single SSE2 register

### Testing

- [ ] **Property-based tests**: Expand proptest coverage — `add_buffers`, `sum_of_squares`, `weighted_sum`, subnormal floats, NaN inputs, all-zero buffers, extreme buffer sizes
- [ ] **SIMD parity tests**: Explicit SIMD vs scalar output comparison for every kernel, every platform
- [ ] **Long-buffer stress tests**: 1-hour processing through full DSP chain
- [ ] **Graph concurrency test**: Multi-threaded plan swapping under RT load
- [ ] **EBU R128 reference vectors**: Validate against EBU tech 3341 test signals
- [ ] **90%+ code coverage** (cargo-llvm-cov)
- [ ] **Benchmark expansion**: `sum_of_squares`, `weighted_sum`, varying buffer sizes (64/256/4096/65536), multi-channel (1/2/6/8ch), SIMD-vs-scalar side-by-side harness

### Performance

- [ ] **Parallel DSP chain**: rayon for independent graph branches
- [ ] **Golden benchmark numbers**: Publish baseline numbers for regression detection

### Graph improvements

- [ ] **Node bypass**: Skip processing without removing from graph
- [ ] **Latency compensation**: Nodes report I/O delay, graph compensates

### Cross-project features

- [ ] **SVF Filter (Cytomic topology)** — alternative to biquad, better for modulated synthesis
- [ ] **Spectral rolloff** — frequency below which 95% of spectral energy sits
- [ ] **EQ presets** — convenience layer over parametric EQ (consumers can also define their own)

### Documentation

- [ ] **RT safety docs**: Which types are RT-safe (no alloc, no lock) vs non-RT
- [ ] **SIMD module docs**: Vectorized operations, expected speedups, platform coverage
- [ ] **FFI usage guide**: C/Python integration examples
- [ ] **Thread-safety annotations**: Document non-Sync DSP types
- [ ] **Complete docs.rs**: Every public type has doc comment + example

### Consumer adoption

- [ ] shruti adopts dhvani (replace shruti-engine + shruti-dsp + shruti-session MIDI)
- [ ] jalwa adopts dhvani (replace playback buffer + EQ + normalization)
- [ ] aethersafta adopts dhvani (replace PipeWire capture + mixer)
- [ ] tazama uses dhvani DSP (replace tazama-media/dsp/)
- [ ] hoosh uses `dhvani::midi` for music token preprocessing
- [ ] Cross-crate integration tests
- [ ] Benchmark regression: dhvani not slower than code it replaces

---

## v1.0.0 Criteria

All must be true:

- [ ] API frozen: AudioBuffer, AudioClock, Spectrum, MIDI, Graph, Meter — all fields private, accessors only
- [ ] No panics in non-test code (0 unwrap/expect/assert in production paths)
- [ ] All 106+ `unsafe` blocks have `// SAFETY:` comments
- [ ] DSP effects within 0.01 dB of reference implementations
- [ ] SIMD parity verified on x86_64 (SSE2 + AVX2) and aarch64 (NEON)
- [ ] Format conversion: i16, i24, i32, f32, f64, u8 — all with roundtrip tests
- [ ] PipeWire capture/output tested with real hardware
- [ ] 3+ downstream consumers in production
- [ ] 90%+ test coverage
- [ ] docs.rs complete — every public type documented with examples
- [ ] Golden benchmark numbers published
- [ ] Zero clippy warnings
- [ ] Supply chain clean (audit + deny + vet)

---

## Post-v1

### Advanced DSP
- [ ] Convolution reverb (impulse response)
- [ ] Multiband compressor
- [ ] Noise suppression (RNNoise or custom)
- [ ] Pitch shifting (phase vocoder)
- [ ] Time stretching (WSOLA / phase vocoder)

### MIDI advanced
- [ ] SMF (Standard MIDI File) read/write
- [ ] MIDI clock / sync (MTC, SPP)
- [ ] SysEx handling
- [ ] MPE zone management
- [ ] MIDI tokenization for music LLMs

### Platform
- [ ] CoreAudio (macOS)
- [ ] WASAPI (Windows)
- [ ] JACK (pro audio)
- [ ] WASM (Web Audio API)

### High sample rate support
- [ ] Raise sample rate ceiling from 384kHz to 768kHz (for DXD and pro mastering workflows)
- [ ] Validated resampling paths: 44.1k ↔ 48k ↔ 88.2k ↔ 96k ↔ 176.4k ↔ 192k ↔ 352.8k ↔ 384k ↔ 768k
- [ ] Multi-stage resampling for large ratio conversions (e.g. 44.1k → 384k via intermediate stages)
- [ ] Oversampled DSP mode — run effects at 2x/4x internal rate for reduced aliasing
- [ ] Benchmark and optimize sinc resampler for high-rate conversions (64-point kernel at 768kHz)

### Format — niche
- [ ] u8 a-law / u-law (G.711) — telephony codecs, relevant for voice/VoIP pipelines
- [ ] i8 (signed 8-bit) — embedded audio, low-resource targets
- [ ] DSD (1-bit) — SACD / audiophile playback
- [ ] Ambisonic (3D audio) channel layouts

---

## Non-goals

- **Audio I/O (file read/write)** — tarang / symphonia
- **Plugin hosting (VST/CLAP/LV2)** — shruti
- **Music composition / sequencing** — shruti
- **Streaming protocols (RTMP/SRT)** — aethersafta
- **Specific instruments** — shruti; dhvani provides voice management, consumers build on top
