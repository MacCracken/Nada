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

## Post-v1 — Synthesis Engines (v2.0 scope)

**Vision**: Dhvani expands from audio engine to complete sound generation platform. All synthesis lives here — consumers (shruti, jalwa, kiran, joshua, vansh, SY) get it for free. The LLM decides *what* to say or play; dhvani handles *how* it sounds. Pure math, no neural network inference in the audio path.

**Integration**: Synthesis engines are provided by [`naad`](https://crates.io/crates/naad) 1.0.0 via the `synthesis` feature flag. Voice synthesis is provided by [`svara`](https://crates.io/crates/svara) 1.0.0 via the `voice` feature flag. Shruti becomes a thin UI/preset/DAW layer over dhvani's synthesis integration.

### Synthesis Engines

| # | Engine | Status | Notes |
|---|--------|--------|-------|
| 1 | **Subtractive synth** | ✅ via naad | `SubtractiveSynth` — oscillators + SVF filter + ADSR |
| 2 | **FM synth** | ✅ via naad | `FmSynth` — multi-operator with algorithm selection |
| 3 | **Additive synth** | ✅ via naad | `AdditiveSynth` — harmonic partials with individual control |
| 4 | **Wavetable synth** | ✅ via naad | `WavetableOscillator` + `MorphWavetable` — morphing between tables |
| 5 | **Physical modeling synth** | ✅ via naad | `KarplusStrong` — plucked string model |
| 6 | **Granular synth** | ✅ via naad | `GranularEngine` — grain cloud with window shapes |
| 7 | **Drum synth** | ✅ via naad | `KickDrum`, `SnareDrum`, `HiHat` — synthetic drum voices |
| 8 | **Sampler engine** | — | Not yet — needs sample loading, key/velocity zones, SFZ/SF2 import |

### Voice Synthesis Engine (v2.0 scope)

**Goal**: Deterministic, real-time voice generation from phoneme sequences. The LLM (hoosh) generates text and intent; dhvani produces the acoustic speech signal. No neural TTS, no vendor lock-in. Pure DSP.

**Why in dhvani**: Voice is sound. Every consumer that needs speech — vansh (voice shell), SY (agent speech), joshua (NPC dialogue), kiran (game characters) — depends on dhvani already. One implementation, audited once, benchmarked once. Personality-driven prosody via bhava modulation is composition, not new code.

| # | Item | Status | Notes |
|---|------|--------|-------|
| 1 | **Formant synthesis** | ✅ via svara | `FormantFilter` — parallel biquad bank, `VowelTarget` with F1–F5 |
| 2 | **Glottal source model** | ✅ via svara | `GlottalSource` — Rosenberg + LF models, Rd voice quality, vibrato, jitter, shimmer |
| 3 | **Noise source** | ✅ via svara | Fricatives, plosives, aspiration handled by phoneme synthesis |
| 4 | **Phoneme sequencer** | ✅ via svara | `PhonemeSequence` — ~50 IPA phonemes, coarticulation, crossfading |
| 5 | **Prosody engine** | ✅ via svara | `ProsodyContour` — f0 contours, stress, intonation patterns |
| 6 | **Bhava integration** | — | Personality→prosody mapping not yet implemented |
| 7 | **Vocoder** | ✅ via naad | `Vocoder` — analysis/synthesis filter bank |
| 8 | **Articulatory modeling** | — | Future — start with formant synthesis (done), graduate when demand justifies |

#### Voice synthesis data flow

```
hoosh (LLM) → "Hello, how are you?" (text)
    ↓ text-to-phoneme (lookup table or rules-based, no ML)
phoneme sequence: [h ɛ l oʊ | h aʊ | ɑːr | j uː]
    ↓ + prosody markers (stress, intonation from intent)
    ↓ + bhava modulation (personality → F0 range, rate, breathiness)
dhvani voice synth:
    ├── glottal source (LF model, F0 from prosody)
    ├── noise source (aspiration, plosives)
    └── formant filter bank (F1-F5 interpolating between phoneme targets)
    ↓ audio samples (f32, sample rate)
dhvani output → speaker / PipeWire / recording
```

#### Consumers

| Consumer | Use Case |
|----------|----------|
| **vansh** | Voice AI shell — TTS output for agnoshi responses. Personality via bhava |
| **SY** (SecureYeoman) | Agent speech — T.Ron, Friday speak with distinct voices shaped by bhava presets |
| **joshua** | NPC dialogue — game characters with personality-driven voices, emotional reactivity |
| **kiran** | Game engine — character voices, narrator, environmental speech |
| **shruti** | Vocoder effect in DAW, voice synthesis as instrument |
| **hoosh** | Audio response mode — speak inference results instead of text |

### Goonj Integration (acoustics engine)

- [ ] **Convolution reverb from goonj IR**: Use `goonj::integration::dhvani::generate_dhvani_ir()` to produce room-specific impulse responses; convolve with dry signal via dhvani DSP chain
- [ ] **Per-band reverb**: Consume `goonj::impulse::MultibandIr` for frequency-dependent convolution (8-band: 63–8000 Hz)
- [ ] **FDN reverb**: Use `goonj::fdn::Fdn` for efficient real-time late reverberation (alternative to convolution)
- [ ] **Ambisonics output**: Use `goonj::ambisonics::BFormatIr` for spatial reverb encoding
- [ ] **WAV IR export**: Use `goonj::wav::write_wav_mono()` to export goonj IRs as WAV files for offline reverb processing
- [ ] **Room presets**: Curate goonj room configurations (concert hall, studio, bathroom, cathedral) as dhvani reverb presets

### Advanced DSP

- [ ] Convolution reverb engine (core DSP — goonj provides the impulse responses, dhvani provides the convolution)
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
- **Music composition / sequencing / timeline** — shruti
- **Streaming protocols (RTMP/SRT)** — aethersafta
- **DAW UI / preset management** — shruti; dhvani provides engines, consumers build UX on top
- **Neural TTS / ML-based voice** — hoosh handles LLM inference; dhvani does deterministic DSP only
- **Text-to-phoneme ML models** — rules-based or lookup table in dhvani; ML phoneme prediction is hoosh territory
