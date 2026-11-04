#![allow(warnings)]
mod hardware;
mod cpu;

use cpu::cpu::CPU;
fn main() {
    let mut cpu = CPU::new();
    cpu.request_read();
}
