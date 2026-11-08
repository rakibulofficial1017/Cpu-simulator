
mod hardware;
mod cpu;

use cpu::computer::Computer;

fn main() {
    let mut computer = Computer::new();

    // Request RAM[1000] = 20.
    computer.cpu.request_write(1000, 20);

    // First tick: Low -> High. Write occurs.
    let rising = computer.tick();

    computer.cpu.ar.update(1000, true, rising);

    // Second tick: High -> Low.    
    computer.tick();
    
    // Request a read from RAM[1000].
    computer.cpu.request_read(1000);

    // Third tick: Low -> High. Read response enters DR.
    computer.tick();

    computer.cpu.debug_registers();

    assert_eq!(computer.cpu.dr.output(), 20);
}
