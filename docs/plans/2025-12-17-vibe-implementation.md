# Vibe Terminal Audio Visualizer Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a terminal-based radial music visualizer that captures system audio and displays frequency bands as rainbow-colored spokes.

**Architecture:** Audio capture via WASAPI loopback feeds samples to FFT analysis, which produces 64 frequency bands. A ratatui-based renderer draws these as colored spokes radiating from center at 60fps.

**Tech Stack:** Rust, cpal (audio), rustfft (FFT), ratatui + crossterm (TUI), palette (colors)

---

## Task 1: Project Setup

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`

**Step 1: Initialize Cargo project**

Run:
```bash
cd C:\Users\viktor\Documents\rust
cargo init --name vibe
```

Expected: `Created binary (application) package`

**Step 2: Add dependencies to Cargo.toml**

Replace `Cargo.toml` with:

```toml
[package]
name = "vibe"
version = "0.1.0"
edition = "2021"

[dependencies]
cpal = "0.15"
rustfft = "6.2"
ratatui = "0.29"
crossterm = "0.28"
palette = "0.7"
anyhow = "1.0"
```

**Step 3: Create minimal main.rs**

Replace `src/main.rs` with:

```rust
fn main() {
    println!("vibe starting...");
}
```

**Step 4: Verify build**

Run:
```bash
cargo build
```

Expected: `Compiling vibe v0.1.0` ... `Finished`

**Step 5: Commit**

```bash
git add Cargo.toml src/main.rs Cargo.lock
git commit -m "feat: initialize vibe project with dependencies"
```

---

## Task 2: Color Module

**Files:**
- Create: `src/colors.rs`
- Modify: `src/main.rs`

**Step 1: Create colors module with test**

Create `src/colors.rs`:

```rust
use palette::{Hsl, IntoColor, Srgb};

