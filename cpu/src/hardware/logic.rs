pub fn mux<T>(a: T, b: T, sel: bool) -> T {
    if sel { b } else { a }
}


pub fn dmux<T>(in_: T, sel: bool) -> (T, T)
where T: Default {
    if sel { 
        (T::default(), in_) 
    } else { 
        (in_, T::default()) 
    }
}
pub fn mux8way<T>(a: T, b: T, c:T, d:T, e:T, f:T,g:T, h:T, sel: u8) -> T {
    if select_bit_of8(sel, 2) {
        if select_bit_of8(sel, 1) {
            if select_bit_of8(sel, 0) {
                h
            } else {
                g
            }
        } else {
            if select_bit_of8(sel, 0) {
                f
            } else {
                e
            }
        }
    } else {
        if select_bit_of8(sel, 1) {
            if select_bit_of8(sel, 0) {
                d
            } else {
                c
            }
        } else {
            if select_bit_of8(sel, 0) {
                b
            } else {
                a
            }
        }

    }

}
pub fn dmux8way<T: Default>(in_: T, sel: u8) -> [T; 8] {
    let mut outputs: [T; 8] = [
        T::default(), T::default(), T::default(), T::default(),
        T::default(), T::default(), T::default(), T::default()
    ];
    outputs[(sel & 0b111) as usize] = in_;

    outputs
}
pub fn select_bit(x: u16, bit: u32) -> bool {
    bit < 16 && (x & (1u16 << bit)) != 0
}
pub fn select_bit_of8(x: u8, bit: u32) -> bool {
    bit < 8 && (x & (1u8 << bit)) != 0
}