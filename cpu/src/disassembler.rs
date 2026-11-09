use crate::cpu::{
    decoder::Decoder,
    instruction::{BranchCondition, Instruction, SourceRegister},
    micro_op::ALUOperation,
};
use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisassemblyError {
    pub offset: usize,
    pub opcode: u8,
    pub expected_instruction_bytes: usize,
    pub available_bytes: usize,
}

impl fmt::Display for DisassemblyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "truncated instruction at byte {}: opcode 0x{:02X} needs {} byte(s), only {} available",
            self.offset, self.opcode, self.expected_instruction_bytes, self.available_bytes
        )
    }
}

impl Error for DisassemblyError {}

pub fn disassemble(binary: &[u8]) -> Result<String, DisassemblyError> {
    let mut output = String::new();
    let mut offset = 0;

    while offset < binary.len() {
        let opcode = binary[offset];
        let Ok(decoded) = Decoder::decode(opcode) else {
            output.push_str(&format!(".byte 0x{opcode:02X} ; {offset:04X}\n"));
            offset += 1;
            continue;
        };

        let instruction_bytes = decoded.length() as usize;
        let remaining_bytes = binary.len() - offset;
        if remaining_bytes < instruction_bytes {
            return Err(DisassemblyError {
                offset,
                opcode,
                expected_instruction_bytes: instruction_bytes,
                available_bytes: remaining_bytes,
            });
        }

        let operands = &binary[offset + 1..offset + instruction_bytes];
        let assembly = format_instruction(decoded.instruction, operands);
        output.push_str(&format!("{assembly:<24} ; {offset:04X}\n"));
        offset += instruction_bytes;
    }

    Ok(output)
}

fn format_instruction(instruction: Instruction, operands: &[u8]) -> String {
    match instruction {
        Instruction::LoadImmediate => format!("LDI #0x{:02X}", operands[0]),
        Instruction::LoadAddress => {
            let address = u16::from_be_bytes([operands[0], operands[1]]);
            format!("LDA #0x{address:04X}")
        }
        Instruction::StoreAddress => "STA".to_string(),
        Instruction::Move {
            source,
            destination,
        } => format!(
            "MOV {} -> {}",
            source_name(source),
            destination_name(destination)
        ),
        Instruction::Compare { left, right } => {
            format!("CMP {}, {}", source_name(left), source_name(right))
        }
        Instruction::Alu { operation, source } => {
            format!("{} {}", alu_name(operation), source_name(source))
        }
        Instruction::Unary(operation) => alu_name(operation).to_string(),
        Instruction::Push(source) => format!("PUSH {}", source_name(source)),
        Instruction::Pop(destination) => format!("POP {}", destination_name(destination)),
        Instruction::Jump(condition) => jump_name(condition).to_string(),
        Instruction::Call => "CALL".to_string(),
        Instruction::Return => "RET".to_string(),
        Instruction::Halt => "HLT".to_string(),
        Instruction::Nop => "NOP".to_string(),
        Instruction::Interrupt => format!("INT #0x{:02X}", operands[0]),
        Instruction::InterruptReturn => "IRET".to_string(),
        Instruction::EnableInterrupts => "EI".to_string(),
        Instruction::DisableInterrupts => "DI".to_string(),
    }
}

fn source_name(source: SourceRegister) -> &'static str {
    match source {
        SourceRegister::PC => "PC",
        SourceRegister::ACC => "ACC",
        SourceRegister::TempA => "TempA",
        SourceRegister::TempB => "TempB",
        SourceRegister::PSR => "PSR",
        SourceRegister::SR => "SR",
        SourceRegister::AR => "AR",
        SourceRegister::DR => "DR",
    }
}

fn destination_name(destination: crate::cpu::instruction::DestinationRegister) -> &'static str {
    match destination {
        crate::cpu::instruction::DestinationRegister::ACC => "ACC",
        crate::cpu::instruction::DestinationRegister::AR => "AR",
        crate::cpu::instruction::DestinationRegister::TempA => "TempA",
        crate::cpu::instruction::DestinationRegister::TempB => "TempB",
    }
}

fn alu_name(operation: ALUOperation) -> &'static str {
    match operation {
        ALUOperation::Add => "ADD",
        ALUOperation::Adc => "ADC",
        ALUOperation::Subtract => "SUB",
        ALUOperation::Sbc => "SBC",
        ALUOperation::And => "AND",
        ALUOperation::Or => "OR",
        ALUOperation::Xor => "XOR",
        ALUOperation::Test => "TEST",
        ALUOperation::Increment => "INC",
        ALUOperation::Decrement => "DEC",
        ALUOperation::Not => "NOT",
        ALUOperation::ShiftLeft => "SHL",
        ALUOperation::ShiftRight => "SHR",
        ALUOperation::ShiftArithmeticRight => "SAR",
        ALUOperation::RotateLeft => "ROL",
        ALUOperation::RotateRight => "ROR",
        ALUOperation::RotateCarryLeft => "RCL",
        ALUOperation::RotateCarryRight => "RCR",
    }
}

fn jump_name(condition: BranchCondition) -> &'static str {
    match condition {
        BranchCondition::Always => "JMP",
        BranchCondition::Zero => "JZ",
        BranchCondition::NotZero => "JNZ",
        BranchCondition::Negative => "JN",
        BranchCondition::NotNegative => "JNN",
        BranchCondition::Carry => "JC",
        BranchCondition::NotCarry => "JNC",
        BranchCondition::Overflow => "JV",
        BranchCondition::NotOverflow => "JNV",
        BranchCondition::Equal => "JE",
        BranchCondition::NotEqual => "JNE",
        BranchCondition::Greater => "JGT",
        BranchCondition::Less => "JLT",
        BranchCondition::GreaterOrEqual => "JGE",
        BranchCondition::LessOrEqual => "JLE",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembler::assemble;

    #[test]
    fn disassembles_instructions_operands_and_reserved_bytes() {
        let assembly = disassemble(&[0x00, 0x14, 0x01, 0x12, 0x34, 0x60, 0xCC, 0xC6]).unwrap();
        assert!(assembly.contains("LDI #0x14"));
        assert!(assembly.contains("LDA #0x1234"));
        assert!(assembly.contains("ADD ACC"));
        assert!(assembly.contains(".byte 0xCC"));
        assert!(assembly.contains("HLT"));
    }

    #[test]
    fn disassemble_output_round_trips_to_the_same_bytes() {
        let binary = [
            0x00, 0x14, 0x01, 0x12, 0x34, 0x02, 0x03, 0x25, 0x5F, 0xA9, 0xB4, 0xB5, 0xC6, 0xCC,
            0xFF,
        ];
        let assembly = disassemble(&binary).unwrap();
        assert_eq!(assemble(&assembly).unwrap(), binary);
    }

    #[test]
    fn reports_truncated_immediate_instruction_with_offset() {
        assert_eq!(
            disassemble(&[0xC7, 0x01, 0x12]),
            Err(DisassemblyError {
                offset: 1,
                opcode: 0x01,
                expected_instruction_bytes: 3,
                available_bytes: 2,
            })
        );
    }
}
