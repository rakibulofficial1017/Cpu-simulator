#![allow(warnings)]
mod assembler;
mod cpu;
mod disassembler;
mod hardware;

use cpu::computer::{Computer, SCREEN_HEIGHT, SCREEN_START, SCREEN_WIDTH};
use cpu::decoder::Decoder;
use cpu::instruction::Instruction;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::{
    env,
    error::Error,
    fs,
    io::{self, BufRead, Write},
    path::Path,
    time::Duration,
};

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
    execute_program_with_input(computer, program, false)
}

fn execute_program_with_input(
    computer: &mut Computer,
    program: &[u8],
    interactive: bool,
) -> Result<(), Box<dyn Error>> {
    let mut offset = 0;

    while offset < program.len() {
        if interactive && event::poll(Duration::from_millis(1))? {
            computer.handle_input_event(event::read()?);
        }

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

fn execute_interactive_program(
    computer: &mut Computer,
    program: &[u8],
) -> Result<(), Box<dyn Error>> {
    let mut offset = 0usize;
    let mut rendered_position = None;
    loop {
        if event::poll(Duration::from_millis(1))? {
            let input_event = event::read()?;
            if matches!(
                input_event,
                Event::Key(key)
                    if key.code == crossterm::event::KeyCode::Esc
                        && key.kind == crossterm::event::KeyEventKind::Press
            ) {
                break;
            }
            computer.handle_input_event(input_event);
        }

        let Some(next_offset) = execute_interactive_step(computer, program, offset)? else {
            break;
        };

        let position = (
            computer.ram.output(cpu::computer::BOARD_X),
            computer.ram.output(cpu::computer::BOARD_Y),
        );
        if rendered_position != Some(position) {
            if let Some((x, y)) = rendered_position {
                computer.set_pixel(x as usize, y as usize, [0, 0, 0])?;
            }
            computer.set_pixel(position.0 as usize, position.1 as usize, [255, 220, 80])?;
            computer.update_screen()?;
            rendered_position = Some(position);
        }

        offset = next_offset;
    }

    Ok(())
}

fn execute_interactive_step(
    computer: &mut Computer,
    program: &[u8],
    offset: usize,
) -> Result<Option<usize>, Box<dyn Error>> {
    if offset == program.len() {
        return Ok(None);
    }
    let Some(&opcode) = program.get(offset) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("program counter {offset} is outside the binary"),
        )
        .into());
    };
    let decoded = Decoder::decode(opcode).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid opcode at byte {offset}: {error}"),
        )
    })?;
    let instruction_end = offset + decoded.length() as usize;
    if instruction_end > program.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("truncated {:?} at byte {offset}", decoded.instruction),
        )
        .into());
    }

    computer
        .cpu
        .pc
        .update(instruction_end as u16, true, false, false, true);
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

    if computer.cpu.halted {
        Ok(None)
    } else if matches!(decoded.instruction, Instruction::Jump(_)) {
        Ok(Some(computer.cpu.pc.output() as usize))
    } else {
        Ok(Some(instruction_end))
    }
}

fn cpu_state(computer: &Computer) -> String {
    let cpu = &computer.cpu;
    let flags = cpu.sr.output();
    format!(
        "PC    0x{:04X} ({:5})   ACC   0x{:02X} ({:3})\n\
         TempA 0x{:04X} ({:5})   TempB 0x{:04X} ({:5})\n\
         PSR   0x{:04X} ({:5})   SR    0x{:02X} ({:3})\n\
         AR    0x{:04X} ({:5})   DR    0x{:02X} ({:3})\n\
         SP    0x{:04X} ({:5})   IR    0x{:02X} ({:3})\n\
         Flags: ZR={} NG={} CY={} OV={} EQ={} GT={} LT={}  Halted={}",
        cpu.pc.output(),
        cpu.pc.output(),
        cpu.acc.output(),
        cpu.acc.output(),
        cpu.temp_a.output(),
        cpu.temp_a.output(),
        cpu.temp_b.output(),
        cpu.temp_b.output(),
        cpu.psr.output(),
        cpu.psr.output(),
        flags,
        flags,
        cpu.ar.output(),
        cpu.ar.output(),
        cpu.dr.output(),
        cpu.dr.output(),
        cpu.sp.output(),
        cpu.sp.output(),
        cpu.ir.output(),
        cpu.ir.output(),
        u8::from(flags & 0x80 != 0),
        u8::from(flags & 0x40 != 0),
        u8::from(flags & 0x20 != 0),
        u8::from(flags & 0x10 != 0),
        u8::from(flags & 0x08 != 0),
        u8::from(flags & 0x04 != 0),
        u8::from(flags & 0x02 != 0),
        cpu.halted
    )
}

