use crate::{
    Bandwidth, Channels, CodecConfig, CodecMode, Error, FrameSizeMs, FramesPerPacket,
    TableOfContentsHeader,
};
use std::convert::TryFrom;

const MAX_FRAME_COUNT_PER_PACKET: usize = 48;

pub struct Decoder {
    sample_rate: u32,
    channels: Channels,
}

impl Decoder {
    pub fn new(sample_rate: u32, channels: Channels) -> Self {
        Self { sample_rate, channels }
    }

    pub fn decode_f32(&mut self, data: &[u8]) -> Result<Vec<f32>, Error> {
        if data.is_empty() {
            return Err(Error::InvalidPacketSize);
        }

        let table_of_contents = TableOfContentsHeader::try_from(data[0])?;

        if data.len() < 2 {
            return Ok(vec![]);
        }

        let frame_iter = FrameIterator::new(&table_of_contents, &data[1..])?;

        for frame in frame_iter {
            let frame = frame?;
        }

        Ok(vec![])
    }
}

/// Returns (size, num_bytes)
fn parse_size(data: &[u8]) -> Option<(usize, usize)> {
    match data[0] {
        0 => None,
        len @ 1..=251 => Some((len as usize, 1)),
        first_byte @ 252..=255 if data.len() >= 2 => {
            let len = (data[1] as usize * 4) + first_byte as usize;
            Some((len, 2))
        },
        _ => None,
    }
}

struct FrameIterator<'a> {
    toc: &'a TableOfContentsHeader,
    packet: &'a [u8],
    count: usize,
    constant_bit_rate: bool,
    sizes: [usize; MAX_FRAME_COUNT_PER_PACKET],
}

impl<'a> FrameIterator<'a> {
    fn new(toc: &'a TableOfContentsHeader, mut packet: &'a [u8]) -> Result<Self, Error> {
        let mut sizes = [0; MAX_FRAME_COUNT_PER_PACKET];

        let (count, last_frame_size, constant_bit_rate) = match toc.frames_per_packet {
            FramesPerPacket::One => (1, packet.len(), true),
            FramesPerPacket::TwoEquallyCompressed => {
                if packet.len() & 0b1 == 0b1 {
                    return Err(Error::InvalidPacketSize);
                }

                sizes[0] = packet.len() / 2;

                (2, sizes[0], true)
            },
            FramesPerPacket::TwoDifferentlyCompressed => {
                let (first_frame_size, num_bytes) =
                    parse_size(packet).ok_or(Error::InvalidPacketSize)?;

                if first_frame_size > packet.len() {
                    return Err(Error::InvalidPacketSize);
                }

                packet = &packet[num_bytes..];
                let last_frame_size = packet.len() - first_frame_size;

                sizes[0] = first_frame_size;

                (2, last_frame_size, false)
            },
            FramesPerPacket::Arbitrary => {
                // TODO(bschwind) - Implement this
                (7, packet.len(), false)
            },
        };

        sizes[count - 1] = last_frame_size;

        Ok(Self { toc, packet, count, constant_bit_rate, sizes })
    }
}

impl<'a> Iterator for FrameIterator<'a> {
    type Item = Result<OpusFrame<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

struct OpusFrame<'a> {
    compressed_data: &'a [u8],
}

impl<'a> OpusFrame<'a> {
    fn new(compressed_data: &'a [u8]) -> Self {
        Self { compressed_data }
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
