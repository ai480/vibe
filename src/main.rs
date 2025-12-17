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
