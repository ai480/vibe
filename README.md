# Vibe

A terminal-based radial music visualizer written in Rust.

![Vibe Visualization](assets/Visualisation.gif)

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
