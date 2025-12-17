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
