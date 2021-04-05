use crate::range_coding::{CODE_BOTTOM, CODE_SHIFT, CODE_TOP, SYMBOL_BITS, SYMBOL_MAX};

pub struct RangeEncoder {
    // The low end of the current range
    val: u32,

    // The size of the current range
    rng: u32,

    // A buffered output byte (should always be less than 255)
    rem: Option<u8>,

    // A count of additional carry-propagating output bytes
    ext: u16,
}

impl RangeEncoder {
    pub fn new() -> Self {
        Self { val: 0, rng: CODE_TOP, rem: None, ext: 0 }
    }

    pub fn encode(&mut self, frequency_low: u16, frequency_high: u16, frequency_total: u16) {
        let r = self.rng / frequency_total as u32;

        if frequency_low > 0 {
            self.val += self.rng - (r * (frequency_total - frequency_low) as u32);
            self.rng = r * (frequency_high - frequency_low) as u32;
        } else {
            self.rng = r * (frequency_total - frequency_high) as u32;
        }

        self.renormalize();
    }

    fn renormalize(&mut self) {
        while self.rng <= CODE_BOTTOM {
            self.carry_out((self.val >> CODE_SHIFT) as u8);
        }
    }

    fn carry_out(&mut self, c: u8) {
        if c as u32 != SYMBOL_MAX {
            let carry = c >> SYMBOL_BITS;

            if let Some(rem) = self.rem {
                // TODO - Write a byte (self.rem + carry)
            }

            if self.ext > 0 {
                let sym = ((SYMBOL_MAX + carry as u32) & SYMBOL_MAX) as u8;

                loop {
                    // TODO - Write a byte (sym)
                    self.ext -= 1;
                    if self.ext <= 0 {
                        break;
                    }
                }
            }

            self.rem = Some((c as u32 & SYMBOL_MAX) as u8);
        } else {
            self.ext += 1;
        }
    }

    fn write_byte(&mut self) {
        todo!();
    }
}
