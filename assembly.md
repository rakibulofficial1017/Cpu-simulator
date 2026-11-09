# Assembly Language Reference

This document describes the supported assembly syntax and convenient
pseudo-instructions. The project includes a two-pass assembler for this
syntax, invoked through the simulator command line.

## Conventions

- Opcode and immediate values are written in hexadecimal with a `0x` prefix.
- `#value` means an immediate constant; it is not a register.
- A 16-bit address is encoded most-significant byte first.
- Register names are `PC`, `ACC`, `TempA`, `TempB`, `PSR`, `SR`, `AR`, and
  `DR`. `SP` is internal and cannot be named by an instruction.
- `MOV source -> destination` is written in that order. Valid destinations are
  `ACC`, `AR`, `TempA`, and `TempB`; redundant self-moves do not have opcodes.
- ALU instructions use `ACC` as the left operand and result. For example,
  `ADD TempA` means `ACC = ACC + low8(TempA)`.
- When both `CMP` operands are 16-bit registers (`PC`, `TempA`, `TempB`,
  `PSR`, `AR`), all 16 bits are compared. Otherwise both operands are compared
  as low-eight-bit values.
- `JMP` and conditional jumps use the address already in `AR`; they do not
  take an address operand.

## Native instruction syntax

Each row below describes an instruction encoded directly by one opcode, with
only the noted immediate bytes following it.

| Assembly syntax | Operation |
|---|---|
| `LDI #byte` | Load an immediate byte into `ACC`; encoded as opcode `00`, then the byte |
| `LDA #address` | Set `AR` to an immediate 16-bit address and request a memory read; encoded as opcode `01`, then high and low address bytes |
| `STA` | Store `ACC` at the address in `AR` (`02`) |
| `MOV source -> destination` | Copy between a valid source and destination (`03–1E`); 8-to-16-bit moves zero-extend, 16-to-8-bit moves keep the low byte |
| `CMP left, right` | Compare an ordered source-register pair and update status flags (`1F–5E`) |
| `ADD source` | Add `low8(source)` to `ACC` (`5F–66`) |
| `ADC source` | Add `low8(source)` and carry-in to `ACC` (`67–6E`) |
| `SUB source` | Subtract `low8(source)` from `ACC` (`6F–76`) |
| `SBC source` | Subtract `low8(source)` and borrow-in from `ACC` (`77–7E`) |
| `AND source` | Bitwise AND `ACC` with `low8(source)` (`7F–86`) |
| `OR source` | Bitwise OR `ACC` with `low8(source)` (`87–8E`) |
| `XOR source` | Bitwise XOR `ACC` with `low8(source)` (`8F–96`) |
| `TEST source` | AND `ACC` with `low8(source)` for flags only; `ACC` is unchanged (`97–9E`) |
| `INC` | Increment `ACC` (`9F`) |
| `DEC` | Decrement `ACC` (`A0`) |
| `NOT` | Bitwise complement of `ACC` (`A1`) |
| `SHL`, `SHR`, `SAR` | Shift `ACC` left, logical-right, or arithmetic-right (`A2–A4`) |
| `ROL`, `ROR` | Rotate `ACC` left or right (`A5–A6`) |
| `RCL`, `RCR` | Rotate `ACC` through carry left or right (`A7–A8`) |
| `PUSH source` | Push a source register (`A9–B0`) |
| `POP destination` | Pop into `ACC`, `AR`, `TempA`, or `TempB` (`B1–B4`) |
| `JMP` | Unconditionally set `PC = AR` (`B5`) |
| `JZ`, `JNZ` | Jump if `ZR` is set or clear (`B6–B7`) |
| `JN`, `JNN` | Jump if `NG` is set or clear (`B8–B9`) |
| `JC`, `JNC` | Jump if `CY` is set or clear (`BA–BB`) |
| `JV`, `JNV` | Jump if `OV` is set or clear (`BC–BD`) |
| `JE`, `JNE` | Jump if `EQ` is set or clear (`BE–BF`) |
| `JGT`, `JLT`, `JGE`, `JLE` | Jump according to the comparison flags (`C0–C3`) |
| `CALL` | Call the address in `AR` (`C4`) |
| `RET` | Return from a call (`C5`) |
| `HLT` | Halt (`C6`) |
| `NOP` | No operation (`C7`) |
| `INT #number` | Interrupt with an immediate number; opcode `C8`, then one byte |
| `IRET` | Return from an interrupt (`C9`) |
| `EI`, `DI` | Enable or disable interrupts (`CA–CB`) |

For `CMP`, the source-register order is `PC, ACC, TempA, TempB, PSR, SR, AR,
DR`. Its opcode is `0x1F + left_index * 8 + right_index`. For the ALU
operations, the eight opcodes in each operation's range follow the same
register order.

`ADC` adds when `CY = 1`. `SBC` treats `CY = 0` as a borrow-in. Arithmetic
updates `ZR` and `NG`; arithmetic also updates `CY` and `OV`. Logical
operations update `ZR` and `NG`. `CMP` updates comparison flags `EQ`, `GT`,
and `LT`, as well as arithmetic status flags.

## Supported pseudo-instructions

Pseudo-instructions are shorthand sequences, not additional opcodes. An
assembler expands them into native instructions.

### Loads, stores, and copies

| Pseudo-instruction | Expansion |
|---|---|
| `LOAD #byte` | `LDI #byte` |
| `LOAD_ADDRESS #address` | `LDA #address` |
| `STORE #byte, #address` | `LDI #byte`; `LDA #address`; `STA` |
| `COPY source -> destination` | `MOV source -> destination` |

`LDA #address` reads the byte at that address into `DR`; `STORE` writes the
immediate byte there after setting `AR`.

### Labels and data

