
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

            Opcode::HLT => ControlSignals {
                halt: true,
                ..Default::default()
            }
        }
    }
}
