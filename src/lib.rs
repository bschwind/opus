mod decoder;
mod encoder;

pub use decoder::Decoder;
pub use encoder::Encoder;

pub enum Error {
    Encode,
    Decode,
    InvalidFramesPerPacket,
    InvalidCodecConfig,
}

pub enum Channels {
    Mono,
    Stereo,
}

enum FramesPerPacket {
    One,
    TwoEquallyCompressed,
    TwoDifferentlyCompressed,
    Arbitrary,
}

enum CodecMode {
    SILKOnly,
    Hybrid,
    CELTOnly,
}

enum Bandwidth {
    Narrow,
    Medium,
    Wide,
    SuperWide,
    Full,
}

enum FrameSizeMs {
    TwoPointFive,
    Five,
    Ten,
    Twenty,
    Forty,
    Sixty,
}

struct CodecConfig {
    mode: CodecMode,
    bandwidth: Bandwidth,
    frame_size: FrameSizeMs,
}

struct TableOfContentsHeader {
    codec_config: CodecConfig,
    channels: Channels,
    frames_per_packet: FramesPerPacket,
}