/// Maps a frequency band index (0-63) to a rainbow color.
/// Band 0 = red (bass), Band 63 = violet (treble).
pub fn band_to_color(band: usize, intensity: f32) -> (u8, u8, u8) {
    let hue = (band as f32 / 64.0) * 300.0; // 0° (red) to 300° (violet)
    let saturation = 0.9;
    let lightness = 0.3 + (intensity.clamp(0.0, 1.0) * 0.4); // 0.3 to 0.7

    let hsl = Hsl::new(hue, saturation, lightness);
    let rgb: Srgb = hsl.into_color();

    (
        (rgb.red * 255.0) as u8,
        (rgb.green * 255.0) as u8,
        (rgb.blue * 255.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_band_0_is_red() {
        let (r, g, b) = band_to_color(0, 0.5);
        assert!(r > 150, "red channel should be high for bass");
        assert!(g < 100, "green channel should be low for bass");
        assert!(b < 100, "blue channel should be low for bass");
    }

    #[test]
    fn test_band_63_is_violet() {
        let (r, g, b) = band_to_color(63, 0.5);
        assert!(r > 100, "red channel present in violet");
        assert!(b > 100, "blue channel should be high for treble");
    }

    #[test]
    fn test_intensity_affects_lightness() {
        let (r1, g1, b1) = band_to_color(32, 0.0);
        let (r2, g2, b2) = band_to_color(32, 1.0);
        let brightness1 = (r1 as u16 + g1 as u16 + b1 as u16) / 3;
        let brightness2 = (r2 as u16 + g2 as u16 + b2 as u16) / 3;
        assert!(brightness2 > brightness1, "higher intensity = brighter");
    }
}
```

**Step 2: Register module in main.rs**

Replace `src/main.rs` with:

```rust
mod colors;

fn main() {
    println!("vibe starting...");

    // Quick test of colors
    let (r, g, b) = colors::band_to_color(0, 0.5);
    println!("Bass color: rgb({}, {}, {})", r, g, b);
}
```

**Step 3: Run tests**

Run:
```bash
cargo test
```

Expected: `test colors::tests::test_band_0_is_red ... ok` (3 tests pass)

**Step 4: Commit**

```bash
git add src/colors.rs src/main.rs
git commit -m "feat: add color module with rainbow gradient"
```

---

## Task 3: FFT Analysis Module

**Files:**
- Create: `src/analysis.rs`
- Modify: `src/main.rs`

**Step 1: Create analysis module with structs and tests**

Create `src/analysis.rs`:

```rust
use rustfft::{num_complex::Complex, FftPlanner};

pub const SAMPLE_SIZE: usize = 2048;
pub const NUM_BANDS: usize = 64;

pub struct Analyzer {
    fft: std::sync::Arc<dyn rustfft::Fft<f32>>,
    window: Vec<f32>,
    smoothed: [f32; NUM_BANDS],
}

impl Analyzer {
    pub fn new() -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(SAMPLE_SIZE);

        // Hanning window
        let window: Vec<f32> = (0..SAMPLE_SIZE)
            .map(|i| {
                0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / SAMPLE_SIZE as f32).cos())
            })
            .collect();

        Self {
            fft,
            window,
            smoothed: [0.0; NUM_BANDS],
        }
    }

    /// Process raw audio samples into frequency bands.
    pub fn process(&mut self, samples: &[f32]) -> [f32; NUM_BANDS] {
        if samples.len() < SAMPLE_SIZE {
            return self.smoothed;
        }

        // Apply window and convert to complex
        let mut buffer: Vec<Complex<f32>> = samples
            .iter()
            .take(SAMPLE_SIZE)
            .zip(self.window.iter())
            .map(|(s, w)| Complex::new(s * w, 0.0))
            .collect();

        // Run FFT
        self.fft.process(&mut buffer);

        // Convert to magnitudes (only first half is useful)
        let magnitudes: Vec<f32> = buffer
            .iter()
            .take(SAMPLE_SIZE / 2)
            .map(|c| c.norm())
            .collect();

        // Group into bands (logarithmic scaling)
        let mut bands = [0.0f32; NUM_BANDS];
        for band in 0..NUM_BANDS {
            let (start, end) = Self::band_range(band, SAMPLE_SIZE / 2);
            if start < end && end <= magnitudes.len() {
                let sum: f32 = magnitudes[start..end].iter().sum();
                let count = (end - start) as f32;
                bands[band] = sum / count;
            }
        }

        // Normalize
        let max = bands.iter().cloned().fold(0.0f32, f32::max);
        if max > 0.0 {
            for band in &mut bands {
                *band /= max;
            }
        }

        // Smooth (fast attack, slow decay)
        for i in 0..NUM_BANDS {
            if bands[i] > self.smoothed[i] {
                self.smoothed[i] = self.smoothed[i] * 0.3 + bands[i] * 0.7; // fast attack
            } else {
                self.smoothed[i] = self.smoothed[i] * 0.85 + bands[i] * 0.15; // slow decay
            }
        }

        self.smoothed
    }

    /// Get frequency bin range for a band (logarithmic distribution).
    fn band_range(band: usize, total_bins: usize) -> (usize, usize) {
        let min_freq = 20.0f32;
        let max_freq = 16000.0f32;
        let sample_rate = 44100.0f32;

        let freq_per_bin = sample_rate / (total_bins as f32 * 2.0);

        let log_min = min_freq.ln();
        let log_max = max_freq.ln();
        let log_range = log_max - log_min;

        let freq_start = (log_min + (band as f32 / NUM_BANDS as f32) * log_range).exp();
        let freq_end = (log_min + ((band + 1) as f32 / NUM_BANDS as f32) * log_range).exp();

        let bin_start = (freq_start / freq_per_bin) as usize;
        let bin_end = (freq_end / freq_per_bin) as usize;

        (bin_start.min(total_bins), bin_end.min(total_bins))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = Analyzer::new();
        assert_eq!(analyzer.window.len(), SAMPLE_SIZE);
    }

    #[test]
    fn test_process_silence() {
        let mut analyzer = Analyzer::new();
        let silence = vec![0.0f32; SAMPLE_SIZE];
        let bands = analyzer.process(&silence);
        for band in bands.iter() {
            assert!(*band >= 0.0 && *band <= 1.0);
        }
    }

    #[test]
    fn test_process_sine_wave() {
        let mut analyzer = Analyzer::new();
        // Generate 440Hz sine wave
        let samples: Vec<f32> = (0..SAMPLE_SIZE)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin())
            .collect();
        let bands = analyzer.process(&samples);

        // Should have some energy in the mid-range bands
        let has_energy = bands.iter().any(|&b| b > 0.1);
        assert!(has_energy, "sine wave should produce visible energy");
    }

    #[test]
    fn test_band_range_covers_spectrum() {
        let total_bins = SAMPLE_SIZE / 2;
        let (start_first, _) = Analyzer::band_range(0, total_bins);
        let (_, end_last) = Analyzer::band_range(NUM_BANDS - 1, total_bins);

        assert!(start_first < 10, "first band should start near beginning");
        assert!(end_last > 100, "last band should extend into higher bins");
    }
}
```

**Step 2: Register module in main.rs**

Update `src/main.rs`:

```rust
mod analysis;
mod colors;