Labels use a name followed by a colon. Labels identify byte offsets in the
assembled output. Semicolon starts a line comment. `.byte` emits one or more
literal bytes:

```asm
start:
    LDI #0x2A ; load 42
    JMP done
    .byte 0x00, 0xFF
done: HLT
```

For `JMP label`, conditional branches such as `JZ label`, and `CALL label`,
the assembler emits `LDA #label` followed by the corresponding control
instruction. A branch without a label, such as `JZ`, is the native
one-byte instruction and uses the address already in `AR`.

Values can be decimal (`42`), hexadecimal (`0x2A`), or binary (`0b00101010`).
Immediate operands must use the `#` prefix. Register names and instruction
mnemonics are case-insensitive; labels are case-sensitive.

The assembler currently supports native instructions, labels, `.byte`,
`LOAD`, `LOAD_ADDRESS`, `STORE`, and `COPY`. The destination-oriented
arithmetic forms described below are useful expansion patterns but are not
currently parsed as assembler pseudo-instructions. They must be written as
their native instruction sequences.

### Destination-oriented arithmetic patterns

The following syntax illustrates destination-oriented arithmetic. These
convenience forms are **not currently accepted by the assembler**; write the
corresponding native sequences described below.

```text
ADD source TO destination       ; destination = destination + low8(source)
ADC source TO destination       ; destination = destination + low8(source) + CY
SUB source FROM destination     ; destination = destination - low8(source)
SBC source FROM destination     ; destination = destination - low8(source) - borrow
AND source WITH destination     ; destination = destination & low8(source)
OR source WITH destination      ; destination = destination | low8(source)
XOR source WITH destination     ; destination = destination ^ low8(source)
```

For `destination = ACC`, each expands to the corresponding native operation
with `source`, for example `ADD TempA TO ACC` becomes `ADD TempA`.

For a different valid destination, preserve the destination in `ACC`, perform
the operation, then copy the result back. For example:

```text
ADD TempA TO TempB
```

expands to:

```text
MOV TempB -> ACC
ADD TempA
MOV ACC -> TempB
```

When `source` is `ACC` and `destination` is not `ACC`, first preserve the
original `ACC` in a temporary register not used as `destination`, then load
`destination` into `ACC`, apply the operation using that temporary source,
and copy the result back. This avoids overwriting the source before it is
read.

### Immediate arithmetic and logic patterns

There are no immediate ALU opcodes. A future assembler could expand
`OP #byte TO destination` by preserving the destination and immediate in
temporary registers, then applying the native ALU operation. One valid
expansion for a destination other than `TempA` or `TempB` is:

```text
MOV destination -> TempA
LDI #byte
MOV ACC -> TempB
MOV TempA -> ACC
OP TempB
MOV ACC -> destination
```

For `destination = TempA`, preserve the old value in `TempB`, use `TempA` for
the immediate, restore the old destination into `ACC`, apply the operation,
then copy the result back to `TempA`. For `destination = TempB`, swap the
temporary roles. If `destination = ACC`, save the original `ACC` in `TempA`,
load the immediate, copy it to `TempB`, restore `ACC`, then apply the
operation using `TempB`.

Replace `OP` with `ADD`, `ADC`, `SUB`, `SBC`, `AND`, `OR`, or `XOR`. For
example, `SUB #3 FROM TempA` means `TempA = TempA - 3`; it does **not** mean
`3 - TempA`.

`TEST #byte` can be implemented with this sequence, which preserves `ACC` and
leaves it unchanged while updating flags:

```text
MOV ACC -> TempA
LDI #byte
MOV ACC -> TempB
MOV TempA -> ACC
TEST TempB
```

### Increment, decrement, and compare

For a destination other than `ACC`, increment/decrement can be expanded as:

```text
MOV destination -> ACC
INC                             ; or DEC
MOV ACC -> destination
```

`TEST source` updates result flags without storing the AND result.
`CMP left, right` is already register-to-register and needs no expansion.

## Binary file format and running programs

A `.bin` file is a sequence of opcode bytes and, for `LDI`, `LDA`, or `INT`,
the immediate bytes shown above. It contains no text, comments, separators, or
mnemonics. For example, this program loads `0x14` into `ACC` and halts:

```text
00 14 C6
```

On a Unix-like system, create and run it with:

```sh
printf '\x00\x14\xc6' > hello.bin
cargo run --manifest-path cpu/Cargo.toml -- hello.bin
```

Assemble a source file from the repository root with:

```sh
cargo run --manifest-path cpu/Cargo.toml -- assemble program.asm -o program.bin
```

Then run the resulting binary:

```sh
cargo run --manifest-path cpu/Cargo.toml -- program.bin
```

Run the included screen-writing sample from the repository root with:

```sh
cargo run --manifest-path cpu/Cargo.toml -- cpu/sampleProgram.bin
```

The sample uses repeated `LDI`, `LDA`, and `STA` instructions to write yellow
`HELLO WORLD` glyph pixels into the RGB framebuffer, then executes `HLT`.

With no file argument, the simulator draws its built-in `HELLO WORLD` demo:

```sh
cargo run --manifest-path cpu/Cargo.toml
```

The command-line runner executes bytes sequentially from the file, services
`LDA` and `STA` memory requests between instructions, and stops at `HLT` or the
end of the file. It does not fetch instructions from RAM, increment `PC`, or
use jump targets to change the file position. The CPU can update its `PC` when
a jump is executed, but this does not implement control flow in the file
runner.

The decoder recognizes all assigned opcodes, but not every instruction can
currently execute. `PUSH`, `POP`, `CALL`, `RET`, `INT`, `IRET`, and the shift
and rotate instructions return an unsupported-instruction error. Stack
layout, interrupt behavior, and shift/rotate flag details still need a
finalized specification.
