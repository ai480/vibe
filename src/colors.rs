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