fn main() {
    println!("vibe starting...");

    // Test analyzer
    let mut analyzer = analysis::Analyzer::new();
    let silence = vec![0.0f32; analysis::SAMPLE_SIZE];
    let bands = analyzer.process(&silence);
    println!("Bands from silence: first={:.2}, last={:.2}", bands[0], bands[63]);
}
```

**Step 3: Run tests**

Run:
```bash
cargo test
```

Expected: All tests pass (7 total now)

**Step 4: Commit**

```bash
git add src/analysis.rs src/main.rs
git commit -m "feat: add FFT analysis module with smoothing"
```

---

## Task 4: Audio Capture Module

**Files:**
- Create: `src/audio.rs`
- Modify: `src/main.rs`

**Step 1: Create audio capture module**

Create `src/audio.rs`:

```rust
use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

use crate::analysis::SAMPLE_SIZE;

pub struct AudioCapture {
    buffer: Arc<Mutex<Vec<f32>>>,
    _stream: cpal::Stream,
}

impl AudioCapture {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();

        // Try to get loopback device (system audio)
        let device = Self::find_loopback_device(&host)?;
        let config = device.default_output_config()?;

        let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::with_capacity(SAMPLE_SIZE * 2)));
        let buffer_clone = buffer.clone();

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => Self::build_stream::<f32>(&device, &config.into(), buffer_clone)?,
            cpal::SampleFormat::I16 => Self::build_stream::<i16>(&device, &config.into(), buffer_clone)?,
            cpal::SampleFormat::U16 => Self::build_stream::<u16>(&device, &config.into(), buffer_clone)?,
            _ => return Err(anyhow!("Unsupported sample format")),
        };

        stream.play()?;

        Ok(Self {
            buffer,
            _stream: stream,
        })
    }

    fn find_loopback_device(host: &cpal::Host) -> Result<cpal::Device> {
        // On Windows, look for loopback device
        for device in host.output_devices()? {
            if let Ok(name) = device.name() {
                // Windows WASAPI loopback devices often have "Loopback" in name
                // or we can use any output device as loopback on supported hosts
                if name.to_lowercase().contains("loopback") {
                    return Ok(device);
                }
            }
        }

        // Fall back to default output device (works on Windows WASAPI)
        host.default_output_device()
            .ok_or_else(|| anyhow!("No output device found"))
    }

    fn build_stream<T>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        buffer: Arc<Mutex<Vec<f32>>>,
    ) -> Result<cpal::Stream>
    where
        T: cpal::Sample + cpal::SizedSample,
        f32: cpal::FromSample<T>,
    {
        let channels = config.channels as usize;

        let stream = device.build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                let mut buf = buffer.lock().unwrap();

                // Convert to mono f32
                for frame in data.chunks(channels) {
                    let sum: f32 = frame.iter().map(|s| f32::from_sample(*s)).sum();
                    let mono = sum / channels as f32;
                    buf.push(mono);
                }

                // Keep buffer size manageable
                if buf.len() > SAMPLE_SIZE * 4 {
                    buf.drain(0..SAMPLE_SIZE * 2);
                }
            },
            |err| eprintln!("Audio stream error: {}", err),
            None,
        )?;

        Ok(stream)
    }

    /// Get latest samples for analysis.
    pub fn get_samples(&self) -> Vec<f32> {
        let buf = self.buffer.lock().unwrap();
        if buf.len() >= SAMPLE_SIZE {
            buf[buf.len() - SAMPLE_SIZE..].to_vec()
        } else {
            buf.clone()
        }
    }
}
```

**Step 2: Update main.rs to test audio**

Update `src/main.rs`:

```rust
mod analysis;
mod audio;
mod colors;

