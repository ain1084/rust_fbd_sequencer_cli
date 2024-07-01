use crate::{OutputMode, PsgTrait};
use psg::PSG;

pub struct PsgWrapper {
    sg: PSG,
    clock_rate: u32,
    sample_rate: u32,
}

impl PsgWrapper {
    pub fn new(clock_rate: u32, sample_rate: u32) -> Self {
        Self {
            clock_rate,
            sample_rate,
            sg: PSG::new(clock_rate as f64, sample_rate).unwrap(),
        }
    }
}

impl PsgTrait for PsgWrapper {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn clock_rate(&self) -> u32 {
        self.clock_rate
    }

    fn set_tone_period(&mut self, channel: usize, period: u16) {
        self.sg.set_tone_period(channel as u8, period)
    }

    fn set_volume(&mut self, channel: usize, volume: u8) {
        self.sg.set_amplitude(channel as u8, volume)
    }

    fn set_output_mode(&mut self, channel: usize, mode: OutputMode) {
        self.sg.set_tone_disabled(
            channel as u8,
            mode == OutputMode::Noise || mode == OutputMode::None,
        );
        self.sg.set_noise_disabled(
            channel as u8,
            mode == OutputMode::Tone || mode == OutputMode::None,
        );
    }

    fn set_noise_period(&mut self, period: u8) {
        self.sg.set_noise_period(period)
    }

    fn next_sample_i16(&mut self) -> i16 {
        let (l, r) = self.sg.render();
        (((l + r) / 2.0) * i16::MAX as f64) as i16
    }

    #[cfg(feature = "float")]
    fn next_sample_f32(&mut self) -> f32 {
        let (l, r) = self.sg.render();
        ((l + r) / 2.0) as f32
    }
}
