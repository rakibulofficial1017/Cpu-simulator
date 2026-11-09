use super::{
    decoder::Decoder,
    instruction::{
        BranchCondition, DestinationRegister, Instruction, InvalidOpcode, SourceRegister,
    },
    micro_op::{ALUOperation, MicroOp, Register16 as R16, Register8 as R8},
};

use crate::{
    cpu::{
        interrupt_controller::{InterruptController, InterruptState},
        micro_op::{ALUDestination, ALUInput},
    },
    hardware::{
        alu::alu,
        pc::PC,
        register::{Register, Register16},
    },
};
use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryRequest {
    pub address: u16,
    pub data: u8,
    pub read: bool,
    pub write: bool,
}
const FLAG_ZR: u8 = 1 << 7;
const FLAG_NG: u8 = 1 << 6;
const FLAG_CY: u8 = 1 << 5;
const FLAG_OV: u8 = 1 << 4;
const FLAG_EQ: u8 = 1 << 3;
const FLAG_GT: u8 = 1 << 2;
const FLAG_LT: u8 = 1 << 1;

#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryResponse {
    pub data: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionExecutionError {
    InvalidOpcode(InvalidOpcode),
    OperandRequired(Instruction),
    UnsupportedInstruction(Instruction),
}

impl fmt::Display for InstructionExecutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOpcode(error) => error.fmt(formatter),
            Self::OperandRequired(instruction) => {
                write!(formatter, "{instruction:?} requires operand bytes")
            }
            Self::UnsupportedInstruction(instruction) => {
                write!(
                    formatter,
                    "execution semantics for {instruction:?} are not defined"
                )
            }
        }
    }
}

impl Error for InstructionExecutionError {}