fn main() {
    println!("vibe starting...");

    // Test audio capture
    match audio::AudioCapture::new() {
        Ok(capture) => {
            println!("Audio capture initialized!");
            std::thread::sleep(std::time::Duration::from_millis(500));
            let samples = capture.get_samples();
            println!("Got {} samples", samples.len());
        }
        Err(e) => {
            eprintln!("Failed to initialize audio: {}", e);
        }
    }
}
```

**Step 3: Test build and run**

Run:
```bash
cargo build
cargo run
```

Expected: "Audio capture initialized!" and sample count (may vary based on system)

**Step 4: Commit**

```bash
git add src/audio.rs src/main.rs
git commit -m "feat: add WASAPI audio capture module"
```

---

## Task 5: Visualizer Module

**Files:**
- Create: `src/visualizer.rs`
- Modify: `src/main.rs`

**Step 1: Create visualizer with radial rendering**

Create `src/visualizer.rs`:

```rust
use crate::analysis::NUM_BANDS;
use crate::colors::band_to_color;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

pub struct RadialVisualizer {
    bands: [f32; NUM_BANDS],
}

impl RadialVisualizer {
    pub fn new(bands: [f32; NUM_BANDS]) -> Self {
        Self { bands }
    }
}

impl Widget for RadialVisualizer {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let center_x = area.x + area.width / 2;
        let center_y = area.y + area.height / 2;

        // Max radius is limited by smallest dimension
        // Account for character aspect ratio (chars are ~2x tall as wide)
        let max_radius_x = (area.width / 2) as f32;
        let max_radius_y = (area.height / 2) as f32 * 2.0; // Adjust for aspect ratio

        let max_radius = max_radius_x.min(max_radius_y) * 0.9;

        // Draw each band as a spoke
        for band in 0..NUM_BANDS {
            let angle = (band as f32 / NUM_BANDS as f32) * 2.0 * std::f32::consts::PI;
            let intensity = self.bands[band];
            let length = max_radius * (0.2 + intensity * 0.8); // Min 20% length

            let (r, g, b) = band_to_color(band, intensity);
            let color = Color::Rgb(r, g, b);

            // Draw spoke from center outward
            let steps = (length as usize).max(1);
            for step in 0..steps {
                let ratio = step as f32 / length;
                let x = center_x as f32 + angle.cos() * ratio * length;
                let y = center_y as f32 - angle.sin() * ratio * length / 2.0; // Adjust for aspect

                let px = x.round() as u16;
                let py = y.round() as u16;

                if px >= area.x && px < area.x + area.width && py >= area.y && py < area.y + area.height {
                    let cell = buf.get_mut(px, py);
                    cell.set_char(Self::intensity_char(intensity));
                    cell.set_style(Style::default().fg(color));
                }
            }
        }
    }
}

