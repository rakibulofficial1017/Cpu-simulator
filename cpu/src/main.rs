use std::{panic::RefUnwindSafe, result};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ALUResult {
    pub out: u16,
    pub ng: bool,
    pub zr: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level{
    Low,
    High,
}

#[derive(Debug)]
pub struct Clock{
    pub level: Level,
    pub cycles: u64,
}

impl Clock {
    pub fn new() -> Self {
        Self{
            level: Level::Low,
            cycles: 0,
        }
    }
    // Returns true if rising
    pub fn tick(&mut self) -> bool {
        let rising = self.level == Level::Low;

        self.level = match self.level {
            Level::High => Level::Low,
            Level::Low => Level::High,
        };

        if rising {
            self.cycles += 1;
        }

        rising
    }
}


#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct DFF {
    pub q: bool,
}

impl DFF {
    pub fn new() -> Self {
        Self { q: false }
    }

    pub fn update(&mut self, d: bool, rising_edge: bool) {
        if rising_edge {
            self.q = d;
        }
    }
}


#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct Bit {
    dff: DFF,
}

impl Bit {
    pub fn new() -> Self {
        Self {
            dff: DFF::new(),
        }
    }

    pub fn output(&self) -> bool {
        self.dff.q
    }

    pub fn update(&mut self, input: bool, load: bool, rising_edge: bool) {
        let d = mux(self.dff.q, input, load);
        self.dff.update(d, rising_edge);
    }
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Register {
    bits: [Bit; 8],
}

impl Register {
    pub fn new() -> Self {
        Self {
            bits: [Bit::new(); 8],
        }
    }

    pub fn output(&self) -> u8 {
        let mut result = 0u8;

        for i in 0..8 {
            if self.bits[i].output() {
                result |= 1u8 << i;
            }
        }

        result
    }

    pub fn update(&mut self, input: u8, load: bool, rising: bool) -> u8 {
        for i in 0..8 {
            self.bits[i].update(
                select_bit_of8(input, i as u32),
                load,
                rising,
            );
        }

        self.output()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub struct Register16 {
    bits: [Bit; 16],
}

impl Register16 {
    pub fn new() -> Self {
        Self {
            bits: [Bit::new(); 16],
        }
    }
    
    pub fn output(&self) -> u16 {
        let mut result = 0u16;
        
        for i in 0..16 {
            if self.bits[i].output() {
                result |= 1u16 << i;
            }
        }
        
        result
    }
    
    pub fn update(&mut self, input: u16, load: bool, rising: bool) -> u16 {
        for i in 0..16 {
            self.bits[i].update(
                select_bit(input, i as u32),
                load,
                rising,
            );
        }
        
        self.output()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RAM8 {
    registers: [Register16; 8],
}

impl RAM8 {
    pub fn new() -> Self {
        Self {
            registers: [Register16::new(); 8],
        }
    }

    pub fn output(&self, address: u8) -> u16 {
        self.registers[(address & 0b111) as usize].output()
    }

    pub fn update(
        &mut self,
        input: u16,
        load: bool,
        address: u8,
        rising: bool,
    ) -> u16 {
        let selected = (address & 0b111) as usize;

        for i in 0..8 {
            self.registers[i].update(
                input,
                load && i == selected,
                rising,
            );
        }

        self.output(address)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PC {
    register: Register16,
}

impl PC {
    pub fn new() -> Self {
        Self {
            register: Register16::new(),
        }
    }

    pub fn output(&self) -> u16 {
        self.register.output()
    }

    pub fn update(
        &mut self,
        input: u16,
        load: bool,
        inc: bool,
        reset: bool,
        rising: bool,
    ) -> u16 {
        let reg_out = self.register.output();

        // 1. Increment the current program counter.
        let inc_out = inc16(reg_out);

        // 2. If inc, select the incremented value; otherwise hold.
        let out_inc_selection = mux(reg_out, inc_out, inc);

        // 3. If load, select the input address.
        let out_load_selection =
            mux(out_inc_selection, input, load);

        // 4. If reset, select zero.
        let to_reg_in = mux(out_load_selection, 0u16, reset);

        // 5. Update the register if any control signal is active.
        let reg_load_signal = inc || load || reset;

        // 6. Store the selected value on the rising edge.
        self.register
            .update(to_reg_in, reg_load_signal, rising);

        self.output()
    }
}
pub fn inc16(input: u16) -> u16 {
    add(input, 1, false).0
}
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
// 16-bit addition with carry-in and carry-out.
pub fn add(x: u16, y: u16, cin: bool) -> (u16, bool) {
    let sum = x as u32 + y as u32 + u32::from(cin);
    (sum as u16, sum > u16::MAX as u32)
}

pub fn select_bit(x: u16, bit: u32) -> bool {
    bit < 16 && (x & (1u16 << bit)) != 0
}
pub fn select_bit_of8(x: u8, bit: u32) -> bool {
    bit < 8 && (x & (1u8 << bit)) != 0
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
    let mut clock = Clock::new();
    let mut pc = PC::new();

    assert_eq!(pc.output(), 0);


    let rising = clock.tick();
    pc.update(100, true, false, false, rising);
    assert_eq!(pc.output(), 100);


    clock.tick(); // Falling edge.
    pc.update(0, false, true, false, false);
    let rising = clock.tick();
    pc.update(0, false, true, false, rising);
    assert_eq!(pc.output(), 101);


    clock.tick();
    pc.update(500, true, true, true, false);
    let rising = clock.tick();
    pc.update(500, true, true, true, rising);
    assert_eq!(pc.output(), 0);

    println!("PC tests passed!");
}
