# Dhvani Roadmap

> **Principle**: Correctness first, then SIMD, then capture backends. Every consumer gets the same audio math.

---

## Next — Testing, SIMD Completeness, Docs & Consumer Integration

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

### Analysis additions

- [ ] **Beat/tempo detection**: Autocorrelation of onset function → BPM estimate
- [ ] **Key detection**: Krumhansl-Schmuckler profile matching on existing chromagram output
- [ ] **Zero-crossing rate** — simple feature useful for speech/music discrimination

### DSP additions

- [ ] **SVF Filter (Cytomic topology)** — alternative to biquad, better for modulated synthesis
- [ ] **Sample-accurate automation curves**: Linear/exponential/bezier interpolation between timestamped breakpoints
- [ ] **Channel routing matrix**: NxM routing with per-crosspoint gain

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
