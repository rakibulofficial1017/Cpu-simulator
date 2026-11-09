use crate::cpu::instruction::{BranchCondition, DestinationRegister, Instruction, SourceRegister};
use crate::cpu::micro_op::ALUOperation;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssemblyError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for AssemblyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for AssemblyError {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Number(u32),
    Label(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Statement {
    Byte(Vec<Value>),
    Instruction(Instruction),
    Immediate(Instruction, Value),
    Transfer(Instruction),
    Branch(Instruction, Option<Value>),
    Store(Value, Value),
}

#[derive(Debug)]
struct SourceStatement {
    line: usize,
    statement: Statement,
}

impl Statement {
    fn encoded_len(&self) -> usize {
        match self {
            Self::Byte(values) => values.len(),
            Self::Instruction(instruction) => instruction.length() as usize,
            Self::Immediate(instruction, _) => instruction.length() as usize,
            Self::Transfer(_) => 1,
            Self::Branch(_, None) => 1,
            Self::Branch(_, Some(_)) => 4,
            Self::Store(_, _) => 6,
        }
    }
}

pub fn assemble(source: &str) -> Result<Vec<u8>, AssemblyError> {
    let mut statements = Vec::new();
    let mut labels = HashMap::new();
    let mut address = 0usize;

    for (index, raw_line) in source.lines().enumerate() {
        let line = index + 1;
        let line_content = raw_line.split(';').next().unwrap_or("").trim();
        if line_content.is_empty() {
            continue;
        }

        let (label, instruction_text) = split_label(line_content, line)?;
        if let Some(label) = label {
            if labels.insert(label.clone(), address).is_some() {
                return Err(error(line, format!("duplicate label '{label}'")));
            }
        }

        if instruction_text.is_empty() {
            continue;
        }

        let statement = parse_statement(instruction_text, line)?;
        address = address
            .checked_add(statement.encoded_len())
            .ok_or_else(|| error(line, "program size overflow"))?;
        if address > u16::MAX as usize + 1 {
            return Err(error(line, "program exceeds the 16-bit address space"));
        }
        statements.push(SourceStatement { line, statement });
    }

    let mut output = Vec::with_capacity(address);
    for source_statement in statements {
        encode_statement(
            &source_statement.statement,
            source_statement.line,
            &labels,
            &mut output,
        )?;
    }

    debug_assert_eq!(output.len(), address);
    Ok(output)
}

fn split_label(line_content: &str, line: usize) -> Result<(Option<String>, &str), AssemblyError> {
    let Some((candidate, rest)) = line_content.split_once(':') else {
        return Ok((None, line_content));
    };

    if candidate.is_empty()
        || !candidate
            .chars()
            .enumerate()
            .all(|(index, ch)| is_label_char(ch, index == 0))
    {
        return Err(error(line, format!("invalid label '{candidate}'")));
    }

    Ok((Some(candidate.to_string()), rest.trim()))
}

fn is_label_char(ch: char, first: bool) -> bool {
    ch == '_' || ch.is_ascii_alphabetic() || (!first && ch.is_ascii_digit())
}

fn parse_statement(text: &str, line: usize) -> Result<Statement, AssemblyError> {
    let (mnemonic, operands) = text
        .split_once(char::is_whitespace)
        .map_or((text, ""), |(mnemonic, operands)| {
            (mnemonic, operands.trim())
        });
    let mnemonic = mnemonic.to_ascii_uppercase();

    if mnemonic == ".BYTE" {
        let values = comma_separated(operands, line)?;
        if values.is_empty() {
            return Err(error(line, ".byte requires at least one value"));
        }
        return Ok(Statement::Byte(
            values
                .into_iter()
                .map(|value| parse_value(value, line))
                .collect::<Result<_, _>>()?,
        ));
    }

    match mnemonic.as_str() {
        "LDI" | "LOAD" => Ok(Statement::Immediate(
            Instruction::LoadImmediate,
            parse_immediate(operands, line)?,
        )),
        "LDA" | "LOAD_ADDRESS" => Ok(Statement::Immediate(
            Instruction::LoadAddress,
            parse_immediate(operands, line)?,
        )),
        "INT" => Ok(Statement::Immediate(
            Instruction::Interrupt,
            parse_immediate(operands, line)?,
        )),
        "STA" => no_operands(operands, line, Instruction::StoreAddress),
        "NOP" => no_operands(operands, line, Instruction::Nop),
        "HLT" => no_operands(operands, line, Instruction::Halt),
        "RET" => no_operands(operands, line, Instruction::Return),
        "IRET" => no_operands(operands, line, Instruction::InterruptReturn),
        "EI" => no_operands(operands, line, Instruction::EnableInterrupts),
        "DI" => no_operands(operands, line, Instruction::DisableInterrupts),
        "MOV" | "COPY" => {
            let (source, destination) = parse_arrow_operands(operands, line)?;
            Ok(Statement::Transfer(Instruction::Move {
                source: parse_source(source, line)?,
                destination: parse_destination(destination, line)?,
            }))
        }
        "CMP" => {
            let operands = comma_separated(operands, line)?;
            if operands.len() != 2 {
                return Err(error(line, "CMP expects two comma-separated registers"));
            }
            Ok(Statement::Instruction(Instruction::Compare {
                left: parse_source(operands[0], line)?,
                right: parse_source(operands[1], line)?,
            }))
        }
        "PUSH" => Ok(Statement::Instruction(Instruction::Push(parse_source(
            single_operand(operands, line)?,
            line,
        )?))),
        "POP" => Ok(Statement::Instruction(Instruction::Pop(parse_destination(
            single_operand(operands, line)?,
            line,
        )?))),
        "ADD" | "ADC" | "SUB" | "SBC" | "AND" | "OR" | "XOR" | "TEST" => {
            let operation = match mnemonic.as_str() {
                "ADD" => ALUOperation::Add,
                "ADC" => ALUOperation::Adc,
                "SUB" => ALUOperation::Subtract,
                "SBC" => ALUOperation::Sbc,
                "AND" => ALUOperation::And,
                "OR" => ALUOperation::Or,
                "XOR" => ALUOperation::Xor,
                "TEST" => ALUOperation::Test,
                _ => unreachable!(),
            };
            Ok(Statement::Instruction(Instruction::Alu {
                operation,
                source: parse_source(single_operand(operands, line)?, line)?,
            }))
        }
        "INC" | "DEC" | "NOT" | "SHL" | "SHR" | "SAR" | "ROL" | "ROR" | "RCL" | "RCR" => {
            no_operands(
                operands,
                line,
                Instruction::Unary(match mnemonic.as_str() {
                    "INC" => ALUOperation::Increment,
                    "DEC" => ALUOperation::Decrement,
                    "NOT" => ALUOperation::Not,
                    "SHL" => ALUOperation::ShiftLeft,
                    "SHR" => ALUOperation::ShiftRight,
                    "SAR" => ALUOperation::ShiftArithmeticRight,
                    "ROL" => ALUOperation::RotateLeft,
                    "ROR" => ALUOperation::RotateRight,
                    "RCL" => ALUOperation::RotateCarryLeft,
                    "RCR" => ALUOperation::RotateCarryRight,
                    _ => unreachable!(),
                }),
            )
        }
        "JMP" | "JZ" | "JNZ" | "JN" | "JNN" | "JC" | "JNC" | "JV" | "JNV" | "JE" | "JNE"
        | "JGT" | "JLT" | "JGE" | "JLE" | "CALL" => {
            let instruction = if mnemonic == "CALL" {
                Instruction::Call
            } else {
                Instruction::Jump(match mnemonic.as_str() {
                    "JMP" => BranchCondition::Always,
                    "JZ" => BranchCondition::Zero,
                    "JNZ" => BranchCondition::NotZero,
                    "JN" => BranchCondition::Negative,
                    "JNN" => BranchCondition::NotNegative,
                    "JC" => BranchCondition::Carry,
                    "JNC" => BranchCondition::NotCarry,
                    "JV" => BranchCondition::Overflow,
                    "JNV" => BranchCondition::NotOverflow,
                    "JE" => BranchCondition::Equal,
                    "JNE" => BranchCondition::NotEqual,
                    "JGT" => BranchCondition::Greater,
                    "JLT" => BranchCondition::Less,
                    "JGE" => BranchCondition::GreaterOrEqual,
                    "JLE" => BranchCondition::LessOrEqual,
                    _ => unreachable!(),
                })
            };

            if operands.is_empty() {
                Ok(Statement::Branch(instruction, None))
            } else {
                let operand = single_operand(operands, line)?;
                Ok(Statement::Branch(
                    instruction,
                    Some(parse_value(operand, line)?),
                ))
            }
        }
        "STORE" => {
            let values = comma_separated(operands, line)?;
            if values.len() != 2 {
                return Err(error(line, "STORE expects a value and an address"));
            }
            Ok(Statement::Store(
                parse_immediate(values[0], line)?,
                parse_immediate(values[1], line)?,
            ))
        }
        _ => Err(error(
            line,
            format!("unknown instruction or directive '{mnemonic}'"),
        )),
    }
}

fn no_operands(
    operands: &str,
    line: usize,
    instruction: Instruction,
) -> Result<Statement, AssemblyError> {
    if !operands.is_empty() {
        return Err(error(line, format!("{instruction:?} takes no operands")));
    }
    Ok(Statement::Instruction(instruction))
}

fn single_operand<'a>(operands: &'a str, line: usize) -> Result<&'a str, AssemblyError> {
    if operands.is_empty() || operands.contains(',') {
        return Err(error(line, "expected exactly one operand"));
    }
    Ok(operands.trim())
}

