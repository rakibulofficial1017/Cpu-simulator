
use super::instruction::{InvalidOpcode, Opcode};
use std::convert::TryFrom;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodedInstruction {
    pub opcode: Opcode,
}

pub struct Decoder;

impl Decoder {
    pub fn decode(instruction: u8) -> Result<DecodedInstruction, InvalidOpcode> {
        let opcode = Opcode::try_from(instruction)?;

        Ok(DecodedInstruction { opcode })
    }
}