fn parse_debug_number(value: &str) -> Result<usize, String> {
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        usize::from_str_radix(hex, 16).map_err(|error| error.to_string())
    } else {
        value.parse::<usize>().map_err(|error| error.to_string())
    }
}

fn memory_dump(computer: &Computer, start: usize, length: usize) -> Result<String, String> {
    const RAM_SIZE: usize = u16::MAX as usize + 1;
    if length == 0 || length > 256 {
        return Err("memory length must be between 1 and 256 bytes".to_string());
    }
    let end = start
        .checked_add(length)
        .ok_or_else(|| "memory range overflows".to_string())?;
    if end > RAM_SIZE {
        return Err(format!(
            "memory range 0x{start:04X}..0x{end:05X} exceeds 64 KiB RAM"
        ));
    }

    let mut output = String::new();
    for row_start in (start..end).step_by(16) {
        let row_end = (row_start + 16).min(end);
        output.push_str(&format!("{row_start:04X}: "));
        for address in row_start..row_start + 16 {
            if address < row_end {
                output.push_str(&format!("{:02X} ", computer.ram.output(address as u16)));
            } else {
                output.push_str("   ");
            }
        }
        output.push_str("|");
        for address in row_start..row_end {
            let byte = computer.ram.output(address as u16);
            output.push(if byte.is_ascii_graphic() || byte == b' ' {
                byte as char
            } else {
                '.'
            });
        }
        output.push_str("|\n");
    }
    Ok(output)
}

fn debug_program(computer: &mut Computer, program: &[u8]) -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut offset = 0usize;
    let mut line = String::new();
    println!("Debugger commands: step, continue [count], regs, mem <address> [length], screen, help, quit");
    println!("{}", cpu_state(computer));

    loop {
        print!("debug[0x{offset:04X}]> ");
        io::stdout().flush()?;
        line.clear();
        if input.read_line(&mut line)? == 0 {
            break;
        }
        let mut parts = line.split_whitespace();
        let Some(command) = parts.next() else {
            continue;
        };
        match command.to_ascii_lowercase().as_str() {
            "step" | "s" => {
                if offset >= program.len() {
                    println!("End of program.");
                    continue;
                }
                let decoded = Decoder::decode(program[offset]).map_err(|error| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("invalid opcode at byte {offset}: {error}"),
                    )
                })?;
                let end = offset + decoded.length() as usize;
                if end > program.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        format!("truncated {:?} at byte {offset}", decoded.instruction),
                    )
                    .into());
                }
                println!(
                    "0x{offset:04X}: {}",
                    disassembler::disassemble(&program[offset..end])?.trim()
                );
                match execute_interactive_step(computer, program, offset)? {
                    Some(next) => offset = next,
                    None => {
                        offset = end;
                        println!("Program halted.");
                    }
                }
                println!("{}", cpu_state(computer));
            }
            "continue" | "c" => {
                let max_steps = match parts.next() {
                    Some(value) => parse_debug_number(value)
                        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?,
                    None => 100_000,
                };
                if max_steps == 0 {
                    println!("Step count must be greater than zero.");
                    continue;
                }
                let mut executed = 0;
                while executed < max_steps {
                    match execute_interactive_step(computer, program, offset)? {
                        Some(next) => {
                            offset = next;
                            executed += 1;
                        }
                        None => {
                            if offset < program.len() {
                                let decoded =
                                    Decoder::decode(program[offset]).map_err(|error| {
                                        io::Error::new(
                                            io::ErrorKind::InvalidData,
                                            format!("invalid opcode at byte {offset}: {error}"),
                                        )
                                    })?;
                                offset += decoded.length() as usize;
                                executed += 1;
                            }
                            break;
                        }
                    }
                }
                println!("Executed {executed} instruction(s).");
                println!("{}", cpu_state(computer));
            }
            "regs" | "registers" | "r" => println!("{}", cpu_state(computer)),
            "mem" | "memory" | "m" => {
                let Some(address) = parts.next() else {
                    println!("Usage: mem <address> [length]");
                    continue;
                };
                let start = parse_debug_number(address)
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
                let length = match parts.next() {
                    Some(value) => parse_debug_number(value)
                        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?,
                    None => 64,
                };
                match memory_dump(computer, start, length) {
                    Ok(dump) => print!("{dump}"),
                    Err(error) => println!("{error}"),
                }
            }
            "screen" => computer.update_screen()?,
            "help" | "h" | "?" => println!(
                "step (s)           Execute one instruction\n\
                 continue [count]   Run up to count instructions (default 100000)\n\
                 regs (r)           Show registers and decoded status flags\n\
                 mem <addr> [len]   Hex/ASCII dump RAM (default 64 bytes, max 256)\n\
                 screen             Render the RGB framebuffer\n\
                 quit (q)           Exit debugger"
            ),
            "quit" | "q" | "exit" => break,
            _ => println!("Unknown command '{command}'. Type 'help' for commands."),
        }
    }
    Ok(())
}

