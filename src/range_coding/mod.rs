// The number of bits to use for the range-coded part of unsigned integers.
const UINT_BITS: u32 = 8;
// The total number of bits in each of the state registers.
const CODE_BITS: i32 = 32;
// The number of bits to output at a time.
const SYMBOL_BITS: i32 = 8;
// The maximum symbol value.
#[allow(unused)]
const SYMBOL_MAX: u32 = (1u32 << SYMBOL_BITS) - 1;
// Bits to shift by to move a symbol into the high-order position.
const CODE_SHIFT: i32 = CODE_BITS - SYMBOL_BITS - 1;
// Carry bit of the high-order range symbol.
const CODE_TOP: u32 = 1u32 << (CODE_BITS - 1);
// Low-order bit of the high-order range symbol.
const CODE_BOTTOM: u32 = CODE_TOP >> SYMBOL_BITS;
// The number of bits available for the last, partial symbol in the code field.
const CODE_EXTRA: i32 = (CODE_BITS - 2) % SYMBOL_BITS + 1;
const WINDOW_SIZE: i32 = (std::mem::size_of::<u32>() * 8) as i32;

mod range_decoder;
mod range_encoder;

pub use range_decoder::RangeDecoder;
pub use range_encoder::RangeEncoder;
