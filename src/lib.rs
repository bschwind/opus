mod decoder;
mod encoder;
mod range_coding;

pub use decoder::Decoder;
pub use encoder::Encoder;

#[derive(Debug)]
pub enum Error {
    Encode,
    Decode,
    InvalidPacketSize,
    InvalidFramesPerPacket,
    InvalidFrameCount,
    InvalidOpusPadding,
    InvalidCodecConfig,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Channels {
    Mono,
    Stereo,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum FramesPerPacket {
    One,                      // Code 0 packet
    TwoEquallyCompressed,     // Code 1 packet
    TwoDifferentlyCompressed, // Code 2 packet
    Arbitrary,                // Code 3 packet
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum CodecMode {
    SILKOnly,
    Hybrid,
    CELTOnly,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Bandwidth {
    Narrow,
    Medium,
    Wide,
    SuperWide,
    Full,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum FrameSizeMs {
    TwoPointFive,
    Five,
    Ten,
    Twenty,
    Forty,
    Sixty,
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct CodecConfig {
    mode: CodecMode,
    bandwidth: Bandwidth,
    frame_size: FrameSizeMs,
}

#[derive(Debug, Clone, PartialEq)]
struct TableOfContentsHeader {
    codec_config: CodecConfig,
    channels: Channels,
    frames_per_packet: FramesPerPacket,
}
