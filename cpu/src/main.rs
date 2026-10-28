
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ALUResult {
    pub out: u16,
    pub ng: bool,
    pub zr: bool,
}

pub fn mux<T>(a: T, b: T, sel: bool) -> T {
    if sel { b } else { a }
}

// 16-bit addition with carry-in and carry-out.
pub fn add(x: u16, y: u16, cin: bool) -> (u16, bool) {
    let sum = x as u32 + y as u32 + u32::from(cin);
    (sum as u16, sum > u16::MAX as u32)
}

pub fn select_bit(x: u16, bit: u32) -> bool {
    bit < 16 && (x & (1u16 << bit)) != 0
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
) -> ALUResult {
    let zeroed_x = mux(x, 0u16, zx);
    let final_x = mux(zeroed_x, !zeroed_x, nx);

    let zeroed_y = mux(y, 0u16, zy);
    let final_y = mux(zeroed_y, !zeroed_y, ny);

    let anded = final_x & final_y;
    let added = add(final_x, final_y, false).0;

    let output = mux(anded, added, f);
    let final_output = mux(output, !output, no);

    ALUResult {
        out: final_output,
        ng: select_bit(final_output, 15),
        zr: final_output == 0,
    }
}

fn main() {
    let result = alu(
        7, 3,
        false, false,
        false, false,
        true, false,
    );

    println!("{result:?}");
}
