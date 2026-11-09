use std::{convert::TryFrom, error::Error, fmt};

use super::micro_op::ALUOperation;

const SOURCES: [SourceRegister; 8] = [
    SourceRegister::PC,
    SourceRegister::ACC,
    SourceRegister::TempA,
    SourceRegister::TempB,
    SourceRegister::PSR,
    SourceRegister::SR,
    SourceRegister::AR,
    SourceRegister::DR,
];

const DESTINATIONS: [DestinationRegister; 4] = [
    DestinationRegister::ACC,
    DestinationRegister::AR,
    DestinationRegister::TempA,
    DestinationRegister::TempB,
];

const MOVES: [(SourceRegister, DestinationRegister); 28] = [
    (SourceRegister::PC, DestinationRegister::ACC),
    (SourceRegister::PC, DestinationRegister::AR),
    (SourceRegister::PC, DestinationRegister::TempA),
    (SourceRegister::PC, DestinationRegister::TempB),
    (SourceRegister::ACC, DestinationRegister::AR),
    (SourceRegister::ACC, DestinationRegister::TempA),
    (SourceRegister::ACC, DestinationRegister::TempB),
    (SourceRegister::TempA, DestinationRegister::ACC),
    (SourceRegister::TempA, DestinationRegister::AR),
    (SourceRegister::TempA, DestinationRegister::TempB),
    (SourceRegister::TempB, DestinationRegister::ACC),
    (SourceRegister::TempB, DestinationRegister::AR),
    (SourceRegister::TempB, DestinationRegister::TempA),
    (SourceRegister::PSR, DestinationRegister::ACC),
    (SourceRegister::PSR, DestinationRegister::AR),
    (SourceRegister::PSR, DestinationRegister::TempA),
    (SourceRegister::PSR, DestinationRegister::TempB),
    (SourceRegister::SR, DestinationRegister::ACC),
    (SourceRegister::SR, DestinationRegister::AR),
    (SourceRegister::SR, DestinationRegister::TempA),
    (SourceRegister::SR, DestinationRegister::TempB),
    (SourceRegister::AR, DestinationRegister::ACC),
    (SourceRegister::AR, DestinationRegister::TempA),
    (SourceRegister::AR, DestinationRegister::TempB),
    (SourceRegister::DR, DestinationRegister::ACC),
    (SourceRegister::DR, DestinationRegister::AR),
    (SourceRegister::DR, DestinationRegister::TempA),
    (SourceRegister::DR, DestinationRegister::TempB),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceRegister {
    PC,
    ACC,
    TempA,
    TempB,
    PSR,
    SR,
    AR,
    DR,
}

impl SourceRegister {
    fn index(self) -> usize {
        SOURCES
            .iter()
            .position(|&register| register == self)
            .unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DestinationRegister {
    ACC,
    AR,
    TempA,
    TempB,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BranchCondition {
    Always,
    Zero,
    NotZero,
    Negative,
    NotNegative,
    Carry,
    NotCarry,
    Overflow,
    NotOverflow,
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterOrEqual,
    LessOrEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Instruction {
    LoadImmediate,
    LoadAddress,
    StoreAddress,
    Move {
        source: SourceRegister,
        destination: DestinationRegister,
    },
    Compare {
        left: SourceRegister,
        right: SourceRegister,
    },
    Alu {
        operation: ALUOperation,
        source: SourceRegister,
    },
    Unary(ALUOperation),
    Push(SourceRegister),
    Pop(DestinationRegister),
    Jump(BranchCondition),
    Call,
    Return,
    Halt,
    Nop,
    Interrupt,
    InterruptReturn,
    EnableInterrupts,
    DisableInterrupts,
}

/// A validated, single-byte machine opcode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Opcode(u8);

#[allow(non_upper_case_globals)]
impl Opcode {
    pub const LDI: Self = Self(0x00);
    pub const LDA: Self = Self(0x01);
    pub const STA: Self = Self(0x02);
    pub const INC: Self = Self(0x9F);
    pub const DEC: Self = Self(0xA0);
    pub const NOT: Self = Self(0xA1);
    pub const JMP: Self = Self(0xB5);
    pub const JZ: Self = Self(0xB6);
    pub const JNZ: Self = Self(0xB7);
    pub const CALL: Self = Self(0xC4);
    pub const RET: Self = Self(0xC5);
    pub const HLT: Self = Self(0xC6);
    pub const NOP: Self = Self(0xC7);
    pub const INT: Self = Self(0xC8);
    pub const IRET: Self = Self(0xC9);
    pub const EI: Self = Self(0xCA);
    pub const DI: Self = Self(0xCB);

    pub const fn code(self) -> u8 {
        self.0
    }

    pub fn instruction(self) -> Instruction {
        instruction_from_code(self.0)
            .expect("Opcode values can only be constructed for assigned instructions")
    }

    pub fn length(self) -> u8 {
        self.instruction().length()
    }
}

impl Instruction {
    pub const fn length(self) -> u8 {
        match self {
            Self::LoadImmediate | Self::Interrupt => 2,
            Self::LoadAddress => 3,
            _ => 1,
        }
    }

    pub fn opcode(self) -> Option<Opcode> {
        let code = match self {
            Self::LoadImmediate => 0x00,
            Self::LoadAddress => 0x01,
            Self::StoreAddress => 0x02,
            Self::Move {
                source,
                destination,
            } => {
                let index = MOVES
                    .iter()
                    .position(|&pair| pair == (source, destination))?;
                0x03 + index as u8
            }
            Self::Compare { left, right } => {
                0x1F + (left.index() * SOURCES.len() + right.index()) as u8
            }
            Self::Alu { operation, source } => {
                let operation_index = alu_operation_index(operation)?;
                0x5F + (operation_index * SOURCES.len() + source.index()) as u8
            }
            Self::Unary(operation) => 0x9F + unary_operation_index(operation)?,
            Self::Push(source) => 0xA9 + source.index() as u8,
            Self::Pop(destination) => {
                let index = DESTINATIONS
                    .iter()
                    .position(|&register| register == destination)
                    .unwrap();
                0xB1 + index as u8
            }
            Self::Jump(condition) => {
                0xB5 + match condition {
                    BranchCondition::Always => 0,
                    BranchCondition::Zero => 1,
                    BranchCondition::NotZero => 2,
                    BranchCondition::Negative => 3,
                    BranchCondition::NotNegative => 4,
                    BranchCondition::Carry => 5,
                    BranchCondition::NotCarry => 6,
                    BranchCondition::Overflow => 7,
                    BranchCondition::NotOverflow => 8,
                    BranchCondition::Equal => 9,
                    BranchCondition::NotEqual => 10,
                    BranchCondition::Greater => 11,
                    BranchCondition::Less => 12,
                    BranchCondition::GreaterOrEqual => 13,
                    BranchCondition::LessOrEqual => 14,
                }
            }
            Self::Call => 0xC4,
            Self::Return => 0xC5,
            Self::Halt => 0xC6,
            Self::Nop => 0xC7,
            Self::Interrupt => 0xC8,
            Self::InterruptReturn => 0xC9,
            Self::EnableInterrupts => 0xCA,
            Self::DisableInterrupts => 0xCB,
        };
        Some(Opcode(code))
    }
}

fn alu_operation_index(operation: ALUOperation) -> Option<usize> {
    Some(match operation {
        ALUOperation::Add => 0,
        ALUOperation::Adc => 1,
        ALUOperation::Subtract => 2,
        ALUOperation::Sbc => 3,
        ALUOperation::And => 4,
        ALUOperation::Or => 5,
        ALUOperation::Xor => 6,
        ALUOperation::Test => 7,
        _ => return None,
    })
}

fn unary_operation_index(operation: ALUOperation) -> Option<u8> {
    Some(match operation {
        ALUOperation::Increment => 0,
        ALUOperation::Decrement => 1,
        ALUOperation::Not => 2,
        ALUOperation::ShiftLeft => 3,
        ALUOperation::ShiftRight => 4,
        ALUOperation::ShiftArithmeticRight => 5,
        ALUOperation::RotateLeft => 6,
        ALUOperation::RotateRight => 7,
        ALUOperation::RotateCarryLeft => 8,
        ALUOperation::RotateCarryRight => 9,
        _ => return None,
    })
}

impl TryFrom<u8> for Opcode {
    type Error = InvalidOpcode;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if instruction_from_code(value).is_some() {
            Ok(Self(value))
        } else {
            Err(InvalidOpcode(value))
        }
    }
}

impl From<Opcode> for u8 {
    fn from(opcode: Opcode) -> Self {
        opcode.0
    }
}

impl fmt::Display for InvalidOpcode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "unassigned opcode 0x{:02X}", self.0)
    }
}

impl Error for InvalidOpcode {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidOpcode(pub u8);

fn instruction_from_code(code: u8) -> Option<Instruction> {
    let instruction = match code {
        0x00 => Instruction::LoadImmediate,
        0x01 => Instruction::LoadAddress,
        0x02 => Instruction::StoreAddress,
        0x03..=0x1E => {
            let (source, destination) = MOVES[(code - 0x03) as usize];
            Instruction::Move {
                source,
                destination,
            }
        }
        0x1F..=0x5E => {
            let index = (code - 0x1F) as usize;
            Instruction::Compare {
                left: SOURCES[index / SOURCES.len()],
                right: SOURCES[index % SOURCES.len()],
            }
        }
        0x5F..=0x9E => {
            let index = (code - 0x5F) as usize;
            let operation = match index / SOURCES.len() {
                0 => ALUOperation::Add,
                1 => ALUOperation::Adc,
                2 => ALUOperation::Subtract,
                3 => ALUOperation::Sbc,
                4 => ALUOperation::And,
                5 => ALUOperation::Or,
                6 => ALUOperation::Xor,
                7 => ALUOperation::Test,
                _ => unreachable!(),
            };
            Instruction::Alu {
                operation,
                source: SOURCES[index % SOURCES.len()],
            }
        }
        0x9F..=0xA8 => Instruction::Unary(match code {
            0x9F => ALUOperation::Increment,
            0xA0 => ALUOperation::Decrement,
            0xA1 => ALUOperation::Not,
            0xA2 => ALUOperation::ShiftLeft,
            0xA3 => ALUOperation::ShiftRight,
            0xA4 => ALUOperation::ShiftArithmeticRight,
            0xA5 => ALUOperation::RotateLeft,
            0xA6 => ALUOperation::RotateRight,
            0xA7 => ALUOperation::RotateCarryLeft,
            0xA8 => ALUOperation::RotateCarryRight,
            _ => unreachable!(),
        }),
        0xA9..=0xB0 => Instruction::Push(SOURCES[(code - 0xA9) as usize]),
        0xB1..=0xB4 => Instruction::Pop(DESTINATIONS[(code - 0xB1) as usize]),
        0xB5..=0xC3 => Instruction::Jump(match code {
            0xB5 => BranchCondition::Always,
            0xB6 => BranchCondition::Zero,
            0xB7 => BranchCondition::NotZero,
            0xB8 => BranchCondition::Negative,
            0xB9 => BranchCondition::NotNegative,
            0xBA => BranchCondition::Carry,
            0xBB => BranchCondition::NotCarry,
            0xBC => BranchCondition::Overflow,
            0xBD => BranchCondition::NotOverflow,
            0xBE => BranchCondition::Equal,
            0xBF => BranchCondition::NotEqual,
            0xC0 => BranchCondition::Greater,
            0xC1 => BranchCondition::Less,
            0xC2 => BranchCondition::GreaterOrEqual,
            0xC3 => BranchCondition::LessOrEqual,
            _ => unreachable!(),
        }),
        0xC4 => Instruction::Call,
        0xC5 => Instruction::Return,
        0xC6 => Instruction::Halt,
        0xC7 => Instruction::Nop,
        0xC8 => Instruction::Interrupt,
        0xC9 => Instruction::InterruptReturn,
        0xCA => Instruction::EnableInterrupts,
        0xCB => Instruction::DisableInterrupts,
        _ => return None,
    };
    Some(instruction)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_assigned_opcodes_decode_and_round_trip() {
        let mut assigned = 0;

        for value in u8::MIN..=u8::MAX {
            match Opcode::try_from(value) {
                Ok(opcode) => {
                    assigned += 1;
                    assert_eq!(opcode.instruction().opcode(), Some(opcode));
                }
                Err(error) => assert_eq!(error, InvalidOpcode(value)),
            }
        }

        assert_eq!(assigned, 204);
    }

    #[test]
    fn register_specific_opcodes_match_the_allocation() {
        assert_eq!(
            Opcode::try_from(0x5F).unwrap().instruction(),
            Instruction::Alu {
                operation: ALUOperation::Add,
                source: SourceRegister::PC,
            }
        );
        assert_eq!(
            Opcode::try_from(0x60).unwrap().instruction(),
            Instruction::Alu {
                operation: ALUOperation::Add,
                source: SourceRegister::ACC,
            }
        );
        assert_eq!(Opcode::try_from(0x5F).unwrap().length(), 1);
        assert_eq!(Opcode::LDI.length(), 2);
        assert_eq!(Opcode::LDA.length(), 3);
        assert_eq!(Opcode::INT.length(), 2);
    }

    #[test]
    fn move_encoding_excludes_self_moves_and_internal_stack_pointer() {
        assert_eq!(
            Opcode::try_from(0x03).unwrap().instruction(),
            Instruction::Move {
                source: SourceRegister::PC,
                destination: DestinationRegister::ACC,
            }
        );
        assert_eq!(
            Opcode::try_from(0x1E).unwrap().instruction(),
            Instruction::Move {
                source: SourceRegister::DR,
                destination: DestinationRegister::TempB,
            }
        );
        assert!(MOVES.iter().all(|(source, destination)| {
            !matches!(
                (source, destination),
                (SourceRegister::ACC, DestinationRegister::ACC)
                    | (SourceRegister::AR, DestinationRegister::AR)
                    | (SourceRegister::TempA, DestinationRegister::TempA)
                    | (SourceRegister::TempB, DestinationRegister::TempB)
            )
        }));
        assert_eq!(
            Instruction::Move {
                source: SourceRegister::ACC,
                destination: DestinationRegister::ACC,
            }
            .opcode(),
            None
        );
    }
}
