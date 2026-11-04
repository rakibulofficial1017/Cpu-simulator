use crate::hardware::{logic::{select_bit, select_bit_of8}, flip_flop::Bit};

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
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