#[derive(Debug)]
pub struct CPU {
    pub pc: PC,
    pub acc: Register,
    pub temp_a: Register16,
    pub temp_b: Register16,
    pub psr: Register16,
    pub sr: Register,
    pub ar: Register16,
    pub dr: Register,
    pub sp: Register16,
    pub memory_request: MemoryRequest,
    pub ir: Register,
    pub interrupt_controller: InterruptController,
    pub in_interrupt: bool,
    pub interrupt_state: InterruptState,
    pub interrupt_vector: u8,
    pub interrupt_return_pc: u16,
    pub memory_ready: bool,
    pub halted: bool,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            pc: PC::new(),
            acc: Register::new(),
            temp_a: Register16::new(),
            temp_b: Register16::new(),
            psr: Register16::new(),
            sr: Register::new(),
            ar: Register16::new(),
            dr: Register::new(),
            sp: Register16::new(),
            ir: Register::new(),
            memory_request: MemoryRequest::default(),
            interrupt_controller: InterruptController::new(),
            in_interrupt: false,
            interrupt_state: InterruptState::Idle,
            interrupt_vector: 0u8,
            interrupt_return_pc: 0u16,
            memory_ready: false,
            halted: false,
        }
    }

    /// Decode and execute one opcode byte.
    ///
    /// This does not fetch trailing operand bytes or advance `PC`. Instructions
    /// that need operands or have unresolved semantics return an error.
    pub fn execute_instruction(&mut self, opcode: u8) -> Result<(), InstructionExecutionError> {
        let decoded = Decoder::decode(opcode).map_err(InstructionExecutionError::InvalidOpcode)?;
        let instruction = decoded.instruction;

        match instruction {
            Instruction::LoadImmediate | Instruction::LoadAddress | Instruction::Interrupt => {
                return Err(InstructionExecutionError::OperandRequired(instruction));
            }
            Instruction::Push(_)
            | Instruction::Pop(_)
            | Instruction::Call
            | Instruction::Return
            | Instruction::Unary(
                ALUOperation::ShiftLeft
                | ALUOperation::ShiftRight
                | ALUOperation::ShiftArithmeticRight
                | ALUOperation::RotateLeft
                | ALUOperation::RotateRight
                | ALUOperation::RotateCarryLeft
                | ALUOperation::RotateCarryRight,
            )
            | Instruction::InterruptReturn => {
                return Err(InstructionExecutionError::UnsupportedInstruction(
                    instruction,
                ));
            }
            _ => {}
        }

        self.ir.update(opcode, true, true);

        match instruction {
            Instruction::Nop => {}
            Instruction::Halt => self.halted = true,
            Instruction::EnableInterrupts => self.interrupt_controller.enable(),
            Instruction::DisableInterrupts => self.interrupt_controller.disable(),
            Instruction::StoreAddress => {
                self.request_write(self.ar.output(), self.acc.output());
            }
            Instruction::Move {
                source,
                destination,
            } => {
                let value = self.read_source(source);
                self.write_destination(destination, value);
            }
            Instruction::Compare { left, right } => {
                let left = self.read_source(left) as u8;
                let right = self.read_source(right) as u8;
                let (result, borrow) = left.overflowing_sub(right);
                let signed_difference = left as i8 as i16 - right as i8 as i16;
                let overflow = !(i8::MIN as i16..=i8::MAX as i16).contains(&signed_difference);
                self.update_alu_flags(result, Some(!borrow), Some(overflow), Some((left, right)));
            }
            Instruction::Alu { operation, source } => {
                let left = self.acc.output();
                let right = self.read_source(source) as u8;
                self.execute_alu(operation, left, right);
            }
            Instruction::Unary(operation) => {
                self.execute_unary(operation);
            }
            Instruction::Jump(condition) => {
                if self.branch_condition_is_met(condition) {
                    self.pc.update(self.ar.output(), true, false, false, true);
                }
            }
            Instruction::LoadImmediate
            | Instruction::LoadAddress
            | Instruction::Push(_)
            | Instruction::Pop(_)
            | Instruction::Call
            | Instruction::Return
            | Instruction::Interrupt
            | Instruction::InterruptReturn => unreachable!("instruction was rejected above"),
        }

        Ok(())
    }

    fn read_source(&self, source: SourceRegister) -> u16 {
        match source {
            SourceRegister::PC => self.pc.output(),
            SourceRegister::ACC => self.acc.output() as u16,
            SourceRegister::TempA => self.temp_a.output(),
            SourceRegister::TempB => self.temp_b.output(),
            SourceRegister::PSR => self.psr.output(),
            SourceRegister::SR => self.sr.output() as u16,
            SourceRegister::AR => self.ar.output(),
            SourceRegister::DR => self.dr.output() as u16,
        }
    }

    fn write_destination(&mut self, destination: DestinationRegister, value: u16) {
        match destination {
            DestinationRegister::ACC => {
                self.acc.update(value as u8, true, true);
            }
            DestinationRegister::AR => {
                self.ar.update(value, true, true);
            }
            DestinationRegister::TempA => {
                self.temp_a.update(value, true, true);
            }
            DestinationRegister::TempB => {
                self.temp_b.update(value, true, true);
            }
        }
    }

    fn execute_alu(&mut self, operation: ALUOperation, left: u8, right: u8) {
        match operation {
            ALUOperation::Add | ALUOperation::Adc => {
                let carry_in = operation == ALUOperation::Adc && self.sr.output() & FLAG_CY != 0;
                let sum = left as u16 + right as u16 + u16::from(carry_in);
                let result = sum as u8;
                let signed_sum = left as i8 as i16 + right as i8 as i16 + i16::from(carry_in);
                let overflow = !(i8::MIN as i16..=i8::MAX as i16).contains(&signed_sum);
                self.acc.update(result, true, true);
                self.update_alu_flags(result, Some(sum > u8::MAX as u16), Some(overflow), None);
            }
            ALUOperation::Subtract | ALUOperation::Sbc => {
                let borrow_in = operation == ALUOperation::Sbc && self.sr.output() & FLAG_CY == 0;
                let subtrahend = right as u16 + u16::from(borrow_in);
                let result = left.wrapping_sub(right).wrapping_sub(u8::from(borrow_in));
                let signed_difference =
                    left as i8 as i16 - right as i8 as i16 - i16::from(borrow_in);
                let overflow = !(i8::MIN as i16..=i8::MAX as i16).contains(&signed_difference);
                self.acc.update(result, true, true);
                self.update_alu_flags(
                    result,
                    Some(left as u16 >= subtrahend),
                    Some(overflow),
                    None,
                );
            }
            ALUOperation::And | ALUOperation::Test => {
                let result = left & right;
                if operation == ALUOperation::And {
                    self.acc.update(result, true, true);
                }
                self.update_alu_flags(result, None, None, None);
            }
            ALUOperation::Or => {
                let result = left | right;
                self.acc.update(result, true, true);
                self.update_alu_flags(result, None, None, None);
            }
            ALUOperation::Xor => {
                let result = left ^ right;
                self.acc.update(result, true, true);
                self.update_alu_flags(result, None, None, None);
            }
            _ => unreachable!("only register-source ALU operations are decoded here"),
        }
    }

    fn execute_unary(&mut self, operation: ALUOperation) {
        let value = self.acc.output();
        let (result, carry, overflow) = match operation {
            ALUOperation::Increment => {
                let result = value.wrapping_add(1);
                (result, None, Some(value == 0x7F))
            }
            ALUOperation::Decrement => {
                let result = value.wrapping_sub(1);
                (result, None, Some(value == 0x80))
            }
            ALUOperation::Not => (!value, None, None),
            _ => unreachable!("unresolved shift and rotate operations were rejected above"),
        };
        self.acc.update(result, true, true);
        self.update_alu_flags(result, carry, overflow, None);
    }

    fn update_alu_flags(
        &mut self,
        result: u8,
        carry: Option<bool>,
        overflow: Option<bool>,
        comparison: Option<(u8, u8)>,
    ) {
        let mut flags = self.sr.output() & !(FLAG_ZR | FLAG_NG);
        if result == 0 {
            flags |= FLAG_ZR;
        }
        if result & 0x80 != 0 {
            flags |= FLAG_NG;
        }
        if let Some(carry) = carry {
            flags &= !FLAG_CY;
            if carry {
                flags |= FLAG_CY;
            }
        }
        if let Some(overflow) = overflow {
            flags &= !FLAG_OV;
            if overflow {
                flags |= FLAG_OV;
            }
        }
        if let Some((left, right)) = comparison {
            flags &= !(FLAG_EQ | FLAG_GT | FLAG_LT);
            let left = left as i8;
            let right = right as i8;
            if left == right {
                flags |= FLAG_EQ;
            }
            if left > right {
                flags |= FLAG_GT;
            }
            if left < right {
                flags |= FLAG_LT;
            }
        }
        self.sr.update(flags, true, true);
    }

    fn branch_condition_is_met(&self, condition: BranchCondition) -> bool {
        let flags = self.sr.output();
        match condition {
            BranchCondition::Always => true,
            BranchCondition::Zero => flags & FLAG_ZR != 0,
            BranchCondition::NotZero => flags & FLAG_ZR == 0,
            BranchCondition::Negative => flags & FLAG_NG != 0,
            BranchCondition::NotNegative => flags & FLAG_NG == 0,
            BranchCondition::Carry => flags & FLAG_CY != 0,
            BranchCondition::NotCarry => flags & FLAG_CY == 0,
            BranchCondition::Overflow => flags & FLAG_OV != 0,
            BranchCondition::NotOverflow => flags & FLAG_OV == 0,
            BranchCondition::Equal => flags & FLAG_EQ != 0,
            BranchCondition::NotEqual => flags & FLAG_EQ == 0,
            BranchCondition::Greater => flags & FLAG_GT != 0,
            BranchCondition::Less => flags & FLAG_LT != 0,
            BranchCondition::GreaterOrEqual => flags & FLAG_LT == 0,
            BranchCondition::LessOrEqual => flags & FLAG_GT == 0,
        }
    }

    pub fn request_read(&mut self, address: u16) {
        debug_assert!(
            !self.memory_request.read && !self.memory_request.write,
            "Previous memory request has not been serviced"
        );

        self.memory_request = MemoryRequest {
            address,
            data: 0,
            read: true,
            write: false,
        };
    }

    pub fn request_write(&mut self, address: u16, data: u8) {
        debug_assert!(
            !self.memory_request.read && !self.memory_request.write,
            "Previous memory request has not been serviced"
        );

        self.memory_request = MemoryRequest {
            address,
            data,
            read: false,
            write: true,
        };
    }

    pub fn clear_memory_request(&mut self) {
        self.memory_request.read = false;
        self.memory_request.write = false;
    }
    pub fn receive_memory_response(&mut self, response: MemoryResponse, rising_edge: bool) {
        self.dr.update(response.data, true, rising_edge);
    }
    pub fn debug_registers(&self) {
        println!(
            "
Reg     Binary              Hex             Decimal 
PC    = {:016b}, : {:04x},         : {}  
ACC   = {:08b},         : {:02x},           : {}
TempA = {:016b}, : {:04x},         : {}
TempB = {:016b}, : {:04x},         : {}
PSR   = {:016b}, : {:04x},         : {}
SR    = {:08b},         : {:02x},           : {}
AR    = {:016b}, : {:04x},         : {}
DR    = {:08b},         : {:02x},           : {}
SP    = {:016b}, : {:04x},         : {}
IR    = {:08b},         : {:02x},           : {}
",
            self.pc.output(),
            self.pc.output(),
            self.pc.output(),
            self.acc.output(),
            self.acc.output(),
            self.acc.output(),
            self.temp_a.output(),
            self.temp_a.output(),
            self.temp_a.output(),
            self.temp_b.output(),
            self.temp_b.output(),
            self.temp_b.output(),
            self.psr.output(),
            self.psr.output(),
            self.psr.output(),
            self.sr.output(),
            self.sr.output(),
            self.sr.output(),
            self.ar.output(),
            self.ar.output(),
            self.ar.output(),
            self.dr.output(),
            self.dr.output(),
            self.dr.output(),
            self.sp.output(),
            self.sp.output(),
            self.sp.output(),
            self.ir.output(),
            self.ir.output(),
            self.ir.output(),
        );
    }
    pub fn consume_memory_ready(&mut self) -> bool {
        std::mem::take(&mut self.memory_ready)
    }
    pub fn check_interrupts(&mut self) {
        if self.in_interrupt {
            return;
        }

        if self.interrupt_state != InterruptState::Idle {
            return;
        }

        if let Some(interrupt) = self.interrupt_controller.next_pending() {
            self.interrupt_controller.acknowledge(interrupt);

            self.interrupt_vector = interrupt as u8;
            self.begin_interrupt();
        }
    }
    pub fn begin_interrupt(&mut self) {
        self.in_interrupt = true;
        self.interrupt_return_pc = self.pc.output();
        self.interrupt_state = InterruptState::SavePCHigh;
    }
    pub fn execute_micro_op(&mut self, operation: MicroOp, rising_edge: bool) {
        match operation {
            // ================================================
            // 8-BIT TRANSFERS
            // ================================================
            MicroOp::Move8 {
                source,
                destination,
            } => {
                let value = match source {
                    R8::ACC => self.acc.output(),
                    R8::DR => self.dr.output(),
                    R8::SR => self.sr.output(),
                };

                match destination {
                    R8::ACC => {
                        self.acc.update(value, true, rising_edge);
                    }
                    R8::DR => {
                        self.dr.update(value, true, rising_edge);
                    }
                    R8::SR => {
                        self.sr.update(value, true, rising_edge);
                    }
                }
            }

            // ================================================
            // 16-BIT TRANSFERS
            // ================================================
            MicroOp::Move16 {
                source,
                destination,
            } => {
                let value = match source {
                    R16::PC => self.pc.output(),
                    R16::AR => self.ar.output(),
                    R16::TempA => self.temp_a.output(),
                    R16::TempB => self.temp_b.output(),
                    R16::PSR => self.psr.output(),
                    R16::SP => self.sp.output(),
                };

                match destination {
                    R16::PC => {
                        self.pc.update(value, true, false, false, rising_edge);
                    }
                    R16::AR => {
                        self.ar.update(value, true, rising_edge);
                    }
                    R16::TempA => {
                        self.temp_a.update(value, true, rising_edge);
                    }
                    R16::TempB => {
                        self.temp_b.update(value, true, rising_edge);
                    }
                    R16::PSR => {
                        self.psr.update(value, true, rising_edge);
                    }
                    R16::SP => {
                        self.sp.update(value, true, rising_edge);
                    }
                }
            }

            // ================================================
            // 8-BIT → 16-BIT
            // ================================================
            MicroOp::Extend8To16 {
                source,
                destination,
            } => {
                let value = match source {
                    R8::ACC => self.acc.output(),
                    R8::DR => self.dr.output(),
                    R8::SR => self.sr.output(),
                };

                let value = value as u16;

                match destination {
                    R16::PC => {
                        self.pc.update(value, true, false, false, rising_edge);
                    }
                    R16::AR => {
                        self.ar.update(value, true, rising_edge);
                    }
                    R16::TempA => {
                        self.temp_a.update(value, true, rising_edge);
                    }
                    R16::TempB => {
                        self.temp_b.update(value, true, rising_edge);
                    }
                    R16::PSR => {
                        self.psr.update(value, true, rising_edge);
                    }
                    R16::SP => {
                        self.sp.update(value, true, rising_edge);
                    }
                }
            }

            // ================================================
            // 16-BIT → 8-BIT
            // ================================================
            MicroOp::Truncate16To8 {
                source,
                destination,
            } => {
                let value = match source {
                    R16::PC => self.pc.output(),
                    R16::AR => self.ar.output(),
                    R16::TempA => self.temp_a.output(),
                    R16::TempB => self.temp_b.output(),
                    R16::PSR => self.psr.output(),
                    R16::SP => self.sp.output(),
                };

                let value = value as u8;

                match destination {
                    R8::ACC => {
                        self.acc.update(value, true, rising_edge);
                    }
                    R8::DR => {
                        self.dr.update(value, true, rising_edge);
                    }
                    R8::SR => {
                        self.sr.update(value, true, rising_edge);
                    }
                }
            }

            MicroOp::ALU {
                a,
                b,
                update_flags,
                compare,
                destination,
                zx,
                nx,
                zy,
                ny,
                f,
                no,
                cin,
            } => {
                let x = match a {
                    ALUInput::ACC => self.acc.output() as u16,
                    ALUInput::DR => self.dr.output() as u16,
                    ALUInput::SR => self.sr.output() as u16,
                    ALUInput::PC => self.pc.output(),
                    ALUInput::AR => self.ar.output(),
                    ALUInput::TempA => self.temp_a.output(),
                    ALUInput::TempB => self.temp_b.output(),
                    ALUInput::PSR => self.psr.output(),
                    ALUInput::SP => self.sp.output(),
                    ALUInput::Immediate(value) => value,
                };

                let y = match b {
                    ALUInput::ACC => self.acc.output() as u16,
                    ALUInput::DR => self.dr.output() as u16,
                    ALUInput::SR => self.sr.output() as u16,
                    ALUInput::PC => self.pc.output(),
                    ALUInput::AR => self.ar.output(),
                    ALUInput::TempA => self.temp_a.output(),
                    ALUInput::TempB => self.temp_b.output(),
                    ALUInput::PSR => self.psr.output(),
                    ALUInput::SP => self.sp.output(),
                    ALUInput::Immediate(value) => value,
                };

                let result = alu(x, y, zx, nx, zy, ny, f, no, cin);

                match destination {
                    ALUDestination::ACC => {
                        self.acc.update(result.out as u8, true, rising_edge);
                    }
                    ALUDestination::DR => {
                        self.dr.update(result.out as u8, true, rising_edge);
                    }
                    ALUDestination::SR => {
                        self.sr.update(result.out as u8, true, rising_edge);
                    }
                    ALUDestination::PC => {
                        self.pc.update(result.out, true, false, false, rising_edge);
                    }
                    ALUDestination::AR => {
                        self.ar.update(result.out, true, rising_edge);
                    }
                    ALUDestination::TempA => {
                        self.temp_a.update(result.out, true, rising_edge);
                    }
                    ALUDestination::TempB => {
                        self.temp_b.update(result.out, true, rising_edge);
                    }
                    ALUDestination::PSR => {
                        self.psr.update(result.out, true, rising_edge);
                    }
                    ALUDestination::SP => {
                        self.sp.update(result.out, true, rising_edge);
                    }
                }

                if update_flags {
                    let mut flags = self.sr.output();

                    // ZR and NG are always updated.
                    flags &= !(FLAG_ZR | FLAG_NG);

                    if result.zr {
                        flags |= FLAG_ZR;
                    }

                    if result.ng {
                        flags |= FLAG_NG;
                    }

                    // Only update CY and OV when the ALU defines them.
                    if result.cy_valid {
                        flags &= !FLAG_CY;

                        if result.cy {
                            flags |= FLAG_CY;
                        }
                    }

                    if result.ov_valid {
                        flags &= !FLAG_OV;

                        if result.ov {
                            flags |= FLAG_OV;
                        }
                    }

                    // Comparison flags are updated only for comparisons.
                    if compare {
                        flags &= !(FLAG_EQ | FLAG_GT | FLAG_LT);

                        if result.eq {
                            flags |= FLAG_EQ;
                        }

                        if result.gt {
                            flags |= FLAG_GT;
                        }

                        if result.lt {
                            flags |= FLAG_LT;
                        }
                    }

                    self.sr.update(flags, true, rising_edge);
                }
            }
        }
    }
}

