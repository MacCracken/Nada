# Migration Guide

## v0.20.3 (current)

This is the initial release. No migration needed.

## Planned breaking changes for v0.21.3

v0.21.3 is a large hardening + API freeze release. The following changes will require code updates:

### AudioBuffer field encapsulation

Fields will become private. Update direct field access to use accessors:

```rust
// Before (v0.20.3)
let ch = buf.channels;
let sr = buf.sample_rate;
let data = &buf.samples;

// After (v0.21.3)
let ch = buf.channels();
let sr = buf.sample_rate();
let data = buf.samples();
```

### Other type encapsulation

The following types will also have fields made private with read-only accessors:

- `Spectrum` — use `.magnitudes()` instead of `.magnitudes`
- `Chromagram` — use `.chroma()` instead of `.chroma`
- `Voice` — state mutation only through `VoiceManager::note_on()`/`note_off()`
- `MidiRoute` — use validated setters
- `GraphProcessor` — mutation only through `swap_handle()`

### Analysis functions return Result

Analysis functions that previously returned default values on error will return `Result<T, NadaError>`:

```rust
// Before (v0.20.3)
let spec = spectrum_fft(&buf, 4096);        // returns empty Spectrum on error

// After (v0.21.3)
let spec = spectrum_fft(&buf, 4096)?;       // returns Result<Spectrum, NadaError>
```

Affected: `spectrum_fft()`, `stft()`, `measure_r128()`

### New SampleFormat variants

`SampleFormat` gains `I24`, `F64`, `U8` variants. This enum is `#[non_exhaustive]`, so exhaustive matches already require a wildcard arm — no breakage expected.

### Sample rate ceiling raised

Maximum accepted sample rate raised from 384kHz to 768kHz.

## Already completed in v0.20.3

The following planned changes were completed before release:

- `anyhow` dependency removed — `NadaError::Other` now uses `Box<dyn std::error::Error + Send + Sync>`
- `EqBand` struct and `apply_eq_band()` removed — use `EqBandConfig` with `ParametricEq`
- `compress()` free function removed — use `Compressor` struct
