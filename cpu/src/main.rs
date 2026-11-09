#![allow(warnings)]

mod hardware;
mod cpu;

use cpu::computer::Computer;

fn main() {
    let mut computer = Computer::new();
    computer.cpu.execute_instruction(0b00000000u8);
    computer.cpu.debug_registers();
}