impl RadialVisualizer {
    fn intensity_char(intensity: f32) -> char {
        match (intensity * 4.0) as usize {
            0 => '░',
            1 => '▒',
            2 => '▓',
            _ => '█',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intensity_char() {
        assert_eq!(RadialVisualizer::intensity_char(0.0), '░');
        assert_eq!(RadialVisualizer::intensity_char(0.5), '▒');
        assert_eq!(RadialVisualizer::intensity_char(1.0), '█');
    }

    #[test]
    fn test_visualizer_creation() {
        let bands = [0.5; NUM_BANDS];
        let viz = RadialVisualizer::new(bands);
        assert_eq!(viz.bands[0], 0.5);
    }
}
```

**Step 2: Register module**

Update `src/main.rs`:

```rust
mod analysis;
mod audio;
mod colors;
mod visualizer;

fn main() {
    println!("vibe starting...");
    println!("Modules loaded successfully!");
}
```

**Step 3: Run tests**

Run:
```bash
cargo test
```

Expected: All tests pass (9 total)

**Step 4: Commit**

```bash
git add src/visualizer.rs src/main.rs
git commit -m "feat: add radial visualizer widget"
```

---

## Task 6: Main Loop Integration

**Files:**
- Modify: `src/main.rs`

**Step 1: Implement full main loop**

Replace `src/main.rs` with:

```rust
mod analysis;
mod audio;
mod colors;
mod visualizer;

use std::io::{self, stdout};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;

use analysis::Analyzer;
use audio::AudioCapture;
use visualizer::RadialVisualizer;

const TARGET_FPS: u64 = 60;
const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / TARGET_FPS);

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Setup panic hook to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        let _ = disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
        original_hook(panic);
    }));

    // Initialize audio and analyzer
    let capture = match AudioCapture::new() {
        Ok(c) => c,
        Err(e) => {
            cleanup_terminal()?;
            eprintln!("Failed to initialize audio capture: {}", e);
            eprintln!("Make sure audio is playing on your system.");
            return Ok(());
        }
    };

    let mut analyzer = Analyzer::new();

    // Main loop
    let result = run_loop(&mut terminal, &capture, &mut analyzer);

    // Cleanup
    cleanup_terminal()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    capture: &AudioCapture,
    analyzer: &mut Analyzer,
) -> Result<()> {
    loop {
        let frame_start = Instant::now();

        // Check for quit
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => break,
                        _ => {}
                    }
                }
            }
        }

        // Get audio and analyze
        let samples = capture.get_samples();
        let bands = analyzer.process(&samples);

        // Render
        terminal.draw(|frame| {
            let area = frame.area();

            // Check minimum size
            if area.width < 40 || area.height < 20 {
                let msg = "Terminal too small. Please resize.";
                let x = area.width.saturating_sub(msg.len() as u16) / 2;
                let y = area.height / 2;
                frame.buffer_mut().set_string(x, y, msg, Style::default());
                return;
            }

            let viz = RadialVisualizer::new(bands);
            frame.render_widget(viz, area);
        })?;

        // Frame timing
        let elapsed = frame_start.elapsed();
        if elapsed < FRAME_DURATION {
            std::thread::sleep(FRAME_DURATION - elapsed);
        }
    }

    Ok(())
}

fn cleanup_terminal() -> Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
```

**Step 2: Build and test run**

Run:
```bash
cargo build --release
```

Expected: Successful build

**Step 3: Test manually**

Run:
```bash
cargo run --release
```

Expected: Visualizer appears. Press `q` or `Esc` to quit.

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: integrate main loop with audio, analysis, and rendering"
```

---

## Task 7: Final Polish and README

**Files:**
- Create: `README.md`

**Step 1: Create README**

Create `README.md`:

```markdown
# Vibe

A terminal-based radial music visualizer written in Rust.

## Features

- Real-time system audio capture (Windows WASAPI)
- FFT frequency analysis with 64 bands
- Radial visualization with rainbow colors
- Smooth animations at 60fps

## Usage

```bash
cargo run --release
```

Play music on your system and watch the visualization react.

**Controls:**
- `q` or `Esc` - Quit
- `Ctrl+C` - Quit

## Requirements

- Windows (uses WASAPI for audio capture)
- A terminal that supports Unicode and true color (Windows Terminal, etc.)

## Building

```bash
cargo build --release
```

The binary will be at `target/release/vibe.exe`.
```

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add README with usage instructions"
```

---

## Summary

**Tasks completed:**
1. Project setup with dependencies
2. Color module with rainbow gradients
3. FFT analysis with smoothing
4. WASAPI audio capture
5. Radial visualizer widget
6. Main loop integration
7. README documentation

**To run:**
```bash
cargo run --release
```

Play music and enjoy the visualization!