fn comma_separated<'a>(operands: &'a str, line: usize) -> Result<Vec<&'a str>, AssemblyError> {
    if operands.is_empty() {
        return Ok(Vec::new());
    }
    let values: Vec<_> = operands.split(',').map(str::trim).collect();
    if values.iter().any(|value| value.is_empty()) {
        return Err(error(line, "empty comma-separated operand"));
    }
    Ok(values)
}

fn parse_arrow_operands<'a>(
    operands: &'a str,
    line: usize,
) -> Result<(&'a str, &'a str), AssemblyError> {
    let Some((source, destination)) = operands.split_once("->") else {
        return Err(error(line, "expected 'source -> destination'"));
    };
    if source.trim().is_empty()
        || destination.trim().is_empty()
        || destination.contains("->")
        || source.contains(',')
        || destination.contains(',')
    {
        return Err(error(line, "expected exactly 'source -> destination'"));
    }
    Ok((source.trim(), destination.trim()))
}

fn parse_immediate(operand: &str, line: usize) -> Result<Value, AssemblyError> {
    let value = operand
        .strip_prefix('#')
        .ok_or_else(|| error(line, "immediate operands must start with '#'"))?;
    parse_value(value, line)
}

fn parse_value(value: &str, line: usize) -> Result<Value, AssemblyError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(error(line, "missing value"));
    }
    if let Some(number) = parse_number(value) {
        return Ok(Value::Number(number));
    }
    if value
        .chars()
        .enumerate()
        .all(|(index, ch)| is_label_char(ch, index == 0))
    {
        return Ok(Value::Label(value.to_string()));
    }
    Err(error(line, format!("invalid value '{value}'")))
}

