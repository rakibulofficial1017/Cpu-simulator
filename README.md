# CPU Simulator

A Rust implementation of an 8-bit CPU with 16-bit address and temporary
registers. This README documents the proposed ISA, its complete assigned
opcode table, and the current limits of single-opcode execution.

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

`CPU::execute_instruction(opcode: u8)` executes a one-byte instruction.
`CPU::execute_instruction_with_operands(opcode, operands)` executes an
instruction with its trailing operand bytes. For example:

```rust
cpu.execute_instruction_with_operands(0x00, &[0x14])?; // LDI 0x14
cpu.execute_instruction_with_operands(0x01, &[0x12, 0x34])?; // LDA 0x1234
```

These methods do not fetch instruction bytes or advance `PC`. `LDA` queues a
memory read at the immediate address; the computer services the request on its
next tick and places the byte in `DR`. The methods return an error for an
unassigned opcode, an incorrect number of operand bytes, or an instruction
whose execution behavior has not been implemented. `INT` accepts its operand
but is not yet executable; stack operations, calls/returns, interrupt return,
and shifts/rotations are also not implemented.

Execution behavior for the remaining instructions is still under development;
the opcode assignments describe the ISA allocation, not a guarantee that every
instruction is currently implemented.
