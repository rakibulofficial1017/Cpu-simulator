use super::{
    instruction::{BranchCondition, Instruction, Opcode},
    micro_op::ALUOperation,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct ControlSignals {
    pub halt: bool,
    pub memory_read: bool,
    pub memory_write: bool,
    pub load_acc: bool,
    pub alu_enable: bool,
    pub alu_subtract: bool,
    pub compare: bool,
    pub jump: bool,
    pub jump_if_zero: bool,
    pub branch_condition: Option<BranchCondition>,
    pub stack_push: bool,
    pub stack_pop: bool,
    pub instruction: Option<Instruction>,
}

pub struct InstructionDecoder;

impl InstructionDecoder {
    pub fn decode(opcode: Opcode) -> ControlSignals {
        let instruction = opcode.instruction();
        let mut signals = ControlSignals {
            instruction: Some(instruction),
            ..Default::default()
        };

        match instruction {
            Instruction::LoadImmediate => signals.load_acc = true,
            Instruction::LoadAddress => signals.memory_read = true,
            Instruction::StoreAddress => signals.memory_write = true,
            Instruction::Move { .. } => {}
            Instruction::Alu { operation, .. } => {
                signals.alu_enable = true;
                signals.alu_subtract =
                    matches!(operation, ALUOperation::Subtract | ALUOperation::Sbc);
            }
            Instruction::Compare { .. } => {
                signals.alu_enable = true;
                signals.alu_subtract = true;
                signals.compare = true;
            }
            Instruction::Unary(_) => signals.alu_enable = true,
            Instruction::Jump(condition) => {
                signals.jump = condition == BranchCondition::Always;
                signals.jump_if_zero = condition == BranchCondition::Zero;
                signals.branch_condition = Some(condition);
            }
            Instruction::Call => signals.stack_push = true,
            Instruction::Push(_) => signals.stack_push = true,
            Instruction::Return | Instruction::Pop(_) => signals.stack_pop = true,
            Instruction::Halt => signals.halt = true,
            Instruction::Nop
            | Instruction::Interrupt
            | Instruction::InterruptReturn
            | Instruction::EnableInterrupts
            | Instruction::DisableInterrupts => {}
        }

        signals
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoder_exposes_register_specific_operation_details() {
        let add_pc = InstructionDecoder::decode(Opcode::try_from(0x5F).unwrap());
        let add_acc = InstructionDecoder::decode(Opcode::try_from(0x60).unwrap());

        assert_eq!(
            add_pc.instruction,
            Some(Instruction::Alu {
                operation: ALUOperation::Add,
                source: super::super::instruction::SourceRegister::PC,
            })
        );
        assert_eq!(
            add_acc.instruction,
            Some(Instruction::Alu {
                operation: ALUOperation::Add,
                source: super::super::instruction::SourceRegister::ACC,
            })
        );
        assert!(add_pc.alu_enable);
        assert!(add_acc.alu_enable);
    }
}
