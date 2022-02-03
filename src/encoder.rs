use crate::{Channels, Error};

pub struct Encoder {
    _sample_rate: u32,
    _bit_rate: u32,
    _channels: Channels,
}

impl Encoder {
    pub fn new(sample_rate: u32, bit_rate: u32, channels: Channels) -> Self {
        Self { _sample_rate: sample_rate, _bit_rate: bit_rate, _channels: channels }
    }

    pub fn encode_f32(&mut self, _frame: &[f32]) -> Result<Vec<u8>, Error> {
        Ok(vec![])
    }
}