fn debug_file(path: &Path) -> Result<(), Box<dyn Error>> {
    if path.extension().and_then(|extension| extension.to_str()) != Some("bin") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "debugger input path must have a .bin extension",
        )
        .into());
    }
    let program = fs::read(path)?;
    let mut computer = Computer::new();
    debug_program(&mut computer, &program)
}

fn run_binary(computer: &mut Computer, path: &Path) -> Result<(), Box<dyn Error>> {
    run_binary_with_input(computer, path, false)
}

fn run_binary_with_input(
    computer: &mut Computer,
    path: &Path,
    interactive: bool,
) -> Result<(), Box<dyn Error>> {
    if path.extension().and_then(|extension| extension.to_str()) != Some("bin") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "program path must have a .bin extension",
        )
        .into());
    }

    let program = fs::read(path)?;
    execute_program_with_input(computer, &program, interactive)
}

struct TerminalInput {
    active: bool,
}

impl TerminalInput {
    fn enable() -> io::Result<Self> {
        enable_raw_mode()?;
        if let Err(error) = execute!(io::stdout(), EnableMouseCapture) {
            let _ = disable_raw_mode();
            return Err(error);
        }
        Ok(Self { active: true })
    }

    fn restore(&mut self) -> io::Result<()> {
        let mouse_result = execute!(io::stdout(), DisableMouseCapture);
        let raw_result = disable_raw_mode();
        self.active = false;
        mouse_result.and(raw_result)
    }
}

impl Drop for TerminalInput {
    fn drop(&mut self) {
        if self.active {
            let _ = self.restore();
        }
    }
}

fn run_interactive_binary(computer: &mut Computer, path: &Path) -> Result<(), Box<dyn Error>> {
    let mut terminal = TerminalInput::enable()?;
    let result = (|| {
        if path.extension().and_then(|extension| extension.to_str()) != Some("bin") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "program path must have a .bin extension",
            )
            .into());
        }
        let program = fs::read(path)?;
        execute_interactive_program(computer, &program)
    })();
    terminal.restore()?;
    result
}

fn assemble_file(source_path: &Path, output_path: &Path) -> Result<(), Box<dyn Error>> {
    if source_path
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("asm")
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "assembly source path must have an .asm extension",
        )
        .into());
    }
    if output_path
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("bin")
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "assembler output path must have a .bin extension",
        )
        .into());
    }

    let source = fs::read_to_string(source_path)?;
    let binary = assembler::assemble(&source)?;
    fs::write(output_path, binary)?;
    Ok(())
}

fn run_assembler(args: &[std::ffi::OsString]) -> Result<(), Box<dyn Error>> {
    if args.len() != 4 || args[2] != "-o" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: cpu assemble <source.asm> -o <output.bin>",
        )
        .into());
    }
    assemble_file(Path::new(&args[1]), Path::new(&args[3]))
}

fn disassemble_file(binary_path: &Path, output_path: Option<&Path>) -> Result<(), Box<dyn Error>> {
    if binary_path
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("bin")
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "disassembler input path must have a .bin extension",
        )
        .into());
    }
    if output_path.is_some_and(|path| {
        path.extension().and_then(|extension| extension.to_str()) != Some("asm")
    }) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "disassembler output path must have an .asm extension",
        )
        .into());
    }

    let binary = fs::read(binary_path)?;
    let assembly = disassembler::disassemble(&binary)?;
    if let Some(output_path) = output_path {
        fs::write(output_path, assembly)?;
    } else {
        use io::Write;
        io::stdout().lock().write_all(assembly.as_bytes())?;
    }
    Ok(())
}

fn run_disassembler(args: &[std::ffi::OsString]) -> Result<(), Box<dyn Error>> {
    match args {
        [_, input] => disassemble_file(Path::new(input), None),
        [_, input, output_flag, output] if output_flag == "-o" => {
            disassemble_file(Path::new(input), Some(Path::new(output)))
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: cpu disassemble <input.bin> [-o <output.asm>]",
        )
        .into()),
    }
}

