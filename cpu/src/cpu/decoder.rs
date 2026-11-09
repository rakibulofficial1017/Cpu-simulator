use super::instruction::{Instruction, InvalidOpcode, Opcode};
use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodedInstruction {
    pub opcode: Opcode,
    pub instruction: Instruction,
}

impl DecodedInstruction {
    pub const fn length(self) -> u8 {
        self.instruction.length()
    }
}

pub struct Decoder;

impl Decoder {
    pub fn decode(instruction: u8) -> Result<DecodedInstruction, InvalidOpcode> {
        let opcode = Opcode::try_from(instruction)?;

        Ok(DecodedInstruction {
            opcode,
            instruction: opcode.instruction(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::{instruction::SourceRegister, micro_op::ALUOperation};

    #[test]
    fn decoder_preserves_register_specific_opcodes() {
        let add_pc = Decoder::decode(0x5F).unwrap();
        let add_acc = Decoder::decode(0x60).unwrap();

        assert_ne!(add_pc.opcode, add_acc.opcode);
        assert_eq!(
            add_pc.instruction,
            Instruction::Alu {
                operation: ALUOperation::Add,
                source: SourceRegister::PC,
            }
        );
        assert_eq!(
            add_acc.instruction,
            Instruction::Alu {
                operation: ALUOperation::Add,
                source: SourceRegister::ACC,
            }
        );
    }

    #[test]
    fn decoder_rejects_reserved_opcodes() {
        assert_eq!(Decoder::decode(0xCC), Err(InvalidOpcode(0xCC)));
        assert_eq!(Decoder::decode(0xFF), Err(InvalidOpcode(0xFF)));
    }
}
