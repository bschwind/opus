// The number of bits to use for the range-coded part of unsigned integers.
const UINT_BITS: u32 = 8;
// The total number of bits in each of the state registers.
const CODE_BITS: i32 = 32;
// The number of bits to output at a time.
const SYMBOL_BITS: i32 = 8;
// The maximum symbol value.
#[allow(unused)]
const SYMBOL_MAX: u32 = (1u32 << SYMBOL_BITS) - 1;
// Carry bit of the high-order range symbol.
const CODE_TOP: u32 = 1u32 << (CODE_BITS - 1);
// Low-order bit of the high-order range symbol.
const CODE_BOTTOM: u32 = CODE_TOP >> SYMBOL_BITS;
// The number of bits available for the last, partial symbol in the code field.
const CODE_EXTRA: i32 = (CODE_BITS - 2) % SYMBOL_BITS + 1;
const WINDOW_SIZE: i32 = (std::mem::size_of::<u32>() * 8) as i32;

pub struct RangeDecoder<'a> {
    frame_data: &'a [u8],
    // The difference between the high end of the current range
    // and the actual coded value, minus one
    val: u32,

    // The number of values in the current range
    rng: u32,

    // The saved normalization factor from decode()
    ext: u32,

    bit_decoder: BitDecoder,

    // The leftover bit on the first input byte. The least significant bit.
    leftover_bit: bool,
}

struct BitDecoder {
    // Bits that will be read from at the end
    end_window: u32,

    // Number of valid bits in end_window.
    num_end_bits: i32,

    // The total number of whole bits read/written
    // This does not include partial bits currently in the range coder.
    num_bits_total: i32,
}

impl Default for BitDecoder {
    fn default() -> Self {
        Self {
            end_window: 0,
            num_end_bits: 0,
            num_bits_total: CODE_BITS + 1 - ((CODE_BITS - CODE_EXTRA) / SYMBOL_BITS) * SYMBOL_BITS,
        }
    }
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

        let rng = 1 << CODE_EXTRA;
        let val = rng - 1 - (first_input_byte as u32 >> (SYMBOL_BITS - CODE_EXTRA));
        let ext = 0;
        let bit_decoder = BitDecoder::default();

        let mut myself = Self {
            frame_data,
            rng,
            val,
            ext,
            bit_decoder,
            leftover_bit: first_input_byte & 1 == 1,
        };

        myself.renormalize();

        myself
    }

    fn update(&mut self, frequency_low: u32, frequency_high: u32, frequency_total: u32) {
        let s: u32 = self.ext * (frequency_total - frequency_high);
        self.val -= s;
        self.rng = if frequency_low > 0 {
            self.ext * (frequency_high - frequency_low)
        } else {
            self.rng - s
        };

        self.renormalize();
    }

    // frequency_total - The total frequency of the symbols in the alphabet the next symbol was encoded with.
    pub fn decode_u32(&mut self, mut frequency_total: u32) -> u32 {
        assert!(frequency_total > 1);

        frequency_total -= 1;

        // The number of bits required to store (frequency_total - 1) in two's complement.
        let frequency_total_bits = Self::ilog(frequency_total);

        if frequency_total_bits > UINT_BITS {
            // The top 8 bits of t are decoded using temp:
            let temp = ((frequency_total - 1) >> (frequency_total_bits - UINT_BITS)) + 1;
            let t = self.decode(temp);

            // Update decoder state using (t, t+1, ((ft -1) >> (ftb - 8)) + 1)
            self.update(t, t + 1, temp);

            // The remaining bits are decoded as raw bits.
            let t = (t << (frequency_total_bits - UINT_BITS))
                | self.decode_bits(frequency_total_bits - UINT_BITS);

            if t <= frequency_total {
                return t;
            }

            // TODO(bschwind) - An error occurred at this point in the code, return a Result
            frequency_total
        } else {
            frequency_total += 1;
            let t = self.decode(frequency_total);
            self.update(t, t + 1, frequency_total);
            t
        }
    }

    // TODO(bschwind) - Return a u16 here?
    fn decode(&mut self, frequency_total: u32) -> u32 {
        self.ext = self.rng / frequency_total;
        let s = self.val / self.ext;

        frequency_total - (s + 1).min(frequency_total)
    }

    fn decode_bits(&mut self, bits: u32) -> u32 {
        let mut window = self.bit_decoder.end_window;
        let mut available = self.bit_decoder.num_end_bits;

        if (available as u32) < bits {
            loop {
                window |= (self.read_byte_from_end() as u32) << available;
                available += SYMBOL_BITS;

                if available <= WINDOW_SIZE - SYMBOL_BITS {
                    break;
                }
            }
        }

        let ret = window & ((1 << bits) - 1);

        window >>= bits;
        available -= bits as i32;

        self.bit_decoder.end_window = window;
        self.bit_decoder.num_end_bits = available;
        self.bit_decoder.num_bits_total += bits as i32;

        ret
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

    fn read_byte_from_end(&mut self) -> u8 {
        if !self.frame_data.is_empty() {
            let next = self.frame_data[self.frame_data.len() - 1];
            self.frame_data = &self.frame_data[..(self.frame_data.len() - 1)];
            next
        } else {
            0
        }
    }

    fn renormalize(&mut self) {
        while self.rng <= CODE_BOTTOM {
            self.bit_decoder.num_bits_total += SYMBOL_BITS;
            self.rng <<= SYMBOL_BITS;
            let next_byte = self.read_byte();

            let sym = next_byte | if self.leftover_bit { 1 } else { 0 };
            self.leftover_bit = next_byte & 1 == 1;

            // Slightly weirder but more flexible:
            // self.val = ((self.val << SYMBOL_BITS) + (SYMBOL_MAX & !(sym as u32))) & (CODE_TOP - 1);
            self.val = ((self.val << SYMBOL_BITS) + (255u32 - sym as u32)) & 0x7FFFFFFF;
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
