# CPU Simulator

A Rust implementation of an 8-bit CPU with 16-bit address and temporary
registers. This README covers the ISA, RAM and screen layout, the simulator
runner, and the included raw-binary sample program.

## Quick start

From the repository root, run the built-in Hello World screen demo:

```sh
cargo run --manifest-path cpu/Cargo.toml
```

Run the included raw machine-code sample instead:

```sh
cargo run --manifest-path cpu/Cargo.toml -- cpu/sampleProgram.bin
```

You can also run commands from the `cpu/` directory:

```sh
cargo run
cargo run -- sampleProgram.bin
```

The simulator requires a terminal that supports ANSI cursor positioning and
24-bit true-color background escape sequences. The screen renderer draws a
150-pixel-wide framebuffer, so use a terminal window wide enough to show it.

To run the unit tests:

```sh
cargo test --manifest-path cpu/Cargo.toml
```

## Registers

| Register | Width | Description |
|---|---:|---|
| `PC` | 16 bits | Program counter |
| `ACC` | 8 bits | Accumulator and default ALU result register |
| `TempA` | 16 bits | General-purpose temporary register |
| `TempB` | 16 bits | General-purpose temporary register |
| `PSR` | 16 bits | Program start register |
| `SR` | 8 bits | Status flags |
| `AR` | 16 bits | Address register |
| `DR` | 8 bits | Data register |
| `SP` | 16 bits | Internal stack pointer |
| `IR` | 8 bits | Instruction register |

`SP` is internal to stack/control operations. It is not encoded as a source or
destination operand.

## RAM map and screen

The 64 KiB RAM is divided as follows:

| Address range | Size | Purpose |
|---|---:|---|
| `0–99` | 100 bytes | System-reserved |
| `100–18,099` | 18,000 bytes | 150 × 40 RGB screen framebuffer |
| `18,100–65,535` | 47,436 bytes | General-purpose memory and stack |

Screen pixels are stored in row-major order. Each pixel occupies three
consecutive bytes: red, green, then blue. The first pixel starts at address
`100`, and the final pixel occupies addresses `18,097–18,099`.

Call `Computer::update_screen()` to redraw the framebuffer in the terminal.
It moves the cursor to the home position and prints one ANSI true-color
background space for each pixel. The method returns an I/O error if writing to
stdout fails.

The `SR` flag bits are:

| Bit | Name | Meaning |
|---:|---|---|
| 7 | `ZR` | Zero |
| 6 | `NG` | Negative |
| 5 | `CY` | Carry / no-borrow for subtraction |
| 4 | `OV` | Signed overflow |
| 3 | `EQ` | Equal comparison |
| 2 | `GT` | Greater-than comparison |
| 1 | `LT` | Less-than comparison |
| 0 | — | Reserved |

## Instruction encoding

Each instruction form has its own one-byte opcode. Register-specific
instructions encode the register directly in the opcode; they do not have a
separate register operand byte.

Registers appear in this encoding order:

`PC, ACC, TempA, TempB, PSR, SR, AR, DR`

`MOV` and `POP` destinations appear in this order:

`ACC, AR, TempA, TempB`

Immediate operands follow their opcode:

| Instruction | Following bytes |
|---|---|
| `LDI` | One byte |
| `LDA` | Two-byte address, most-significant byte first |
| `INT` | One interrupt-number byte |

All other assigned instructions are one byte. `STA`, jumps, and `CALL` use the
address held in `AR`.

## Opcode table

Opcode values below are hexadecimal.

### Load, store, and move

| Opcode | Instruction | Description |
|---:|---|---|
| `00` | `LDI byte` | Load an immediate byte into `ACC` |
| `01` | `LDA address` | Load a big-endian immediate address into `AR` |
| `02` | `STA` | Store `ACC` at the address in `AR` |

`MOV` has one opcode per valid source/destination pair. Redundant self-moves
are omitted.

| Opcode | Instruction | Opcode | Instruction |
|---:|---|---:|---|
| `03` | `MOV PC -> ACC` | `04` | `MOV PC -> AR` |
| `05` | `MOV PC -> TempA` | `06` | `MOV PC -> TempB` |
| `07` | `MOV ACC -> AR` | `08` | `MOV ACC -> TempA` |
| `09` | `MOV ACC -> TempB` | `0A` | `MOV TempA -> ACC` |
| `0B` | `MOV TempA -> AR` | `0C` | `MOV TempA -> TempB` |
| `0D` | `MOV TempB -> ACC` | `0E` | `MOV TempB -> AR` |
| `0F` | `MOV TempB -> TempA` | `10` | `MOV PSR -> ACC` |
| `11` | `MOV PSR -> AR` | `12` | `MOV PSR -> TempA` |
| `13` | `MOV PSR -> TempB` | `14` | `MOV SR -> ACC` |
| `15` | `MOV SR -> AR` | `16` | `MOV SR -> TempA` |
| `17` | `MOV SR -> TempB` | `18` | `MOV AR -> ACC` |
| `19` | `MOV AR -> TempA` | `1A` | `MOV AR -> TempB` |
| `1B` | `MOV DR -> ACC` | `1C` | `MOV DR -> AR` |
| `1D` | `MOV DR -> TempA` | `1E` | `MOV DR -> TempB` |

