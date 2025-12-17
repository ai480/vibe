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
