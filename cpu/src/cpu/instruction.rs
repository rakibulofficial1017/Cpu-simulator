
use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    NOP  = 0x00,
    LDI  = 0x01,
    LDA  = 0x02,
    STA  = 0x03,
    ADD  = 0x04,
    SUB  = 0x05,
    JMP  = 0x06,
    JZ   = 0x07,
    PUSH = 0x08,
    POP  = 0x09,
    HLT  = 0xFF,
}

impl TryFrom<u8> for Opcode {
    type Error = InvalidOpcode;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::NOP),
            0x01 => Ok(Self::LDI),
            0x02 => Ok(Self::LDA),
            0x03 => Ok(Self::STA),
            0x04 => Ok(Self::ADD),
            0x05 => Ok(Self::SUB),
            0x06 => Ok(Self::JMP),
            0x07 => Ok(Self::JZ),
            0x08 => Ok(Self::PUSH),
            0x09 => Ok(Self::POP),
            0xFF => Ok(Self::HLT),
            _ => Err(InvalidOpcode(value)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidOpcode(pub u8);

impl Opcode {
    pub fn length(self) -> u8 {
        match self {
            Self::NOP | Self::ADD | Self::SUB |
            Self::PUSH | Self::POP | Self::HLT => 1,

            Self::LDI => 2,

            Self::LDA | Self::STA | Self::JMP | Self::JZ => 3,
        }
    }
}
