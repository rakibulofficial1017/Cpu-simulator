# Registers
| Registers available | Description | Bits |
|---------------------|-------------|------|
| PC | Contains the address of the current program | 16 |
| ACC | Contains the result of ALU operations | 8 |
| TempA | Temporary register | 16 | 
| TempB | Temporary register | 16 |
| PSR | Program start register | 16 |
| SR | The register containing statuses of a comparison in the format `[ZR][NG][LE][LT][GE][GT][NE][EQ]`| 8 |
| AR | Address register | 16 |
| DR | The data in the RAM in address stored in AR | 8 |

# Operations available:
- ## Load (2 instructions)
    - `Load address [address]`
        - This puts address into `AR` which automatically updates `DR`
    - `Load value [value]`
        - This loads the value into `ACC`
- ## Move (48 instructions)
    - `Move [reg1] to [reg2]`
        - reg1 can be register
        - reg2 must be of: ACC, TempA, TempB, AR
    - `Move [reg] to [address]`
        - reg can be anything
- ## Jump (27 instructions)
    ### Absolute Jumps
    - `Jump [address]` # value of AR
        - sets PC to address
    - `Jump rel [value]` # value of AR
        - set PC + value
    - `Jump relprogram [value]` value of AR
        - set PC to PSR + value
    ### Conditional Jumps
    - `JEQ [address]|rel [value]|relprogram [value]`
        - Executes the jump if the 1st LSB is 1, when compared is equal  
    - `JNE [address]|rel [value]|relprogram [value]`
        - Executes the jump if the 2nd LSB is 1, when compared is not equal
    - `JGT [address]|rel [value]|relprogram [value]`
        - Executes the jump if the 3rd LSB is 1, when compared is greater than
    - `JGE [address]|rel [value]|relprogram [value]`
        - Executes the jump if the 4th LSB is 1, when compared is greater than or equal
    - `JLT [address]|rel [value]|relprogram [value]`
        - Executes the jump if the 5th LSB is 1, when compared is less than
    - `JLE [address]|rel [value]|relprogram [value]`
        - Executes the jump if the 6th LSB is 1, when compared is less than or equal
    - `JNG [address]|rel [value]|relprogram [value]`
        - Executes the jump if the 7th LSB is 1, when compared is negative
    - `JZR [address]|rel [value]|relprogram [value]`
        - Executes the jump if the 8th LSB is 1, when compared is zero

- ## Compare (56 instructions)
    - `Compare [reg1] to [reg2]`
        - Where reg1 is not equal to reg2

- ## Push/Pop instructions (2 instructions)
    - `Push`
        - pushes the value of ACC to the stack
    - `Pop`
        - pops the last value of the stack and puts it in ACC
- ## Miscellaneous instructions (41 instructions)
    - `Add [reg]` (8 instructions)
        - ACC = Add reg to ACC
    - `Sub [reg]` (8 instructions)
        - ACC = sub reg to ACC
    - `And [reg]` (8 instructions)
        - ACC = `[reg]` && ACC
    - `Or [reg]` (8 instructions)
        - ACC = `[reg]` || ACC
    - `Not`
        - ACC = `!ACC`
    - `Xor [reg]` (8 instructions)
        - ACC = `[reg] ^ ACC`
