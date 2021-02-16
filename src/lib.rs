mod decoder;
mod encoder;

pub use decoder::Decoder;
pub use encoder::Encoder;

pub enum Error {
    Encode,
    Decode,
}

pub enum Channels {
    Mono,
    Stereo,
}
