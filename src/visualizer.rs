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
                    let cell = buf.cell_mut((px, py)).unwrap();
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
        assert_eq!(RadialVisualizer::intensity_char(0.5), '▓');
        assert_eq!(RadialVisualizer::intensity_char(1.0), '█');
    }

    #[test]
    fn test_visualizer_creation() {
        let bands = [0.5; NUM_BANDS];
        let viz = RadialVisualizer::new(bands);
        assert_eq!(viz.bands[0], 0.5);
    }
}