fn run_debugger(args: &[std::ffi::OsString]) -> Result<(), Box<dyn Error>> {
    match args {
        [_, program] => debug_file(Path::new(program)),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: cpu debug <program.bin>",
        )
        .into()),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() == 1 && matches!(args[0].to_str(), Some("-h" | "--help")) {
        println!(
            "Usage:\n  cpu [program.bin [--interactive]]\n  cpu debug <program.bin>\n  cpu assemble <source.asm> -o <output.bin>\n  cpu disassemble <input.bin> [-o <output.asm>]\n\nWithout a program path, runs the built-in screen demo. Interactive mode captures terminal keyboard and mouse events into reserved RAM."
        );
        return Ok(());
    }
    if args.first().and_then(|arg| arg.to_str()) == Some("assemble") {
        return run_assembler(&args);
    }
    if args.first().and_then(|arg| arg.to_str()) == Some("disassemble") {
        return run_disassembler(&args);
    }
    if args.first().and_then(|arg| arg.to_str()) == Some("debug") {
        return run_debugger(&args);
    }

    let mut computer = Computer::new();
    let mut render_screen = true;
    match args.as_slice() {
        [path] => run_binary(&mut computer, Path::new(path))?,
        [path, flag] if flag == "--interactive" => {
            render_screen = false;
            run_interactive_binary(&mut computer, Path::new(path))?
        }
        [] => {
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
                "usage: cpu [program.bin [--interactive]] | cpu assemble <source.asm> -o <output.bin> | cpu disassemble <input.bin> [-o <output.asm>]",
            )
            .into());
        }
    }

    if render_screen {
        computer.update_screen()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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

    #[test]
    fn debugger_formats_registers_and_status_flags() {
        let mut computer = Computer::new();
        computer.cpu.pc.update(0x1234, true, false, false, true);
        computer.cpu.sr.update(0b1010_1100, true, true);

        let output = cpu_state(&computer);

        assert!(output.contains("PC    0x1234"));
        assert!(output.contains("SR    0xAC"));
        assert!(output.contains("Flags: ZR=1 NG=0 CY=1 OV=0 EQ=1 GT=1 LT=0"));
    }

    #[test]
    fn debugger_memory_dump_displays_hex_ascii_and_checks_bounds() {
        let mut computer = Computer::new();
        computer.ram.update(b'A', true, 0x0010, true);
        computer.ram.update(0x01, true, 0x0011, true);

        let dump = memory_dump(&computer, 0x10, 2).unwrap();
        assert!(dump.contains("0010: 41 01"));
        assert!(dump.contains("|A.|"));
        assert!(memory_dump(&computer, 0xFFFF, 2).is_err());
        assert!(memory_dump(&computer, 0, 257).is_err());
    }

    #[test]
    fn wasd_pixel_program_moves_and_respects_screen_bounds() {
        let program = assembler::assemble(include_str!("../wasd_board.asm")).unwrap();
        let mut computer = Computer::new();
        let mut offset = 0;

        for _ in 0..20 {
            offset = execute_interactive_step(&mut computer, &program, offset)
                .unwrap()
                .unwrap();
        }
        assert_eq!(computer.ram.output(cpu::computer::BOARD_X), 75);
        assert_eq!(computer.ram.output(cpu::computer::BOARD_Y), 20);

        computer.handle_input_event(Event::Key(KeyEvent::new(
            KeyCode::Char('w'),
            KeyModifiers::NONE,
        )));
        for _ in 0..200 {
            offset = execute_interactive_step(&mut computer, &program, offset)
                .unwrap()
                .unwrap();
            if computer.ram.output(cpu::computer::BOARD_Y) == 19
                && computer.ram.output(cpu::computer::KEY_EVENT_TYPE) == 0
            {
                break;
            }
        }
        assert_eq!(computer.ram.output(cpu::computer::BOARD_Y), 19);

        computer.handle_input_event(Event::Key(KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::NONE,
        )));
        for _ in 0..200 {
            offset = execute_interactive_step(&mut computer, &program, offset)
                .unwrap()
                .unwrap();
            if computer.ram.output(cpu::computer::BOARD_X) == 74
                && computer.ram.output(cpu::computer::KEY_EVENT_TYPE) == 0
            {
                break;
            }
        }
        assert_eq!(computer.ram.output(cpu::computer::BOARD_X), 74);

        for _ in 0..74 {
            computer.handle_input_event(Event::Key(KeyEvent::new(
                KeyCode::Char('a'),
                KeyModifiers::NONE,
            )));
            for _ in 0..200 {
                offset = execute_interactive_step(&mut computer, &program, offset)
                    .unwrap()
                    .unwrap();
                if computer.ram.output(cpu::computer::KEY_EVENT_TYPE) == 0 {
                    break;
                }
            }
        }
        assert_eq!(computer.ram.output(cpu::computer::BOARD_X), 0);
    }

    #[test]
    fn assembler_cli_writes_a_binary_program() {
        let directory =
            std::env::temp_dir().join(format!("cpu-assembler-test-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let source_path = directory.join("sample.asm");
        let output_path = directory.join("sample.bin");
        fs::write(&source_path, "LDI #0x14\nHLT\n").unwrap();

        assemble_file(&source_path, &output_path).unwrap();

        assert_eq!(fs::read(&output_path).unwrap(), [0x00, 0x14, 0xC6]);
        fs::remove_dir_all(directory).unwrap();
    }
}
