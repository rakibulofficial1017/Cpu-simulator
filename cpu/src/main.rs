#![allow(warnings)]
mod hardware;
mod cpu;

use cpu::computer::Computer;

fn main() {
    let mut computer = Computer::new();

    // Write 20 into RAM[1000].
    let rising_edge = computer.clock.tick();

    computer.ram.update(
        20,
        true,
        1000,
        rising_edge,
    );

    // Read RAM[1000].
    let data = computer.ram.output(1000);

    // Advance until the next rising edge.
    let mut rising_edge = false;

    while !rising_edge {
        rising_edge = computer.clock.tick();
    }

    computer.cpu.receive_memory_response(
        cpu::cpu::MemoryResponse { data },
        rising_edge,
    );

    computer.cpu.debug_registers();

    assert_eq!(computer.cpu.dr.output(), 20);
}