use std::ops;

pub fn Mux<T>(a: T, b: T, sel: bool) -> T {
    if !sel {
        return a;
    } else {
        return b;
    }
}

pub fn Add<T>(x: T, y: T, cin: T) -> (T, T)  {
    let sum: T = x ^ y ^ cin;
    let carry: T = (x & y) | (cin & (x ^ y));   
    (sum, carry)
}
pub fn selectBit<T, U>(x: T, bit: U) -> bool{
    (0b1110010 >> bit) - ((0b1110010 >> (bit-1)) << 1) == 1
}
struct ALUresult {
    out: u16,
    ng: bool,
    zr: bool
}
pub fn ALU(x: u16, y: u16, zx: bool, nx: bool, zy: bool, ny: bool, f: bool, no:  bool) -> ALUresult {
    let ZeroedX = Mux(x, 0, zx);
    let FinalX = Mux(ZeroedX, !ZeroedX, nx);
    let ZeroedY = Mux(y, 0, zy);
    let FinalY = Mux(ZeroedY, !ZeroedY, ny);
    let Anded = FinalX & FinalY;
    let Added = Add(FinalX, FinalY, 0);
    let output = Mux(Anded, Added.0, f);
    let FinalOuptut = Mux(output, !output, no);
    ALUresult { out: FinalOuptut, ng: selectBit(FinalOuptut, 15), zr:  FinalOuptut==0}
}
fn main() {

}