fn parse_number(value: &str) -> Option<u32> {
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        u32::from_str_radix(hex, 16).ok()
    } else if let Some(binary) = value
        .strip_prefix("0b")
        .or_else(|| value.strip_prefix("0B"))
    {
        u32::from_str_radix(binary, 2).ok()
    } else {
        value.parse().ok()
    }
}

fn parse_source(source: &str, line: usize) -> Result<SourceRegister, AssemblyError> {
    match source.to_ascii_uppercase().as_str() {
        "PC" => Ok(SourceRegister::PC),
        "ACC" => Ok(SourceRegister::ACC),
        "TEMPA" => Ok(SourceRegister::TempA),
        "TEMPB" => Ok(SourceRegister::TempB),
        "PSR" => Ok(SourceRegister::PSR),
        "SR" => Ok(SourceRegister::SR),
        "AR" => Ok(SourceRegister::AR),
        "DR" => Ok(SourceRegister::DR),
        _ => Err(error(line, format!("unknown source register '{source}'"))),
    }
}

fn parse_destination(destination: &str, line: usize) -> Result<DestinationRegister, AssemblyError> {
    match destination.to_ascii_uppercase().as_str() {
        "ACC" => Ok(DestinationRegister::ACC),
        "AR" => Ok(DestinationRegister::AR),
        "TEMPA" => Ok(DestinationRegister::TempA),
        "TEMPB" => Ok(DestinationRegister::TempB),
        _ => Err(error(
            line,
            format!("invalid destination register '{destination}'"),
        )),
    }
}

