
use crate::hardware::{
    clock::Clock,
    ram::RAM64K,
};

use super::cpu::CPU;

#[derive(Debug)]
pub struct Computer {
    pub cpu: CPU,
    pub ram: RAM64K,
    pub clock: Clock,
}

impl Computer {
    pub fn new() -> Self {
        Self {
            cpu: CPU::new(),
            ram: RAM64K::new(),
            clock: Clock::new(),
        }
    }
}
