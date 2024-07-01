use crate::{OutputMode, PsgTrait};
use psg_lite::{Output, SoundGenerator};

pub struct PsgWrapper {
    sg: SoundGenerator,
}

impl PsgWrapper {
    pub fn new(clock_rate: u32, sample_rate: u32) -> Self {
        Self {
            sg: SoundGenerator::new(clock_rate, sample_rate),
        }
    }
}

impl PsgTrait for PsgWrapper {
    fn sample_rate(&self) -> u32 {
        self.sg.sample_rate()
    }

    fn clock_rate(&self) -> u32 {
        self.sg.sample_rate()
    }

    fn set_tone_period(&mut self, channel: usize, period: u16) {
        self.sg.set_period(channel, period)
    }

    fn set_volume(&mut self, channel: usize, volume: u8) {
        self.sg.set_volume(channel, volume)
    }

    fn set_output_mode(&mut self, channel: usize, mode: OutputMode) {
        self.sg.set_mode(
            channel,
            match mode {
                OutputMode::Tone => Output::TONE,
                OutputMode::Noise => Output::NOISE,
                OutputMode::ToneNoise => Output::TONE | Output::NOISE,
                _ => todo!(),
            },
        )
    }

    fn set_noise_period(&mut self, period: u8) {
        self.sg.set_noise_period(period)
    }

    fn next_sample_i16(&mut self) -> i16 {
        self.sg.next_sample()
    }

    #[cfg(feature = "float")]
    fn next_sample_f32(&mut self) -> f32 {
        self.sg.next_sample()
    }
}
