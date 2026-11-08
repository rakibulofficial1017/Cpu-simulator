

use super::instruction::Opcode;

#[derive(Debug, Clone, Copy, Default)]
pub struct ControlSignals {
    pub halt: bool,
    pub memory_read: bool,
    pub memory_write: bool,
    pub load_acc: bool,
    pub alu_enable: bool,
    pub alu_subtract: bool,
    pub jump: bool,
    pub jump_if_zero: bool,
    pub stack_push: bool,
    pub stack_pop: bool,
}

pub struct InstructionDecoder;

impl InstructionDecoder {
    pub fn decode(opcode: Opcode) -> ControlSignals {
        match opcode {
            Opcode::NOP => ControlSignals::default(),

            Opcode::LDI => ControlSignals {
                load_acc: true,
                ..Default::default()
            },

            Opcode::LDA => ControlSignals {
                memory_read: true,
                ..Default::default()
            },

            Opcode::STA => ControlSignals {
                memory_write: true,
                ..Default::default()
            },

            Opcode::ADD => ControlSignals {
                alu_enable: true,
                ..Default::default()
            },

            Opcode::SUB => ControlSignals {
                alu_enable: true,
                alu_subtract: true,
                ..Default::default()
            },

            Opcode::JMP => ControlSignals {
                jump: true,
                ..Default::default()
            },

            Opcode::JZ => ControlSignals {
                jump_if_zero: true,
                ..Default::default()
            },

            Opcode::PUSH => ControlSignals {
                stack_push: true,
                ..Default::default()
            },

            Opcode::POP => ControlSignals {
                stack_pop: true,
                ..Default::default()
            },

            Opcode::HLT => ControlSignals {
                halt: true,
                ..Default::default()
            },
        }
    }
}


use super::micro_op::{MicroOp, Register16};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlState {
    Fetch,
    Decode,
    Execute,
    Halt,
}

#[derive(Debug)]
pub struct ControlUnit {
    pub state: ControlState,
    pub microstep: usize,
}

impl ControlUnit {
    pub fn new() -> Self {
        Self {
            state: ControlState::Fetch,
            microstep: 0,
        }
    }

    pub fn reset(&mut self) {
        self.state = ControlState::Fetch;
        self.microstep = 0;
    }

    pub fn fetch_micro_op(&self) -> Option<MicroOp> {
        match self.microstep {
            0 => Some(MicroOp::Move16 {
                source: Register16::PC,
                destination: Register16::AR,
            }),

            1 => None, // Wait for memory response.

            _ => None,
        }
    }

    pub fn advance(&mut self) {
        self.microstep += 1;
    }
}
