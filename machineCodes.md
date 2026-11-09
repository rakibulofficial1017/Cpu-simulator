# Machine Codes

Each instruction, including each register-specific form, has its own 8-bit
opcode. The opcode decoder accepts the 204 assigned values and rejects the 52
reserved values as invalid.

## Register encoding

Source registers, in encoding order:

`PC, ACC, TempA, TempB, PSR, SR, AR, DR`

Destination registers for `MOV` and `POP`:

`ACC, AR, TempA, TempB`

`SP` is internal to stack/control operations and is not directly encoded as a
source or destination. `MOV` omits redundant self-moves. Transfers between
8-bit and 16-bit registers zero-extend or truncate to the low byte,
respectively.

## Opcode allocation

| Code range | Instruction | Allocation |
|---|---|---|
| `00` | `LDI` | Load an immediate byte into `ACC` |
| `01` | `LDA` | Load a big-endian 16-bit immediate address into `AR` |
| `02` | `STA` | Store `ACC` to the address in `AR` |
| `03–1E` | `MOV source → destination` | One opcode per valid ordered source/destination pair; grouped by source, with destinations in the order above and self-moves omitted |
| `1F–5E` | `CMP left, right` | All ordered source-register pairs; `1F + left_index × 8 + right_index` |
| `5F–9E` | ALU operations | Eight source-register opcodes per operation, in source-register order |
| `9F–A8` | `INC`, `DEC`, `NOT`, `SHL`, `SHR`, `SAR`, `ROL`, `ROR`, `RCL`, `RCR` | One opcode per operation |
| `A9–B0` | `PUSH source` | One opcode per source register, in source-register order |
| `B1–B4` | `POP destination` | One opcode per destination register, in destination-register order |
| `B5–C3` | Jumps | `JMP`, `JZ`, `JNZ`, `JN`, `JNN`, `JC`, `JNC`, `JV`, `JNV`, `JE`, `JNE`, `JGT`, `JLT`, `JGE`, `JLE` |
| `C4` | `CALL` | Call using the existing `AR` convention and internal `SP` |
| `C5` | `RET` | Return |
| `C6` | `HLT` | Halt |
| `C7` | `NOP` | No operation |
| `C8` | `INT` | Interrupt with an immediate interrupt number |
| `C9` | `IRET` | Return from interrupt |
| `CA` | `EI` | Enable interrupts |
| `CB` | `DI` | Disable interrupts |
| `CC–FF` | Reserved (`NULL`) | Invalid opcodes |

ALU operation ranges and per-range source ordering:

| Code range | Operation |
|---|---|
| `5F–66` | `ADD` |
| `67–6E` | `ADC` |
| `6F–76` | `SUB` |
| `77–7E` | `SBC` |
| `7F–86` | `AND` |
| `87–8E` | `OR` |
| `8F–96` | `XOR` |
| `97–9E` | `TEST` |

Each ALU source is treated as an 8-bit value using the lower eight bits of the
register. `LDI` and `INT` are followed by one immediate byte. `LDA` is followed
by a two-byte, big-endian address. `STA`, jumps, and `CALL` use the existing
`AR` convention.

The opcode decoder identifies instructions and register operands. Exact
execution semantics for shifts/rotations, interrupt handling, and calls should
be specified before implementing their execution behavior.
