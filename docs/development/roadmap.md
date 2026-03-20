# Nada Roadmap

> **Principle**: Correctness first, then SIMD, then capture backends. Every consumer gets the same audio math.

Completed items are in [CHANGELOG.md](../../CHANGELOG.md).

---

## v0.20.3 — Foundation

- [x] AudioBuffer (f32 interleaved, channels, sample_rate, frames)
- [x] SampleFormat enum (F32, I16, I32) with bytes_per_sample
- [x] Buffer operations: peak, RMS, gain, clamp, silence
- [x] Mixing: sum N buffers with channel/rate validation
- [x] Resampling: linear interpolation (44.1k ↔ 48k ↔ 96k)
- [x] DSP: noise gate, hard limiter, compressor, normalize
- [x] DSP: amplitude ↔ dB conversion
- [x] Analysis: DFT spectrum, dominant frequency detection
- [x] Analysis: LUFS loudness (simplified), silence detection
- [x] AudioClock: sample-accurate position, tempo, beats, PTS, seek
- [x] 40+ tests, 3 benchmark suites

---

## v0.21.3 — DSP & Format Conversion (current)

### DSP effects
- [x] Biquad filter (low-pass, high-pass, band-pass, notch, all-pass, peaking, shelf)
- [x] Parametric EQ (N-band biquad cascade)
- [x] Reverb (Schroeder/Freeverb: 4 combs + 2 allpasses, stereo decorrelation)
- [x] Delay line (fixed + modulated for chorus/flanger)
- [x] De-esser (sibilance reduction via biquad sidechain)
- [x] Compressor upgrade (envelope follower, soft knee, makeup gain)

### Format conversion
- [x] i16 ↔ f32 interleaved conversion
- [x] i32 ↔ f32 interleaved conversion
- [x] Interleaved ↔ planar conversion
- [x] Mono → stereo (duplicate) and stereo → mono (sum/average)
- [x] 5.1 → stereo downmix (ITU-R BS.775)

### Resampling
- [x] Sinc resampling (windowed sinc with Blackman-Harris window)
- [x] Configurable quality levels (draft/good/best)

### Crate quality
- [x] `#[non_exhaustive]` on enums (NadaError, SampleFormat, Layout, FilterType, etc.)
- [x] Enhanced lib.rs docs with usage guide, feature table, examples
- [x] `#[serde(default)]` on parameter structs for forward compatibility
- [x] 100+ tests, 6 benchmark suites (mix, resample, dsp, convert)

---

## v0.22.3 — SIMD, Capture & Crate Quality (current)

### SIMD acceleration
- [x] SSE2 mixing, gain, clamp, peak, RMS, noise gate (4 samples/iter)
- [x] AVX2 mixing, gain, clamp, peak (8 samples/iter, runtime-detected)
- [x] NEON mixing, gain, clamp, peak, RMS, noise gate (aarch64)
- [x] SIMD gain application (apply_gain, hard_limiter)
- [x] SIMD format conversion (i16↔f32)
- [x] Platform dispatch module (src/simd/) with scalar fallback
- [ ] SIMD resampling inner loop
- [ ] Benchmarks: SIMD vs scalar per operation

### PipeWire capture (requires `pipewire` feature)
- [ ] Device enumeration (sources, sinks)
- [ ] Per-source audio capture (mic, desktop, per-app)
- [ ] Capture → AudioBuffer conversion
- [ ] Output to PipeWire sink
- [ ] Hot-plug device detection

### Crate quality (inspired by ai-hwaccel)
- [ ] CONTRIBUTING.md
- [ ] SECURITY.md
- [ ] deny.toml (license/advisory/source validation)
- [ ] cargo-vet in CI
- [ ] Fuzz targets (mix, resample, DSP chain)
- [ ] Complete docs.rs documentation (all public types + examples)
- [ ] FFI module (C-compatible API for key types)
- [ ] cargo-semver-checks in CI

---

## v0.23.3 — MIDI, RT Infrastructure & DSP Gaps

### MIDI foundation (`midi` module)

Port from shruti's battle-tested implementation (`shruti-session/src/midi.rs`,
`shruti-instruments/src/voice.rs`, `shruti-instruments/src/routing.rs`) and
improve upon it. Nada becomes the canonical MIDI crate for the ecosystem —
shruti, hoosh, jalwa, and tarang all consume `nada::midi`.