fn encode_statement(
    statement: &Statement,
    line: usize,
    labels: &HashMap<String, usize>,
    output: &mut Vec<u8>,
) -> Result<(), AssemblyError> {
    match statement {
        Statement::Byte(values) => {
            for value in values {
                output.push(resolve_byte(value, line, labels)?);
            }
        }
        Statement::Instruction(instruction) | Statement::Transfer(instruction) => {
            output.push(encode_opcode(*instruction, line)?);
        }
        Statement::Immediate(instruction, value) => {
            output.push(encode_opcode(*instruction, line)?);
            let value = resolve_value(value, line, labels)?;
            match instruction.length() {
                2 => output.push(u8::try_from(value).map_err(|_| {
                    error(line, format!("value {value:#X} does not fit in one byte"))
                })?),
                3 => output.extend_from_slice(
                    &u16::try_from(value)
                        .map_err(|_| {
                            error(line, format!("address {value:#X} does not fit in 16 bits"))
                        })?
                        .to_be_bytes(),
                ),
                _ => return Err(error(line, "unexpected immediate instruction width")),
            }
        }
        Statement::Branch(instruction, target) => {
            if let Some(target) = target {
                output.push(encode_opcode(Instruction::LoadAddress, line)?);
                let address = u16::try_from(resolve_value(target, line, labels)?)
                    .map_err(|_| error(line, "branch target does not fit in 16 bits"))?;
                output.extend_from_slice(&address.to_be_bytes());
            }
            output.push(encode_opcode(*instruction, line)?);
        }
        Statement::Store(value, address) => {
            output.push(encode_opcode(Instruction::LoadImmediate, line)?);
            output.push(resolve_byte(value, line, labels)?);
            output.push(encode_opcode(Instruction::LoadAddress, line)?);
            let address = u16::try_from(resolve_value(address, line, labels)?)
                .map_err(|_| error(line, "store address does not fit in 16 bits"))?;
            output.extend_from_slice(&address.to_be_bytes());
            output.push(encode_opcode(Instruction::StoreAddress, line)?);
        }
    }
    Ok(())
}

fn encode_opcode(instruction: Instruction, line: usize) -> Result<u8, AssemblyError> {
    instruction
        .opcode()
        .map(Into::into)
        .ok_or_else(|| error(line, format!("instruction {instruction:?} has no opcode")))
}

fn resolve_byte(
    value: &Value,
    line: usize,
    labels: &HashMap<String, usize>,
) -> Result<u8, AssemblyError> {
    let value = resolve_value(value, line, labels)?;
    u8::try_from(value)
        .map_err(|_| error(line, format!("value {value:#X} does not fit in one byte")))
}

fn resolve_value(
    value: &Value,
    line: usize,
    labels: &HashMap<String, usize>,
) -> Result<u32, AssemblyError> {
    match value {
        Value::Number(number) => Ok(*number),
        Value::Label(label) => labels
            .get(label)
            .copied()
            .and_then(|address| u32::try_from(address).ok())
            .ok_or_else(|| error(line, format!("undefined label '{label}'"))),
    }
}

fn error(line: usize, message: impl Into<String>) -> AssemblyError {
    AssemblyError {
        line,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assembles_native_instructions_and_operands() {
        let binary = assemble("LDI #0x14\nLDA #0x1234\nMOV TempA -> ACC\nCMP PC, AR\nHLT").unwrap();
        assert_eq!(binary, [0x00, 0x14, 0x01, 0x12, 0x34, 0x0A, 0x25, 0xC6]);
    }

    #[test]
    fn expands_simple_pseudo_instructions() {
        assert_eq!(
            assemble("LOAD #20\nSTORE #0x14, #0x1000\nCOPY PC -> ACC").unwrap(),
            [0x00, 0x14, 0x00, 0x14, 0x01, 0x10, 0x00, 0x02, 0x03]
        );
    }

    #[test]
    fn resolves_labels_for_branches_and_data() {
        assert_eq!(
            assemble("start: NOP\nJMP done\n.byte 0xAA\ndone: HLT").unwrap(),
            [0xC7, 0x01, 0x00, 0x06, 0xB5, 0xAA, 0xC6]
        );
    }

    #[test]
    fn rejects_bad_operands_and_undefined_labels_with_line_numbers() {
        let error = assemble("LDI #256").unwrap_err();
        assert_eq!(error.line, 1);
        assert!(error.message.contains("does not fit in one byte"));

        let error = assemble("JMP missing").unwrap_err();
        assert_eq!(error.line, 1);
        assert!(error.message.contains("undefined label"));
    }

    #[test]
    fn supports_labels_and_comments_on_same_line() {
        assert_eq!(
            assemble("entry: LDI #1 ; load one\nHLT").unwrap(),
            [0x00, 0x01, 0xC6]
        );
    }
}
