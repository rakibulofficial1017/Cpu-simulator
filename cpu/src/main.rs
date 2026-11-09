#![allow(warnings)]

mod hardware;
mod cpu;

use cpu::computer::Computer;

fn main() {
    let mut computer = Computer::new();
    computer.tick();
    computer.cpu.execute_micro_op(
        cpu::micro_op::MicroOp::ALU { a: cpu::micro_op::ALUInput::Immediate(0), b: cpu::micro_op::ALUInput::Immediate(20), update_flags: false, compare: false, destination: cpu::micro_op::ALUDestination::ACC, zx: true, nx: false, zy: false, ny: false, f: true, no: false, cin: false }, true
    );
    computer.cpu.execute_micro_op(
        cpu::micro_op::MicroOp::ALU { a: cpu::micro_op::ALUInput::Immediate(0), b: cpu::micro_op::ALUInput::Immediate(65535), update_flags: false, compare: false, destination: cpu::micro_op::ALUDestination::TempA, zx: true, nx: false, zy: false, ny: false, f: true, no: false, cin: false }, true
    );
    computer.cpu.execute_micro_op(
        cpu::micro_op::MicroOp::ALU { a: cpu::micro_op::ALUInput::ACC, b: cpu::micro_op::ALUInput::TempA, update_flags: true, compare: true, destination: cpu::micro_op::ALUDestination::TempB, zx: false, nx: false, zy: false, ny: false, f: true, no: false, cin: false }, true
    );
    computer.cpu.execute_micro_op(
    cpu::micro_op::MicroOp::ALU {
        a: cpu::micro_op::ALUInput::Immediate(10),
        b: cpu::micro_op::ALUInput::Immediate(20),
        update_flags: true,
        compare: false,
        destination: cpu::micro_op::ALUDestination::TempB,
        zx: false,
        nx: false,
        zy: false,
        ny: false,
        f: true,
        no: false,
        cin: true,
    },
    true,
);
    computer.cpu.debug_registers();
}
