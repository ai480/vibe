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
