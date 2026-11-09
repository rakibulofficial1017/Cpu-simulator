; Move one screen pixel with WASD. Run with:
; ./cpu/target/release/cpu wasd_board.bin --interactive
; Escape exits. The moving pixel is yellow on the 150x40 RGB framebuffer.

    LDI #75
    LDA #9
    STA
    LDI #20
    LDA #10
    STA

poll:
    LDA #0
    LDI #1
    CMP DR, ACC
    JNE poll

    LDA #1
    LDI #0x77
    CMP DR, ACC
    JE move_up
    LDA #1
    LDI #0x61
    CMP DR, ACC
    JE move_left
    LDA #1
    LDI #0x73
    CMP DR, ACC
    JE move_down
    LDA #1
    LDI #0x64
    CMP DR, ACC
    JE move_right
    JMP consume_key

move_up:
    LDA #10
    LDI #0
    CMP DR, ACC
    JE consume_key
    LDA #10
    MOV DR -> ACC
    DEC
    LDA #10
    STA
    JMP consume_key

move_left:
    LDA #9
    LDI #0
    CMP DR, ACC
    JE consume_key
    LDA #9
    MOV DR -> ACC
    DEC
    LDA #9
    STA
    JMP consume_key

move_down:
    LDA #10
    LDI #39
    CMP DR, ACC
    JE consume_key
    LDA #10
    MOV DR -> ACC
    INC
    LDA #10
    STA
    JMP consume_key

move_right:
    LDA #9
    LDI #149
    CMP DR, ACC
    JE consume_key
    LDA #9
    MOV DR -> ACC
    INC
    LDA #9
    STA

consume_key:
    LDI #0
    LDA #0
    STA
    JMP poll
