# CPU Simulator

A Rust-based simulator for a small 8-bit CPU with 16-bit address and temporary
registers. It includes a command-line runner, an assembler and disassembler, a
150 × 40 RGB terminal framebuffer, and optional terminal keyboard and mouse
input.

This guide covers building and running the simulator, translating assembly
source to a program binary, and using the included examples. The
[assembly reference](./assembly.md) describes the assembly language and the
[machine-code reference](./machineCodes.md) lists the opcode allocation.

## Contents

- [Requirements](#requirements)
- [Build a standalone simulator executable](#build-a-standalone-simulator-executable)
- [Command-line usage](#command-line-usage)
- [Assemble and run a program](#assemble-and-run-a-program)
- [Included examples](#included-examples)
- [Disassemble a binary](#disassemble-a-binary)
- [Debug a program](#debug-a-program)
- [Run an interactive program](#run-an-interactive-program)
- [CPU and memory overview](#cpu-and-memory-overview)
- [Instruction support and simulator limitations](#instruction-support-and-simulator-limitations)
- [Test the project](#test-the-project)

## Requirements

- Rust and Cargo (edition 2024 compatible) to build the simulator.
- A terminal supporting ANSI cursor control and 24-bit true-color background
  sequences to display the framebuffer.
- A terminal for `--interactive` mode. Keyboard and mouse capture are provided
  through Crossterm; mouse input depends on terminal support.

The framebuffer is 150 columns by 40 rows. Use a terminal at least that large
to see the complete image without automatic wrapping or clipping.

## Build a standalone simulator executable

From the repository root, build the optimized executable:

```sh
cargo build --release --manifest-path cpu/Cargo.toml
```

The resulting executable is:

- Linux and macOS: `cpu/target/release/cpu`
- Windows: `cpu/target/release/cpu.exe`

You can copy this executable to another machine with a compatible operating
system and CPU architecture and run it without Cargo or the Rust compiler.
Executables are platform-specific; build separately for each target platform.

For development, Cargo can build and run the simulator directly:

```sh
cargo run --manifest-path cpu/Cargo.toml
```

The first run compiles the program and may download its Rust dependencies.

## Command-line usage

The commands below assume you are in the repository root. Replace
`cargo run --manifest-path cpu/Cargo.toml --` with the path to the standalone
executable when using a release build.

```text
cpu [program.bin [--interactive]]
cpu debug <program.bin>
cpu assemble <source.asm> -o <output.bin>
cpu disassemble <input.bin> [-o <output.asm>]
```

Examples:

```sh
# Show the built-in Hello World framebuffer demo
cargo run --manifest-path cpu/Cargo.toml --

# Run a machine-code program
cargo run --manifest-path cpu/Cargo.toml -- cpu/sampleProgram.bin

# Assemble source, then run the resulting program
cargo run --manifest-path cpu/Cargo.toml -- assemble program.asm -o program.bin
cargo run --manifest-path cpu/Cargo.toml -- program.bin

# Disassemble a binary to the terminal or save its assembly text
cargo run --manifest-path cpu/Cargo.toml -- disassemble program.bin
cargo run --manifest-path cpu/Cargo.toml -- disassemble program.bin -o program-disassembled.asm

# Step through a binary and inspect CPU state or RAM
cargo run --manifest-path cpu/Cargo.toml -- debug program.bin
```

To run the release executable instead:

```sh
./cpu/target/release/cpu
./cpu/target/release/cpu assemble program.asm -o program.bin
./cpu/target/release/cpu program.bin
./cpu/target/release/cpu debug program.bin
```

On Windows, run `cpu.exe` (or `.\cpu.exe` in PowerShell) with the same
arguments. Use `cpu --help` for the built-in usage summary.

## Assemble and run a program

Assembly source is plain text saved with an `.asm` extension. Assemble it to a
raw `.bin` file with the `assemble` subcommand:

```sh
cargo run --manifest-path cpu/Cargo.toml -- assemble path/to/program.asm -o path/to/program.bin
```

The output file contains only machine-code bytes; it does not contain assembly
comments or source formatting. Run that file with:

```sh
cargo run --manifest-path cpu/Cargo.toml -- path/to/program.bin
```

The assembler uses the instruction set's native syntax. For example, this
program loads decimal 20 (`0x14`) into the accumulator and stops:

```asm
    LDI #0x14
    HLT
```

Save it as `hello.asm`, then translate and run it:

```sh
cargo run --manifest-path cpu/Cargo.toml -- assemble hello.asm -o hello.bin
cargo run --manifest-path cpu/Cargo.toml -- hello.bin
```

The runner starts with fresh, zeroed CPU registers and RAM. It reads program
bytes from the beginning of the file, decodes each instruction and its
immediate operands, and services memory reads and writes before continuing.
Ordinary mode runs instructions sequentially until `HLT` or the end of the
file. See [Instruction support and simulator limitations](#instruction-support-and-simulator-limitations)
for the differences in interactive mode.

### Assembly language basics

- Mnemonics and register names are case-insensitive.
- Decimal, hexadecimal (`0x`), and binary (`0b`) numbers are accepted.
- Prefix immediate operands with `#`, such as `LDI #0x14` or `LDA #0x1000`.
- Use `;` to start a comment.
- Define labels with a trailing colon, then branch to them by name.
- Use `.byte value, value` to emit literal bytes.
- Addresses for `LDA` are encoded big-endian.
- `JMP label`, conditional branches to labels, and `CALL label` expand to an
  address load followed by the control instruction.

Example:

```asm
start:
    LDI #0x14
    LDA #0x1000
    STA
    JMP start
```

The assembler also supports the `LOAD`, `LOAD_ADDRESS`, `STORE`, and `COPY`
pseudo-instructions. These expand into native instructions; they do not have
their own opcodes. Destination-oriented arithmetic forms described in the
[assembly reference](./assembly.md) are examples only and are not currently
accepted by the assembler.

See [assembly.md](./assembly.md) for all supported syntax, examples, label
rules, and pseudo-instruction expansions.

## Included examples

### Hello World framebuffer binary

[`cpu/sampleProgram.bin`](./cpu/sampleProgram.bin) is a raw binary program that
writes yellow `HELLO WORLD` glyph pixels to screen RAM. Run it from the
repository root:

```sh
cargo run --manifest-path cpu/Cargo.toml -- cpu/sampleProgram.bin
```

The equivalent built-in demo can be rendered without a program file:

```sh
cargo run --manifest-path cpu/Cargo.toml
```

### WASD moving pixel

[`cpu/wasd_board.asm`](./cpu/wasd_board.asm) is an assembly example that moves
a yellow pixel on the framebuffer. Assemble and run it in interactive mode:

```sh
cargo run --manifest-path cpu/Cargo.toml -- assemble cpu/wasd_board.asm -o cpu/wasd_board.bin
cargo run --manifest-path cpu/Cargo.toml -- cpu/wasd_board.bin --interactive
```

Use **W**, **A**, **S**, and **D** to move the pixel; press **Escape** to exit.
The demo is constrained to the framebuffer boundaries. In interactive mode the
runner follows taken jump instructions, allowing the assembly program to poll
for input repeatedly. It redraws the framebuffer when the pixel moves.

## Disassemble a binary

Disassemble a `.bin` file to standard output:

```sh
cargo run --manifest-path cpu/Cargo.toml -- disassemble program.bin
```

Or save the generated assembly text:

```sh
cargo run --manifest-path cpu/Cargo.toml -- disassemble program.bin -o program-disassembled.asm
```

The disassembler prints native instruction syntax, `.byte` directives for
reserved opcode values, and byte offsets as comments. Its output can be
reassembled to reproduce the input bytes. It does not reconstruct symbolic
labels; branch instructions retain their native `AR`-based form. A truncated
immediate instruction is reported with its byte offset.

## Debug a program

Start the command-line debugger with a machine-code file:

```sh
cargo run --manifest-path cpu/Cargo.toml -- debug program.bin
```

The debugger begins at byte offset zero and accepts commands at its prompt:

| Command | Action |
|---|---|
| `step` or `s` | Decode and execute one instruction, then show registers and flags |
| `continue [count]` or `c` | Execute up to `count` instructions; defaults to 100,000 per command |
| `regs` or `r` | Show registers in hexadecimal and decimal, and decode status flags |
| `mem <address> [length]` or `m` | Display a hex/ASCII RAM dump; default length is 64 bytes, maximum 256 |
| `screen` | Render the current RGB framebuffer |
| `help` | Show debugger commands |
| `quit` or `q` | Exit the debugger |

Memory addresses accept decimal or `0x`-prefixed hexadecimal. The debugger
follows taken jumps in the loaded binary and executes memory operations against
the simulator's RAM. `continue` has a per-command instruction limit so a
program loop returns control to the prompt.

## Run an interactive program

Pass `--interactive` after the program path:

```sh
cargo run --manifest-path cpu/Cargo.toml -- program.bin --interactive
```

Interactive mode enables terminal raw input and mouse capture, polls for
events before each instruction, and writes the latest event values into the
system-reserved RAM area. Pressing Escape exits the interactive runner. This
mode requires a terminal and works best when the program contains a loop.

### Keyboard and mouse RAM map

| Address | Meaning |
|---:|---|
| `0` | Latest key event: `1` pressed, `2` released, `3` repeat |
| `1` | Key code |
| `2` | Key modifiers bitmask |
| `3` | Latest mouse event: `1` pressed, `2` released, `3` moved/dragged, `4` scroll up, `5` scroll down |
| `4` | Held mouse buttons: left `1`, right `2`, middle `4` |
| `5–6` | Mouse X coordinate, low byte then high byte |
| `7–8` | Mouse Y coordinate, low byte then high byte |
| `9` | Used by the WASD demo for pixel X |
| `10` | Used by the WASD demo for pixel Y |

Coordinates are zero-based. ASCII key codes use their ASCII values. Special
key codes are Backspace `0x80`, Enter `0x81`, Left `0x82`, Right `0x83`, Up
`0x84`, Down `0x85`, Home `0x86`, End `0x87`, Page Up `0x88`, Page Down
`0x89`, Tab `0x8A`, BackTab `0x8B`, Delete `0x8C`, Insert `0x8D`, Escape
`0x8E`, and F1–F24 `0x90–0xA7`. Modifier bits 0–5 indicate Shift, Control,
Alt, Super, Hyper, and Meta respectively. Each mailbox keeps its latest event
until a new event arrives; programs may clear the key event type after reading
the key.

## CPU and memory overview

### Registers

| Register | Width | Purpose |
|---|---:|---|
| `PC` | 16 bits | Program counter |
| `ACC` | 8 bits | Accumulator and default ALU result |
| `TempA`, `TempB` | 16 bits each | Temporary registers |
| `PSR` | 16 bits | Program start register |
| `SR` | 8 bits | Status flags |
| `AR` | 16 bits | Address register |
| `DR` | 8 bits | Data register |
| `SP` | 16 bits | Internal stack pointer |
| `IR` | 8 bits | Instruction register |

`SP` is internal and cannot be named as an encoded source or destination.

### RAM map

The simulator has 64 KiB of RAM:

| Address range | Size | Purpose |
|---|---:|---|
| `0–99` | 100 bytes | System-reserved memory (including input mailbox) |
| `100–18,099` | 18,000 bytes | 150 × 40 RGB framebuffer |
| `18,100–65,535` | 47,436 bytes | General-purpose memory and stack area |

Each screen pixel uses three consecutive bytes in red, green, blue order.
Pixels are stored row-major, starting at address `100`. Programs can draw to
the screen by storing RGB values into this range. The runner displays the
framebuffer after ordinary program execution; interactive pixel programs
refresh it while running.

### Instruction set

The opcode allocation has 204 assigned values and 52 reserved values
(`0xCC–0xFF`). Each register-specific variant has its own one-byte opcode.
`LDI` takes one immediate byte, `LDA` takes a two-byte big-endian address, and
`INT` takes one interrupt-number byte. `STA`, jumps, and `CALL` use the address
in `AR`.

The ISA includes:

- Loads, stores, moves, and all ordered `CMP` source pairs.
- `ADD`, `ADC`, `SUB`, `SBC`, `AND`, `OR`, `XOR`, and `TEST` variants for each
  source register.
- Unary arithmetic/logic, shifts and rotations, and stack operations.
- Conditional and unconditional jumps, call/return, halt/no-op, and interrupt
  control instructions.

`CMP` compares all 16 bits when both sources are 16-bit registers; otherwise
it compares the low eight bits of both. The ALU uses the lower eight bits of
its source. `ADC` adds when carry is set; `SBC` treats clear carry as a
borrow-in. For exact opcode values and flag definitions, see
[machineCodes.md](./machineCodes.md).

## Instruction support and simulator limitations

The ordinary binary runner is a sequential file runner, not a complete
fetch/decode loop. It does not fetch instructions from RAM, and branch
instructions do not change the file position. It stops at `HLT` or end of
file. The instruction-level CPU API also executes one supplied opcode at a
time and does not fetch bytes or advance `PC` automatically.

Interactive mode is different: it uses the CPU program counter to follow
taken jumps within the loaded binary, allowing looping keyboard-driven
programs. It still fetches instruction bytes from the loaded file, not RAM.
Calls and returns do not provide a working subroutine stack.

The simulator currently executes `LDI`, `LDA`, `STA`, valid `MOV` forms,
`CMP`, arithmetic/logical operations, `INC`, `DEC`, `NOT`, jumps, `NOP`,
`HLT`, `EI`, and `DI`. `LDA` sets `AR`, requests a read, and places the result
in `DR`; `STA` writes `ACC` to the address in `AR`.

The decoder recognizes assigned opcodes for `PUSH`, `POP`, `CALL`, `RET`,
`INT`, `IRET`, and shifts/rotates, but these instructions currently return an
unsupported-instruction error. Their execution semantics are not yet
implemented. The interrupt controller itself is tested for masking, priority,
acknowledgement, and CPU interrupt-entry scheduling; this does not mean that
interrupt handlers or the `INT`/`IRET` instructions are executable yet.
Reserved opcodes are rejected. The runner reports invalid opcodes and
truncated operands with their byte offsets.

The screen renderer uses ANSI terminal output. A 150-column display may wrap
or clip on narrower terminals; increase the terminal width for accurate
pixel positions.

## Test the project

Run the unit tests from the repository root:

```sh
cargo test --manifest-path cpu/Cargo.toml
```

To check the release build:

```sh
cargo build --release --manifest-path cpu/Cargo.toml
```
