
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

    // Detect the subtraction configuration:
    // !(~x + y) = x - y
    let is_subtraction =
        f && nx && no && !zx && !zy && !ny && !cin;

    // Signed overflow of the internal addition.
    let signed_sum =
        (final_x as i16 as i32)
        + (final_y as i16 as i32)
        + i32::from(cin);

    let overflow =
        signed_sum > i16::MAX as i32
        || signed_sum < i16::MIN as i32;

    // CY means carry for addition and no-borrow for subtraction.
    let cy = if is_subtraction {
        !carry
    } else {
        carry
    };

    let signed_x = x as i16;
    let signed_y = y as i16;

    ALUResult {
        out: final_output,

        ng: select_bit(final_output, 15),
        zr: final_output == 0,

        cy,
        ov: overflow,

        cy_valid: f,
        ov_valid: f && (!no || is_subtraction),

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

    pub cy_valid: bool,
    pub ov_valid: bool,

    pub eq: bool,
    pub gt: bool,
    pub lt: bool,
}
