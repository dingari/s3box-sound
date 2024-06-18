#![no_std]

use micromath::F32Ext;
use core::iter::Iterator;

fn linlin(x: f32, irange: (f32, f32), orange: (f32, f32)) -> f32 {
    let (imin, imax) = irange;
    let (omin, omax) = orange;

    (x - imin) / (imax - imin) * (omax - omin) + omin
}

#[allow(dead_code)]
fn sawtooth(t: f32, f: f32, sr: f32) -> f32 {
    let tp = (t / sr) * f;
    2.0 * (tp - (0.5 + tp).floor())
}

fn sine(t: f32, f: f32, sr: f32) -> f32 {
    (2.0 * core::f32::consts::PI * t * f / sr).sin()
}

pub struct SampleProcessor {
    pub sample_rate: f32,
    pub freq: f32,
    pub mod_freq: f32,
    pub num_processed_samples: usize,
}


/// Just a simple demo audio processor
/// Will produce a sine wave of the given `freq` on the left channel
/// and an AM modulated sine wave on the right channel
impl SampleProcessor {
    pub fn new(sample_rate: f32, freq: f32, mod_freq: f32) -> Self {
        Self {
            sample_rate,
            freq,
            mod_freq,
            num_processed_samples: 0,
        }
    }

    pub fn process_samples(&mut self, buffer: &mut [i16], num_channels: usize) {
        let num_samples = buffer.len();
        let to_i16 = |sample: f32| (sample * i16::MAX as f32) as i16;

        for i in (0..num_samples).step_by(num_channels) {
            let (sample_rate, freq, mod_freq) = (self.sample_rate, self.freq, self.mod_freq);
            let t = (self.num_processed_samples + i) as f32;
            let m = linlin(sine(t, mod_freq, sample_rate), (-1.0, 1.0), (0.0, 1.0));

            let (left, right) = (
                sine(t, freq, sample_rate),
                sine(t, freq, sample_rate) * m,
            );

            buffer[i] = to_i16(left);
            buffer[i + 1] = to_i16(right);
        }

        self.num_processed_samples += num_samples;
    }
}
