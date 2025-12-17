use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample;
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