#[cfg(test)]
mod instruction_execution_tests {
    use super::*;

    #[test]
    fn executes_register_moves_and_alu_opcodes() {
        let mut cpu = CPU::new();
        cpu.pc.update(0x1234, true, false, false, true);

        cpu.execute_instruction(0x03).unwrap(); // MOV PC -> ACC
        assert_eq!(cpu.acc.output(), 0x34);

        cpu.acc.update(5, true, true);
        cpu.execute_instruction(0x60).unwrap(); // ADD ACC
        assert_eq!(cpu.acc.output(), 10);
    }

    #[test]
    fn compare_updates_flags_and_conditional_jump_uses_them() {
        let mut cpu = CPU::new();
        cpu.acc.update(0x80, true, true);
        cpu.temp_a.update(0x7F, true, true);
        cpu.ar.update(0x2345, true, true);

        cpu.execute_instruction(0x29).unwrap(); // CMP ACC, TempA
        assert_ne!(cpu.sr.output() & FLAG_LT, 0);
        cpu.execute_instruction(0xC1).unwrap(); // JLT
        assert_eq!(cpu.pc.output(), 0x2345);
    }

    #[test]
    fn store_requests_a_write_and_halt_sets_halted() {
        let mut cpu = CPU::new();
        cpu.ar.update(0x4567, true, true);
        cpu.acc.update(0xAB, true, true);

        cpu.execute_instruction(0x02).unwrap(); // STA
        assert_eq!(cpu.memory_request.address, 0x4567);
        assert_eq!(cpu.memory_request.data, 0xAB);
        assert!(!cpu.memory_request.read);
        assert!(cpu.memory_request.write);

        cpu.clear_memory_request();
        cpu.execute_instruction(0xC6).unwrap(); // HLT
        assert!(cpu.halted);
    }

    #[test]
    fn invalid_opcodes_and_missing_operands_are_reported() {
        let mut cpu = CPU::new();
        assert_eq!(
            cpu.execute_instruction(0xCC),
            Err(InstructionExecutionError::InvalidOpcode(InvalidOpcode(
                0xCC
            )))
        );
        assert_eq!(
            cpu.execute_instruction(0x00),
            Err(InstructionExecutionError::OperandRequired(
                Instruction::LoadImmediate
            ))
        );
        assert_eq!(cpu.ir.output(), 0);
    }
}
