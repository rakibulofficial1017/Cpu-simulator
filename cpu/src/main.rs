#![allow(warnings)]
mod cpu;
mod hardware;

use cpu::computer::{Computer, SCREEN_HEIGHT, SCREEN_START, SCREEN_WIDTH};
use cpu::decoder::Decoder;
use cpu::instruction::Instruction;
use std::{env, error::Error, fs, io, path::Path};

const GLYPH_WIDTH: usize = 5;
const GLYPH_HEIGHT: usize = 7;
const CHARACTER_SPACING: usize = 1;
const LINE_SPACING: usize = 3;
const TEXT_COLOR: [u8; 3] = [255, 220, 80];

fn glyph(character: char) -> Option<[u8; GLYPH_HEIGHT]> {
    Some(match character {
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        ' ' => [0; GLYPH_HEIGHT],
        _ => return None,
    })
}

fn draw_text(computer: &mut Computer, text: &str, y: usize) {
    let text_width = text.len() * (GLYPH_WIDTH + CHARACTER_SPACING) - CHARACTER_SPACING;
    let start_x = (SCREEN_WIDTH - text_width) / 2;

    for (character_index, character) in text.chars().enumerate() {
        let rows = glyph(character).expect("demo text uses supported characters");
        let character_x = start_x + character_index * (GLYPH_WIDTH + CHARACTER_SPACING);

        for (row_index, row) in rows.iter().enumerate() {
            for column in 0..GLYPH_WIDTH {
                if row & (1 << (GLYPH_WIDTH - column - 1)) == 0 {
                    continue;
                }

                let x = character_x + column;
                let screen_y = y + row_index;
                let pixel = screen_y * SCREEN_WIDTH + x;
                let address = SCREEN_START as usize + pixel * 3;

                for (channel, value) in TEXT_COLOR.iter().enumerate() {
                    computer
                        .ram
                        .update(*value, true, (address + channel) as u16, true);
                }
            }
        }
    }
}

fn execute_program(computer: &mut Computer, program: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut offset = 0;

    while offset < program.len() {
        let opcode = program[offset];
        let decoded = Decoder::decode(opcode).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid opcode at byte {offset}: {error}"),
            )
        })?;
        let operand_count = decoded.length() as usize - 1;
        let instruction_end = offset + 1 + operand_count;
        if instruction_end > program.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!(
                    "truncated {:?} at byte {offset}: expected {operand_count} operand byte(s)",
                    decoded.instruction
                ),
            )
            .into());
        }

        computer
            .cpu
            .execute_instruction_with_operands(opcode, &program[offset + 1..instruction_end])
            .map_err(|error| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("failed to execute instruction at byte {offset}: {error}"),
                )
            })?;

        if computer.cpu.memory_request.read || computer.cpu.memory_request.write {
            while !computer.tick() {}
        }

        offset = instruction_end;
        if decoded.instruction == Instruction::Halt {
            break;
        }
    }

    Ok(())
}

fn run_binary(computer: &mut Computer, path: &Path) -> Result<(), Box<dyn Error>> {
    if path.extension().and_then(|extension| extension.to_str()) != Some("bin") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "program path must have a .bin extension",
        )
        .into());
    }

    let program = fs::read(path)?;
    execute_program(computer, &program)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut computer = Computer::new();

    let mut args = env::args_os().skip(1);
    match (args.next(), args.next()) {
        (Some(path), None) => run_binary(&mut computer, Path::new(&path))?,
        (None, None) => {
            let text_height = GLYPH_HEIGHT * 2 + LINE_SPACING;
            let first_line_y = (SCREEN_HEIGHT - text_height) / 2;

            draw_text(&mut computer, "HELLO", first_line_y);
            draw_text(
                &mut computer,
                "WORLD",
                first_line_y + GLYPH_HEIGHT + LINE_SPACING,
            );
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "usage: cpu [path/to/program.bin]",
            )
            .into());
        }
    }

    computer.update_screen()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn program_runner_executes_operand_instructions_and_stops_at_halt() {
        let mut computer = Computer::new();
        let program = [
            0x00, 0x14, // LDI 0x14
            0x01, 0x10, 0x00, // LDA 0x1000
            0x02, // STA
            0xC6, // HLT
            0x00, 0x99, // Must not execute
        ];

        execute_program(&mut computer, &program).unwrap();

        assert_eq!(computer.ram.output(0x1000), 0x14);
        assert_eq!(computer.cpu.acc.output(), 0x14);
    }

    #[test]
    fn program_runner_reports_truncated_operand_bytes() {
        let mut computer = Computer::new();
        let error = execute_program(&mut computer, &[0x01, 0x10]).unwrap_err();

        assert!(error
            .to_string()
            .contains("truncated LoadAddress at byte 0"));
    }
}
