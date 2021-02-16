use crate::Error;

pub struct Decoder {}

impl Decoder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn decode_f32(&mut self, _encoded_data: &[u8]) -> Result<Vec<f32>, Error> {
        Ok(vec![])
    }
}
