use std::result;


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
    let mut main_clock = Clock::new();
}
