# Vibe: Terminal Audio Visualizer

A real-time radial music visualizer for the terminal, written in Rust.

## Overview

Vibe captures system audio and displays a mesmerizing circular visualization where frequency bands radiate outward from the center. Colors shift through the rainbow spectrum — bass in warm reds, treble in cool violets.

**Key characteristics:**
- Captures system audio via WASAPI loopback (Windows)
- Radial/circular visualization style
- Rainbow spectrum colors mapped to frequencies
- Minimal interaction — launch and enjoy

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Audio     │────▶│   FFT       │────▶│  Renderer   │
│   Capture   │     │   Analysis  │     │  (ratatui)  │
└─────────────┘     └─────────────┘     └─────────────┘
     WASAPI           rustfft            terminal
     loopback         + smoothing        output
```

1. **Audio Capture** — Uses `cpal` crate with WASAPI loopback to grab system audio in real-time
2. **FFT Analysis** — Uses `rustfft` to transform samples into frequency data, divided into 64 bands
3. **Renderer** — Uses `ratatui` to draw the radial visualization at ~60fps

## Dependencies

- `cpal` — cross-platform audio capture
- `rustfft` — fast Fourier transform
- `ratatui` + `crossterm` — terminal UI
- `palette` — color interpolation for rainbow gradients

## Visual Design

### Radial Layout

- 64 spokes radiate outward from center, evenly spaced around 360°
- Each spoke represents a frequency band
- Spoke length = intensity of that frequency
- Spoke color = position in spectrum (bass = red → treble = violet)

### Rendering Technique

- Calculate center point based on terminal dimensions
- Draw spokes from center outward using Unicode blocks (█▓▒░)
- Use HSL color interpolation — hue shifts 0° to 300° (red→violet)
- Intensity affects both length and brightness/saturation

### Smoothing

- Exponential moving average on frequency values
- Fast attack (react quickly to beats), slow decay (smooth falloff)
- Prevents flickering, creates satisfying "breathing" effect

## Audio Processing

### Capture

- Enumerate audio devices, find default output's loopback
- Open stream with callback receiving sample buffers
- Push samples into ring buffer shared with main thread

### FFT Analysis

- Collect 2048 samples (~46ms at 44.1kHz)
- Apply Hanning window to reduce spectral leakage
- Run FFT → 1024 frequency bins
- Group into 64 bands using logarithmic scaling

### Band Mapping

| Bands   | Frequency Range | Description          |
|---------|-----------------|----------------------|
| 0-10    | 20Hz - 200Hz    | Sub-bass, bass       |
| 11-25   | 200Hz - 2kHz    | Low-mids, mids       |
| 26-50   | 2kHz - 8kHz     | Upper-mids, presence |
| 51-63   | 8kHz - 16kHz    | Treble, air          |

### Normalization

- Track rolling max amplitude for auto-sensitivity
- Visualization scales regardless of volume level

## Project Structure

```
vibe/
├── Cargo.toml
└── src/
    ├── main.rs          # Entry point, main loop, terminal setup
    ├── audio.rs         # WASAPI capture, ring buffer
    ├── analysis.rs      # FFT, windowing, band grouping
    ├── visualizer.rs    # Radial rendering logic
    └── colors.rs        # Rainbow gradient helpers
```

## Main Loop

```rust
fn main() {
    setup_terminal();
    start_audio_capture();

    loop {
        if key_pressed(Esc) { break; }

        let samples = get_latest_samples();
        let bands = analyze_frequencies(samples);
        let smoothed = apply_smoothing(bands);
        render_frame(smoothed);

        sleep_until_next_frame(); // target 60fps
    }

    cleanup_terminal();
}
```

## Error Handling

- **No audio device found:** Print friendly message, exit gracefully
- **Audio stream error:** Attempt reconnect once, then exit with message
- **Terminal too small:** Show "resize terminal" message instead of crashing
- Panic hook restores terminal state

## Exit

- `Esc`, `q`, or `Ctrl+C` exits cleanly
- Terminal always restored to original state
