use crate::{
    Bandwidth, Channels, CodecConfig, CodecMode, Error, FrameSizeMs, FramesPerPacket,
    TableOfContentsHeader,
};
use std::convert::TryFrom;

pub struct Decoder {}

impl Decoder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn decode_f32(&mut self, _encoded_data: &[u8]) -> Result<Vec<f32>, Error> {
        Ok(vec![])
    }
}

impl TryFrom<u8> for FramesPerPacket {
    type Error = Error;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte & 0b00000011 {
            0 => Ok(FramesPerPacket::One),
            1 => Ok(FramesPerPacket::TwoEquallyCompressed),
            2 => Ok(FramesPerPacket::TwoDifferentlyCompressed),
            3 => Ok(FramesPerPacket::Arbitrary),
            _ => Err(Error::InvalidFramesPerPacket),
        }
    }
}

impl TryFrom<u8> for CodecConfig {
    type Error = Error;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        let config_val = (byte & 0b11111000) >> 3;

        let (mode, bandwidth, frame_size) = match config_val {
            0 => (CodecMode::SILKOnly, Bandwidth::Narrow, FrameSizeMs::Ten),
            1 => (CodecMode::SILKOnly, Bandwidth::Narrow, FrameSizeMs::Twenty),
            2 => (CodecMode::SILKOnly, Bandwidth::Narrow, FrameSizeMs::Forty),
            3 => (CodecMode::SILKOnly, Bandwidth::Narrow, FrameSizeMs::Sixty),

            4 => (CodecMode::SILKOnly, Bandwidth::Medium, FrameSizeMs::Ten),
            5 => (CodecMode::SILKOnly, Bandwidth::Medium, FrameSizeMs::Twenty),
            6 => (CodecMode::SILKOnly, Bandwidth::Medium, FrameSizeMs::Forty),
            7 => (CodecMode::SILKOnly, Bandwidth::Medium, FrameSizeMs::Sixty),

            8 => (CodecMode::SILKOnly, Bandwidth::Wide, FrameSizeMs::Ten),
            9 => (CodecMode::SILKOnly, Bandwidth::Wide, FrameSizeMs::Twenty),
            10 => (CodecMode::SILKOnly, Bandwidth::Wide, FrameSizeMs::Forty),
            11 => (CodecMode::SILKOnly, Bandwidth::Wide, FrameSizeMs::Sixty),

            12 => (CodecMode::Hybrid, Bandwidth::SuperWide, FrameSizeMs::Ten),
            13 => (CodecMode::Hybrid, Bandwidth::SuperWide, FrameSizeMs::Twenty),

            14 => (CodecMode::Hybrid, Bandwidth::Full, FrameSizeMs::Ten),
            15 => (CodecMode::Hybrid, Bandwidth::Full, FrameSizeMs::Twenty),

            16 => (CodecMode::CELTOnly, Bandwidth::Narrow, FrameSizeMs::TwoPointFive),
            17 => (CodecMode::CELTOnly, Bandwidth::Narrow, FrameSizeMs::Five),
            18 => (CodecMode::CELTOnly, Bandwidth::Narrow, FrameSizeMs::Ten),
            19 => (CodecMode::CELTOnly, Bandwidth::Narrow, FrameSizeMs::Twenty),

            20 => (CodecMode::CELTOnly, Bandwidth::Wide, FrameSizeMs::TwoPointFive),
            21 => (CodecMode::CELTOnly, Bandwidth::Wide, FrameSizeMs::Five),
            22 => (CodecMode::CELTOnly, Bandwidth::Wide, FrameSizeMs::Ten),
            23 => (CodecMode::CELTOnly, Bandwidth::Wide, FrameSizeMs::Twenty),

            24 => (CodecMode::CELTOnly, Bandwidth::SuperWide, FrameSizeMs::TwoPointFive),
            25 => (CodecMode::CELTOnly, Bandwidth::SuperWide, FrameSizeMs::Five),
            26 => (CodecMode::CELTOnly, Bandwidth::SuperWide, FrameSizeMs::Ten),
            27 => (CodecMode::CELTOnly, Bandwidth::SuperWide, FrameSizeMs::Twenty),

            28 => (CodecMode::CELTOnly, Bandwidth::Full, FrameSizeMs::TwoPointFive),
            29 => (CodecMode::CELTOnly, Bandwidth::Full, FrameSizeMs::Five),
            30 => (CodecMode::CELTOnly, Bandwidth::Full, FrameSizeMs::Ten),
            31 => (CodecMode::CELTOnly, Bandwidth::Full, FrameSizeMs::Twenty),
            _ => return Err(Error::InvalidCodecConfig),
        };

        Ok(CodecConfig { mode, bandwidth, frame_size })
    }
}

impl TryFrom<u8> for TableOfContentsHeader {
    type Error = Error;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        let codec_config = CodecConfig::try_from(byte)?;
        let frames_per_packet = FramesPerPacket::try_from(byte)?;
        let channels = if (byte >> 2) & 0b1 == 0b1 { Channels::Stereo } else { Channels::Mono };

        Ok(TableOfContentsHeader { codec_config, channels, frames_per_packet })
    }
}
