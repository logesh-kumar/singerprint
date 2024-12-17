use std::error::Error;
use rustfft::FftPlanner;
use rustfft::num_complex::Complex;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use crate::fingerprint::AudioFingerprint;

pub struct AudioProcessor {
    pub sample_rate: u32,
    pub window_size: usize,
    pub overlap: usize,
}

impl AudioProcessor {
    pub fn new(sample_rate: u32) -> Self {
        AudioProcessor {
            sample_rate,
            window_size: 2048,  // Typical FFT window size
            overlap: 1024,      // 50% overlap between windows
        }
    }

    pub fn process_audio(&self, audio_data: &[f32]) -> Result<AudioFingerprint, Box<dyn Error>> {
        // Create FFT planner
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(self.window_size);
        
        // Process audio in overlapping windows
        let mut spectrogram = Vec::new();
        let mut i = 0;
        
        while i < audio_data.len() - self.window_size {
            let mut window = vec![0.0; self.window_size];
            
            // Apply Hanning window
            for j in 0..self.window_size {
                let sample = audio_data[i + j];
                let hanning = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * j as f32 
                    / self.window_size as f32).cos());
                window[j] = sample * hanning;
            }
            
            // Perform FFT
            let mut spectrum: Vec<Complex<f32>> = window.iter()
                .map(|&x| Complex::new(x, 0.0))
                .collect();
            fft.process(&mut spectrum);
            
            // Extract magnitude spectrum
            let magnitudes: Vec<f32> = spectrum.iter()
                .take(self.window_size / 2)
                .map(|c| c.norm())
                .collect();
                
            spectrogram.push(magnitudes);
            i += self.overlap;
        }
        
        // Find peaks in the spectrogram
        let peaks = self.find_peaks(&spectrogram);
        
        // Generate hash from peaks
        let hash = self.generate_hash(&peaks);
        
        Ok(AudioFingerprint { peaks, hash })
    }
    
    pub fn find_peaks(&self, spectrogram: &[Vec<f32>]) -> Vec<(f32, f32)> {
        let mut peaks = Vec::new();
        let neighborhood_size = 10; // Size of the neighborhood to check for local maxima
        
        for t in neighborhood_size..(spectrogram.len() - neighborhood_size) {
            for f in neighborhood_size..(spectrogram[t].len() - neighborhood_size) {
                if self.is_local_maximum(spectrogram, f, t) {
                    let freq = f as f32 * self.sample_rate as f32 / (2.0 * self.window_size as f32);
                    let time = t as f32 * self.overlap as f32 / self.sample_rate as f32;
                    peaks.push((freq, time));
                }
            }
        }
        peaks
    }

    pub fn is_local_maximum(&self, spectrogram: &[Vec<f32>], f: usize, t: usize) -> bool {
        let current = spectrogram[t][f];
        let threshold = 0.1; // Minimum amplitude threshold
        
        if current < threshold {
            return false;
        }

        // Check neighborhood
        for dt in -3..=3 {
            for df in -3..=3 {
                if dt == 0 && df == 0 {
                    continue;
                }
                
                let t_idx = (t as i32 + dt) as usize;
                let f_idx = (f as i32 + df) as usize;
                
                if spectrogram[t_idx][f_idx] >= current {
                    return false;
                }
            }
        }
        true
    }

    pub fn generate_hash(&self, peaks: &[(f32, f32)]) -> Vec<u64> {
        let mut hashes = Vec::new();
        let fan_out = 5; // Number of target points to pair with each anchor point
        
        for (i, &anchor) in peaks.iter().enumerate() {
            for j in 1..=fan_out {
                if i + j >= peaks.len() {
                    break;
                }
                
                let target = peaks[i + j];
                let time_delta = target.1 - anchor.1;
                
                // Create hash using anchor frequency, target frequency, and time delta
                let hash = {
                    let mut h = DefaultHasher::new();
                    (anchor.0 as u32).hash(&mut h);
                    (target.0 as u32).hash(&mut h);
                    (time_delta as u32).hash(&mut h);
                    h.finish()
                };
                
                hashes.push(hash);
            }
        }
        hashes
    }
}