A transfer from an 8-bit register to a 16-bit register is zero-extended. A
transfer from a 16-bit register to an 8-bit register keeps the low byte.

### Compare

`CMP left, right` compares the low eight bits of both source registers and
updates the comparison status flags. The opcode is
`0x1F + left_index × 8 + right_index`, where indices use the source-register
order above.

| Left \ Right | PC | ACC | TempA | TempB | PSR | SR | AR | DR |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| **PC** | `1F` | `20` | `21` | `22` | `23` | `24` | `25` | `26` |
| **ACC** | `27` | `28` | `29` | `2A` | `2B` | `2C` | `2D` | `2E` |
| **TempA** | `2F` | `30` | `31` | `32` | `33` | `34` | `35` | `36` |
| **TempB** | `37` | `38` | `39` | `3A` | `3B` | `3C` | `3D` | `3E` |
| **PSR** | `3F` | `40` | `41` | `42` | `43` | `44` | `45` | `46` |
| **SR** | `47` | `48` | `49` | `4A` | `4B` | `4C` | `4D` | `4E` |
| **AR** | `4F` | `50` | `51` | `52` | `53` | `54` | `55` | `56` |
| **DR** | `57` | `58` | `59` | `5A` | `5B` | `5C` | `5D` | `5E` |

### Arithmetic and logical operations

Each operation has eight opcode variants, one per source register. The source
order in every row is `PC, ACC, TempA, TempB, PSR, SR, AR, DR`. Each operation
uses `ACC` as its left input and result destination, except `TEST`, which
updates flags without changing `ACC`.

| Operation | PC | ACC | TempA | TempB | PSR | SR | AR | DR |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `ADD` | `5F` | `60` | `61` | `62` | `63` | `64` | `65` | `66` |
| `ADC` | `67` | `68` | `69` | `6A` | `6B` | `6C` | `6D` | `6E` |
| `SUB` | `6F` | `70` | `71` | `72` | `73` | `74` | `75` | `76` |
| `SBC` | `77` | `78` | `79` | `7A` | `7B` | `7C` | `7D` | `7E` |
| `AND` | `7F` | `80` | `81` | `82` | `83` | `84` | `85` | `86` |
| `OR` | `87` | `88` | `89` | `8A` | `8B` | `8C` | `8D` | `8E` |
| `XOR` | `8F` | `90` | `91` | `92` | `93` | `94` | `95` | `96` |
| `TEST` | `97` | `98` | `99` | `9A` | `9B` | `9C` | `9D` | `9E` |

The ALU uses the lower eight bits of a source register. `ADC` adds the current
carry flag; `SBC` subtracts with borrow, where a clear carry flag represents
borrow. Arithmetic operations update zero, negative, carry, and overflow flags
as applicable. Logical operations update zero and negative; `TEST` does not
store its result.

### Unary operations

These operations act on `ACC`.

| Opcode | Instruction | Description |
|---:|---|---|
| `9F` | `INC` | Increment `ACC` |
| `A0` | `DEC` | Decrement `ACC` |
| `A1` | `NOT` | Bitwise complement of `ACC` |
| `A2` | `SHL` | Shift `ACC` left |
| `A3` | `SHR` | Shift `ACC` right |
| `A4` | `SAR` | Arithmetic shift `ACC` right |
| `A5` | `ROL` | Rotate `ACC` left |
| `A6` | `ROR` | Rotate `ACC` right |
| `A7` | `RCL` | Rotate `ACC` left through carry |
| `A8` | `RCR` | Rotate `ACC` right through carry |

Exact shift/rotation carry and sign behavior has not yet been finalized.

### Stack operations

The register variants are encoded in source/destination order. `SP` is
implicit.

| Opcode | Instruction | Opcode | Instruction |
|---:|---|---:|---|
| `A9` | `PUSH PC` | `AA` | `PUSH ACC` |
| `AB` | `PUSH TempA` | `AC` | `PUSH TempB` |
| `AD` | `PUSH PSR` | `AE` | `PUSH SR` |
| `AF` | `PUSH AR` | `B0` | `PUSH DR` |
| `B1` | `POP ACC` | `B2` | `POP AR` |
| `B3` | `POP TempA` | `B4` | `POP TempB` |

Stack width, byte order, and pointer update details remain to be specified.

### Jumps

Every jump tests the indicated `SR` flag and, if its condition is met, loads
`PC` from `AR`.

