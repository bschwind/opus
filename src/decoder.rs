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
    current_frame: usize,
    constant_bit_rate: bool,
    sizes: [usize; MAX_FRAME_COUNT_PER_PACKET],
}

impl<'a> FrameIterator<'a> {
    fn new(toc: &'a TableOfContentsHeader, mut packet: &'a [u8]) -> Result<Self, Error> {
        let mut sizes = [0; MAX_FRAME_COUNT_PER_PACKET];

        let (count, last_frame_size, constant_bit_rate) = match toc.frames_per_packet {
            FramesPerPacket::One => (1, packet.len(), true),
            FramesPerPacket::TwoEquallyCompressed => {
                if packet.len() & 0b1 != 0 {
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
                if packet.len() < 2 {
                    return Err(Error::InvalidPacketSize);
                }

                let first_byte = packet[0];
                let num_frames = (first_byte & 0b00111111) as usize;
                let variable_bit_rate = first_byte & 0b1000000 == 0b1000000;
                let opus_padding_present = first_byte & 0b0100000 == 0b0100000;

                packet = &packet[1..];

                // TODO - Assert num_frames does not exceed 120ms of audio data.
                if num_frames == 0 {
                    return Err(Error::InvalidFrameCount);
                }

                // Decode the amount of padding bytes, if any.
                if opus_padding_present {
                    let mut total_padding_bytes = 0usize;

                    loop {
                        if packet.is_empty() {
                            return Err(Error::InvalidOpusPadding);
                        }

                        match packet[0] {
                            n @ 0..=254 => {
                                total_padding_bytes += n as usize;
                                packet = &packet[1..];
                                break;
                            },
                            255 => {
                                total_padding_bytes += 254;
                                packet = &packet[1..];
                            },
                        }
                    }

                    if packet.len() <= total_padding_bytes as usize {
                        return Err(Error::InvalidPacketSize);
                    }

                    // Chop off the padding bytes at the end.
                    packet = &packet[..(packet.len() - total_padding_bytes)];
                }

                if variable_bit_rate {
                    for size in sizes.iter_mut().take(num_frames - 1) {
                        let (frame_size, num_bytes) =
                            parse_size(packet).ok_or(Error::InvalidPacketSize)?;
                        if frame_size > packet.len() {
                            return Err(Error::InvalidPacketSize);
                        }

                        packet = &packet[num_bytes..];
                        *size = frame_size;
                    }

                    (num_frames, packet.len(), false)
                } else {
                    if packet.len() % num_frames != 0 {
                        // The packet is not cleanly divisible by the number of
                        // constant bit rate encoded frames.
                        return Err(Error::InvalidPacketSize);
                    }

                    let frame_size = packet.len() / num_frames;

                    for size in &mut sizes[0..(num_frames - 1)] {
                        *size = frame_size;
                    }

                    (num_frames, frame_size, true)
                }
            },
        };

        sizes[count - 1] = last_frame_size;
        let current_frame = 0;

        Ok(Self { toc, packet, count, current_frame, constant_bit_rate, sizes })
    }
}

impl<'a> Iterator for FrameIterator<'a> {
    type Item = Result<OpusFrame<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_frame < self.count {
            let (next_frame, remaining_packet) =
                self.packet.split_at(self.sizes[self.current_frame]);
            self.packet = remaining_packet;

            self.current_frame += 1;
            Some(Ok(OpusFrame::new(next_frame)))
        } else {
            None
        }
    }
}

pub struct RangeDecoder<'a> {
    frame_data: &'a [u8],
    // The difference between the high end of the current range
    // and the actual coded value, minus one
    val: u32,

    // The size of the current range
    rng: u32,

    // The leftover bit on the first input byte. The least significant bit.
    leftover_bit: bool,
}

impl<'a> RangeDecoder<'a> {
    pub fn new(mut frame_data: &'a [u8]) -> Self {
        let first_input_byte = if !frame_data.is_empty() {
            let first = frame_data[0];
            frame_data = &frame_data[1..];
            first
        } else {
            0
        };

        let rng = 128;
        let val = (127 - (first_input_byte >> 1)) as u32;

        let mut myself = Self { frame_data, rng, val, leftover_bit: first_input_byte & 1 == 1 };

        myself.renormalize();

        myself
    }

    pub fn decode_u32(&mut self, mut ft: u32) -> u32 {
        assert!(ft > 1);

        ft -= 1;
        let ftb = Self::ilog(ft);

        if ftb > 8 {
            todo!();
            0
        } else {
            todo!();
            ft += 1;
            0
        }
    }

    fn read_byte(&mut self) -> u8 {
        if !self.frame_data.is_empty() {
            let next = self.frame_data[0];
            self.frame_data = &self.frame_data[1..];
            next
        } else {
            0
        }
    }

    fn renormalize(&mut self) {
        while self.rng <= 2u32.pow(23) {
            self.rng <<= 8;
            let next_byte = self.read_byte();

            let sym = next_byte | if self.leftover_bit { 1 } else { 0 };
            self.leftover_bit = next_byte & 1 == 1;

            self.val = ((self.val << 8) + (255u32 - sym as u32)) & 0x7FFFFFFF;
        }
    }

    fn ilog(mut v: u32) -> u32 {
        let mut ret = !!v;
        let mut m = !!(v & 0xFFFF0000) << 4;

        v >>= m;
        ret |= m;
        m = !!(v & 0xFF00) << 3;
        v >>= m;
        ret |= m;
        m = !!(v & 0xF0) << 2;
        v >>= m;
        ret |= m;
        m = !!(v & 0xC) << 1;
        v >>= m;
        ret |= m;
        ret += !!(v & 0x2);

        ret
    }
}

#[derive(Debug)]
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

#[test]
fn test_decode_table_of_contents() {
    let opus_bytes = include_bytes!("../test_data/sin.opus");

    let toc = TableOfContentsHeader::try_from(opus_bytes[0]).unwrap();

    assert_eq!(
        toc,
        TableOfContentsHeader {
            codec_config: CodecConfig {
                bandwidth: Bandwidth::Full,
                frame_size: FrameSizeMs::Ten,
                mode: CodecMode::CELTOnly,
            },
            channels: Channels::Mono,
            frames_per_packet: FramesPerPacket::One,
        }
    );
}

#[test]
fn test_decode_f32() {
    let opus_bytes = include_bytes!("../test_data/sin.opus");
    let mut decoder = Decoder::new(48_000, Channels::Mono);
    decoder.decode_f32(opus_bytes).unwrap();
}
