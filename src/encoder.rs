use crate::{Channels, Error};

pub struct Encoder {
    sample_rate: u32,
    bit_rate: u32,
    channels: Channels,
}

impl Encoder {
    pub fn new(sample_rate: u32, bit_rate: u32, channels: Channels) -> Self {
        Self { sample_rate, bit_rate, channels }
    }

    pub fn encode_f32(&mut self, _frame: &[f32]) -> Result<Vec<u8>, Error> {
        Ok(vec![])
    }
}
