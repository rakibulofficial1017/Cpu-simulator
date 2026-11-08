
use crate::hardware::logic::{mux, select_bit};

pub fn inc16(input: u16) -> u16 {
    add(input, 1, false).0
}

pub fn add(x: u16, y: u16, cin: bool) -> (u16, bool) {
    let sum = x as u32 + y as u32 + u32::from(cin);
    (sum as u16, sum > u16::MAX as u32)
}

pub fn alu(
    x: u16,
    y: u16,
    zx: bool,
    nx: bool,
    zy: bool,
    ny: bool,
    f: bool,
    no: bool,
    cin: bool,
) -> ALUResult {
    let zeroed_x = mux(x, 0u16, zx);
    let final_x = mux(zeroed_x, !zeroed_x, nx);

    let zeroed_y = mux(y, 0u16, zy);
    let final_y = mux(zeroed_y, !zeroed_y, ny);

    let anded = final_x & final_y;
    let (added, carry) = add(final_x, final_y, cin);

    let output = mux(anded, added, f);
    let final_output = mux(output, !output, no);

    // Signed overflow for two's-complement addition.
    let overflow = (!(final_x ^ final_y) & (final_x ^ added) & 0x8000) != 0;

    // Signed comparison of the original operands.
    let signed_x = x as i16;
    let signed_y = y as i16;

    ALUResult {
        out: final_output,
        ng: select_bit(final_output, 15),
        zr: final_output == 0,

        // Carry and overflow are meaningful for the addition path.
        cy: f && carry,
        ov: f && overflow,

        eq: signed_x == signed_y,
        gt: signed_x > signed_y,
        lt: signed_x < signed_y,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ALUResult {
    pub out: u16,
    pub ng: bool,
    pub zr: bool,
    pub cy: bool,
    pub ov: bool,
    pub eq: bool,
    pub gt: bool,
    pub lt: bool,
}
