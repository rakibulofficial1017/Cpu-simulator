#![allow(warnings)]

mod cpu;
mod hardware;

use cpu::computer::Computer;

fn tick_until_memory_request_is_serviced(computer: &mut Computer) {
    while !computer.tick() {}
}

fn main() {
    let mut computer = Computer::new();
    computer
        .cpu
        .execute_instruction_with_operands(0x00, &[0x14])
        .expect("LDI should execute with one immediate byte");

    computer
        .cpu
        .execute_instruction_with_operands(0x01, &[0x10, 0x00])
        .expect("LDA should execute with a two-byte address");
    tick_until_memory_request_is_serviced(&mut computer);
    
    computer
    .cpu
    .execute_instruction(0x02)
    .expect("STA should execute without operands");
    tick_until_memory_request_is_serviced(&mut computer);

    computer
        .cpu
        .execute_instruction_with_operands(0x01, &[0x10, 0x00])
        .expect("LDA should execute with a two-byte address");
    tick_until_memory_request_is_serviced(&mut computer);

    computer.cpu.debug_registers();
}
