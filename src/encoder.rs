use crate::Error;

pub struct Encoder {}

impl Encoder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn encode_f32(&mut self, _frame: &[f32]) -> Result<Vec<u8>, Error> {
        Ok(vec![])
    }
}
