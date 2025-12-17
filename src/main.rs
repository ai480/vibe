mod colors;

fn main() {
    println!("vibe starting...");

    // Quick test of colors
    let (r, g, b) = colors::band_to_color(0, 0.5);
    println!("Bass color: rgb({}, {}, {})", r, g, b);
}