#### Core types (`midi`)
- [ ] `NoteEvent` — note on/off with position (u64 frames), duration, note 0-127, velocity 0-127, channel 0-15
- [ ] `ControlChange` — CC number, value, channel, position
- [ ] `MidiClip` — sorted note/CC container with `add_note()`, `add_cc()`, `notes_at()`, `note_ons_at()`, `note_offs_at()`
- [ ] `MidiEvent` enum — unified event type (NoteOn, NoteOff, CC, PitchBend, Aftertouch, ProgramChange)
      _Improvement: shruti has separate structs; nada unifies into a single enum for cleaner pattern matching_

#### MIDI 2.0 / UMP (`midi::v2`)
- [ ] `NoteOnV2`, `NoteOffV2` — 16-bit velocity, per-note attributes (type + data)
- [ ] `ControlChangeV2` — 32-bit value (full resolution)
- [ ] `PerNotePitchBend`, `PerNoteController` — MPE support
- [ ] `ChannelPressureV2`, `PolyPressureV2`, `PitchBendV2` — 32-bit resolution
- [ ] `UmpMessageType` enum — Utility, SystemCommon, Midi1ChannelVoice, Data64, Midi2ChannelVoice, Data128

#### MIDI 1.0 ↔ 2.0 translation (`midi::translate`)
- [ ] `velocity_7_to_16()` / `velocity_16_to_7()` — MIDI 2.0 spec scaling (v << 9 | v << 2 | v >> 5)
- [ ] `cc_7_to_32()` / `cc_32_to_7()` — full 32-bit CC range
- [ ] `pitch_bend_14_to_32()` / `pitch_bend_32_to_14()`
- [ ] `note_event_to_v2()` / `note_on_v2_to_event()` — event conversion
- [ ] `cc_to_v2()` / `cc_v2_to_cc()` — CC conversion
- [ ] Roundtrip tests for all conversions (port shruti's 10+ tests)

#### Voice management (`midi::voice`)
- [ ] `VoiceState` — Idle, Active, Releasing
- [ ] `Voice` — per-voice state: note, velocity, channel, envelope_level, age, pitch_bend, pressure, brightness
- [ ] `Voice::frequency()` — MIDI note → Hz (12-TET, A4=440)
- [ ] `Voice::apply_per_note_cc()` — CC#74 brightness routing
- [ ] `VoiceStealMode` — Oldest, Quietest, Lowest, None
- [ ] `VoiceManager` — polyphonic voice pool with allocation, stealing, release, age tracking
      _Improvement: decouple oscillator state (phase accumulators) from voice — nada provides voice management, consumers own synthesis state_

#### Routing & mapping (`midi::routing`)
- [ ] `VelocityCurve` — Linear, Soft (sqrt), Hard (square), Fixed(u8)
- [ ] `MidiRoute` — channel filter, note range, velocity curve, `filter_event()`
      _Improvement: make MidiRoute generic over event type (not tied to track UUIDs) so it works outside DAW context_
- [ ] `CcMapping` — CC number → parameter range mapping with `map_value()` (7-bit) and `map_value_32()` (32-bit)

#### Improvements over shruti's implementation
- [ ] `MidiClip::events_in_range(start, end)` — range query using binary search (shruti scans linearly)
- [ ] `MidiClip::merge(other)` — combine clips maintaining sort order
- [ ] `MidiClip::transpose(semitones)` — shift all notes
- [ ] `MidiClip::quantize(grid_frames)` — snap positions to grid
- [ ] `#[non_exhaustive]` on all enums for forward compat
- [ ] Property-based tests (proptest: random events, roundtrip translation, sort invariants)

### RT infrastructure (from shruti-engine)

Generic real-time building blocks that every audio app needs.

#### Lock-free metering (`meter`)
- [ ] `PeakMeter` — stereo peak levels via `AtomicU32` (f32 bit patterns, no mutex)
- [ ] `MeterBank` — growable slot bank, pre-allocated
- [ ] `SharedMeterBank` — `Arc`-wrapped for multi-thread sharing
      _Source: shruti-engine/src/meter.rs — production-proven, extract as-is_

#### Audio graph (`graph`)
- [ ] `AudioNode` trait — name, num_inputs, num_outputs, `process()`, `is_finished()`
- [ ] `Graph` — non-RT builder: add nodes, connect edges
- [ ] `ExecutionPlan` — compiled topological order (Kahn's algorithm, cycle detection)
- [ ] `GraphProcessor` — RT-thread processor with double-buffered plan swapping via `try_lock()` fallback
- [ ] `NodeId` — atomic ID generator
      _Source: shruti-engine/src/graph/ — extract core traits/planner, leave concrete nodes in consumers_

#### Ring-buffer recording (`capture`)
- [ ] `RecordManager` — lock-free ring buffer (rtrb) → accumulator thread → output
- [ ] `LoopRecordManager` — loop-aware recording with sentinel-based take splitting
      _Source: shruti-engine/src/record.rs — generic RT→disk pipeline_

### DSP gaps (from shruti-dsp)
- [ ] `StereoPanner` — constant-power panning (shruti-dsp `effects/pan.rs`)
- [ ] `EnvelopeLimiter` — soft-knee limiter with envelope follower (shruti-dsp `effects/limiter.rs`)
- [ ] `DynamicsAnalysis` — peak, RMS, true peak, crest factor, dynamic range (shruti-dsp `analysis/dynamics.rs`)
- [ ] FFT spectrum — radix-2 FFT replacing O(n²) DFT (shruti-dsp `analysis/spectral.rs`)

---

## v0.24.3 — Integration & Performance

### Consumer integration
- [ ] shruti adopts nada (replace shruti-engine audio math, shruti-dsp, shruti-session MIDI types)
- [ ] jalwa adopts nada (replace internal playback buffer + EQ)
- [ ] aethersafta adopts nada (replace PipeWire capture + mixer stub)
- [ ] hoosh uses `nada::midi` for music token preprocessing

### Performance
- [ ] Zero-copy buffer views (borrow slices for read-only DSP)
- [ ] Buffer pool (reuse allocations across frames — arena allocator)
- [ ] Parallel DSP chain (rayon for independent effects)

### Analysis
- [ ] STFT (short-time Fourier transform) for spectrograms
- [ ] Full EBU R128 loudness (K-weighting + gating)
- [ ] Chromagram (pitch class distribution)
- [ ] Onset detection (transient analysis)

### Quality
- [ ] Property-based tests (proptest: random buffers, sample rates, channels)
- [ ] 90%+ code coverage

---

## v1.0.0 Criteria

- [ ] AudioBuffer, AudioClock, Spectrum, MIDI APIs frozen
- [ ] All DSP effects match reference implementations (within 0.01 dB)
- [ ] SIMD on x86_64 (SSE2+AVX2) and aarch64 (NEON)
- [ ] PipeWire capture/output stable
- [ ] Sinc resampling passing SRC quality tests
- [ ] MIDI 1.0 + 2.0 types, translation, voice management stable
- [ ] At least 3 downstream consumers (shruti, jalwa, aethersafta)
- [ ] 90%+ test coverage
- [ ] docs.rs documentation complete
- [ ] No `unsafe` without `// SAFETY:` comments
- [ ] Benchmarks establish golden numbers

---

## Post-v1

### Advanced DSP
- [ ] Convolution reverb (impulse response loading)
- [ ] Multiband compressor
- [ ] Noise suppression (RNNoise integration or custom)
- [ ] Pitch shifting (phase vocoder)
- [ ] Time stretching (WSOLA or phase vocoder)

### MIDI advanced
- [ ] SMF (Standard MIDI File) read/write
- [ ] MIDI clock / sync (MTC, SPP)
- [ ] SysEx message handling
- [ ] MPE (MIDI Polyphonic Expression) zone management
- [ ] MIDI tokenization for music LLMs (port from shruti-ml `tokenizer.rs`)

### Platform
- [ ] CoreAudio backend (macOS)
- [ ] WASAPI backend (Windows)
- [ ] JACK backend (pro audio)
- [ ] WASM target (Web Audio API)

### Format
- [ ] 24-bit audio support
- [ ] DSD (1-bit) support
- [ ] Ambisonic (3D audio) channel layouts

---

## Non-goals

- **Audio I/O (file read/write)** — that's tarang (decode) and symphonia (pure Rust decode)
- **Plugin hosting (VST/CLAP/LV2)** — that's shruti
- **Music composition / sequencing** — that's shruti
- **Streaming protocols (RTMP/SRT)** — that's aethersafta
- **Specific instruments (synth/sampler/drums)** — that's shruti; nada provides voice management, consumers build instruments on top