| Opcode | Instruction | Condition |
|---:|---|---|
| `B5` | `JMP` | Always |
| `B6` | `JZ` | `ZR = 1` |
| `B7` | `JNZ` | `ZR = 0` |
| `B8` | `JN` | `NG = 1` |
| `B9` | `JNN` | `NG = 0` |
| `BA` | `JC` | `CY = 1` |
| `BB` | `JNC` | `CY = 0` |
| `BC` | `JV` | `OV = 1` |
| `BD` | `JNV` | `OV = 0` |
| `BE` | `JE` | `EQ = 1` |
| `BF` | `JNE` | `EQ = 0` |
| `C0` | `JGT` | `GT = 1` |
| `C1` | `JLT` | `LT = 1` |
| `C2` | `JGE` | `LT = 0` |
| `C3` | `JLE` | `GT = 0` |

### Control instructions

| Opcode | Instruction | Description |
|---:|---|---|
| `C4` | `CALL` | Call the address in `AR`; return-address stack behavior is not finalized |
| `C5` | `RET` | Return from a call |
| `C6` | `HLT` | Halt execution |
| `C7` | `NOP` | No operation |
| `C8` | `INT number` | Request the numbered interrupt; followed by one byte |
| `C9` | `IRET` | Return from an interrupt |
| `CA` | `EI` | Enable interrupts |
| `CB` | `DI` | Disable interrupts |

Interrupt entry/return and call/return execution details remain to be finalized.

### Reserved opcodes

`CC` through `FF` are reserved (`NULL`) and are rejected by the opcode
decoder. There are 204 assigned opcode values and 52 reserved values.

## Executing instructions in the simulator

The command-line runner accepts zero or one argument:

```text
cpu [path/to/program.bin]
```

- With no argument, it draws the built-in `HELLO WORLD` bitmap into screen RAM.
- With a `.bin` path, it reads the file as raw bytes and executes the program.
- With more than one argument, a non-`.bin` path, or a malformed program, it
  reports an error and exits.

The runner starts with fresh, zeroed CPU registers and RAM. It reads the binary
from byte zero, decodes each opcode, reads the number of operand bytes specified
by that instruction, and executes instructions sequentially. Memory requests
created by `LDA` or `STA` are serviced before the next instruction. Execution
stops at `HLT` or at the end of the file. The screen is rendered after the
program finishes.

This is currently a sequential file runner, not a complete fetch/decode CPU
loop: it does not fetch instructions from RAM or advance `PC`. Jump opcodes can
change the CPU's `PC`, but they do not change the runner's file position.

### Writing a binary

A `.bin` file is a sequence of raw opcode and operand bytes. Opcode values and
operand byte order are described in the tables above. For example:

```text
00 14       ; LDI 0x14       ACC = 20
01 10 00    ; LDA 0x1000     AR = 0x1000; read RAM[0x1000] into DR
02          ; STA            write ACC to RAM[AR]
C6          ; HLT
```

The lines above are assembly-style notation for the bytes; comments and spaces
are not part of a raw binary file. For a minimal runnable example on Unix-like
systems, create a binary containing `LDI 0x14; HLT` with:

```sh
printf '\x00\x14\xc6' > hello.bin
cargo run --manifest-path cpu/Cargo.toml -- hello.bin
```

There is no assembler yet. The public CPU methods
`execute_instruction(opcode)` and
`execute_instruction_with_operands(opcode, operands)` are also available to
Rust callers. They execute one instruction at a time; neither fetches bytes
from RAM nor advances `PC`.

The runner reports an invalid opcode with its byte offset, detects a truncated
operand at the end of a file, and reports unsupported execution with the
instruction's byte offset. The decoder recognizes every assigned opcode, but
not every recognized instruction has execution semantics implemented.

Currently executable instructions are `LDI`, `LDA`, `STA`, valid `MOV` forms,
`CMP`, the arithmetic/logical operations, `INC`, `DEC`, `NOT`, all listed
jumps, `NOP`, `HLT`, `EI`, and `DI`. `LDA` sets `AR`, requests a read from that
address, and places the returned byte in `DR`. `STA` writes `ACC` to the
address in `AR`.

The following currently return an unsupported-instruction error:
`PUSH`, `POP`, `CALL`, `RET`, `INT`, `IRET`, and `SHL`, `SHR`, `SAR`, `ROL`,
`ROR`, `RCL`, `RCR`. These instructions have assigned opcodes, but their
execution details are not fully implemented.

### Sample binary

[`cpu/sampleProgram.bin`](./cpu/sampleProgram.bin) is a raw machine-code
program that draws yellow `HELLO WORLD` pixel glyphs into the RGB framebuffer
using repeated `LDI`, `LDA`, and `STA` instructions, then halts. Each lit pixel
is written as three RGB channel values: for each channel, the program loads the
color into `ACC`, selects the channel's screen-RAM address, then stores `ACC`
there. Unlit pixels remain black. Run it from the repository root with:

```sh
cargo run --manifest-path cpu/Cargo.toml -- cpu/sampleProgram.bin
```

The sample's output is the same yellow bitmap as the no-argument demo, but the
pixels are produced by executing machine-code instructions that write to the
screen's RAM address range.
