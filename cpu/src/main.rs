#![allow(warnings)]
mod hardware;
mod cpu;

use cpu::cpu::CPU;
use cpu::micro_op::{MicroOp, Register8};

fn main() {
    let mut cpu = CPU::new();

    // Put 42 into ACC.
    cpu.acc.update(42, true, true);

    // Transfer ACC -> DR.
    cpu.execute_micro_op(
        MicroOp::Move8 {
            source: Register8::ACC,
            destination: Register8::DR,
        },
        true,
    );

    cpu.debug_registers();

    assert_eq!(cpu.acc.output(), 42);
    assert_eq!(cpu.dr.output(), 42);
}