# Dhvani Architecture

> Core audio engine — buffers, DSP, resampling, mixing, analysis, and capture.
>
> **Name**: Dhvani (ध्वनि, Sanskrit) — sound, resonance.
> Extracted from [shruti](https://github.com/MacCracken/shruti) (DAW) as a standalone, reusable crate.

---

## Design Principles

1. **f32 internally** — all processing in 32-bit float; format conversion at I/O boundaries only
2. **Sample-accurate** — clock, mixing, and DSP operate at sample granularity
3. **Zero-allocation hot path** — mixing and DSP reuse buffers; no alloc per frame
4. **SIMD where it matters** — mixing, resampling, gain — the inner loops
5. **PipeWire-native** — first-class Linux audio capture/output (feature-gated)

---

## Module Structure

```
src/
├── lib.rs              Public API, Result type
├── error.rs            NadaError enum
├── buffer/
│   ├── mod.rs          AudioBuffer, SampleFormat, Layout, mix()
│   ├── convert.rs      i16/i32/f32, interleaved/planar, mono/stereo, 5.1 downmix
│   └── resample.rs     Linear + sinc resampling (Blackman-Harris window)
├── dsp/
│   ├── mod.rs          noise_gate, hard_limiter, normalize, dB conversions
│   ├── biquad.rs       BiquadFilter (8 types, Bristow-Johnson cookbook)
│   ├── eq.rs           ParametricEq (N-band cascade)
│   ├── compressor.rs   Compressor (envelope follower, soft knee, makeup gain)
│   ├── limiter.rs      EnvelopeLimiter (brick-wall)
│   ├── reverb.rs       Reverb (Schroeder/Freeverb, 4 combs + 2 allpasses)
│   ├── delay.rs        DelayLine + ModulatedDelay (chorus/flanger)
│   ├── deesser.rs      DeEsser (biquad sidechain)
│   ├── envelope.rs     ADSR envelope generation
│   ├── oscillator.rs   PolyBLEP synthesis (sine, saw, square, triangle, noise)
│   ├── lfo.rs          LFO (6 shapes, sample-and-hold, tempo sync)
│   ├── pan.rs          StereoPanner (constant-power law)
│   └── noise_reduction.rs  Spectral noise reduction (STFT gating)
├── analysis/
│   ├── mod.rs          Spectrum type, spectrum_dft, loudness_lufs, is_silent
│   ├── fft.rs          Radix-2 Cooley-Tukey FFT
│   ├── loudness.rs     EBU R128 (K-weighting, gating, LRA)
│   ├── dynamics.rs     True peak (4x), crest factor, dynamic range
│   ├── chroma.rs       Chromagram (12 pitch classes)
│   ├── onset.rs        Onset detection (spectral flux)
│   ├── stft.rs         STFT spectrograms
│   └── waveform.rs     Downsampled min/max for UI visualization
├── clock/
│   └── mod.rs          AudioClock (position, tempo, beats, PTS, seek)
├── midi/
│   ├── mod.rs          NoteEvent, ControlChange, MidiEvent, MidiClip
│   ├── v2.rs           MIDI 2.0 / UMP types
│   ├── voice.rs        VoiceManager (16-voice pool, 4 steal modes)
│   ├── routing.rs      VelocityCurve, MidiRoute, CcMapping
│   └── translate.rs    MIDI 1.0 ↔ 2.0 conversion
├── graph/
│   └── mod.rs          AudioNode trait, Graph, ExecutionPlan, GraphProcessor
├── meter/
│   └── mod.rs          PeakMeter, MeterBank (lock-free via AtomicU32)
├── capture/
│   ├── mod.rs          CaptureConfig, OutputConfig, AudioDevice
│   ├── pw.rs           PipeWire bindings (feature-gated)
│   └── record.rs       RecordManager, LoopRecordManager (ring-buffer)
├── simd/
│   ├── mod.rs          Platform dispatch (x86_64/aarch64)
│   ├── x86.rs          SSE2 + AVX2 kernels
│   └── aarch64.rs      NEON kernels
├── synthesis/          Synth engines via naad (subtractive, FM, additive, wavetable, granular, drum, vocoder)
├── voice_synth/
│   ├── mod.rs          Voice synthesis via svara (glottal, formant, phoneme, prosody, vocal tract)
│   └── bhava_bridge.rs Personality/mood → voice parameter mapping via bhava
├── creature/           Animal/creature vocals via prani
├── environment/        Nature/environmental sounds via garjan
├── mechanical/         Mechanical sounds via ghurni
├── sampler/            Sample playback via nidhi
├── acoustics/          Room acoustics via goonj (IR, convolution, FDN, ambisonics, presets)
├── g2p/                Grapheme-to-phoneme via shabda (G2P engine, SSML, heteronyms)
├── ffi.rs              C-compatible nada_buffer_* API
└── tests/
    ├── mod.rs          Integration tests
    ├── proptest_tests.rs  Property-based tests
    └── serde_tests.rs  Serialization roundtrip tests
```

---

## Pipeline

```
Input (file, capture, synthesis, MIDI)
    │
    ▼
AudioBuffer (f32 interleaved, channels, sample_rate)
    │
    ├──▶ DSP chain (EQ → compress → gate → limit → reverb → delay)
    │
    ├──▶ Analysis (FFT spectrum, R128 loudness, dynamics, chromagram, onsets)
    │
    ├──▶ Audio graph (topological execution, double-buffered plan swap)
    │
    ├──▶ Mix (sum multiple sources with gain)
    │
    ├──▶ Resample (linear + sinc, 44.1k ↔ 48k ↔ 96k)
    │
    ├──▶ Meter (lock-free peak/RMS via atomics)
    │
    ▼
Output (encode via tarang, play via PipeWire, sync via clock PTS)
```

---

## Key Types

### AudioBuffer
Core sample buffer. Holds f32 interleaved samples with channel count, sample rate, and frame count. Provides peak/RMS/gain/clamp operations.

### AudioClock
Sample-accurate transport. Tracks position in samples, converts to seconds/ms/beats/PTS. Tempo-aware for DAW integration. Generates PTS timestamps for A/V sync with aethersafta.

### Spectrum
FFT magnitude analysis. Provides frequency bins, dominant frequency detection, and per-bin access. Radix-2 Cooley-Tukey FFT (O(n log n)) for production use; simple DFT available for small windows.

---

## Consumers

| Project | Usage |
|---------|-------|
| **shruti** | DAW — all audio math (mix, DSP, analysis, transport, synthesis) |
| **jalwa** | Media player — playback EQ, spectrum visualizer, resampling, normalization |
| **aethersafta** | Compositor — PipeWire capture, audio mixing for streams |
| **kiran** | Game engine — game audio, spatial sound, creature/environment synthesis |
