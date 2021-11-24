//! The fields of the instruction types are in reverse order

use anyhow::{anyhow, Result};
use modular_bitfield::{
    bitfield,
    prelude::{B16, B26, B5},
    BitfieldSpecifier,
};
use strum_macros::EnumString;

#[derive(BitfieldSpecifier, EnumString, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "lowercase")]
#[bits = 6]
#[repr(u8)]
pub enum OpCode {
    LW = 0x23,
    SW = 0x2B,
    ADDI = 0x8,
    /// ADD, SLL, SRL, SUB, AND, OR
    MATH = 0x00,
    BEQZ = 0x04,
    JALR = 0x13,
    HALT = 0x3F,
}

#[derive(Debug, BitfieldSpecifier, EnumString, Clone, Copy, PartialEq, Eq)]
#[strum(serialize_all = "lowercase")]
#[bits = 6]
#[repr(u8)]
pub enum MathFunc {
    ADD = 0x20,
    SLL = 0x4,
    SRL = 0x6,
    SUB = 0x22,
    AND = 0x24,
    OR = 0x25,
}

pub enum InstructionType {
    J,
    I,
    R,
}
#[bitfield]
#[derive(Clone, Copy)]
pub struct RTypeInstruction {
    #[bits = 6]
    pub func: MathFunc,
    /// shamt isn't used in this implementation
    pub shamt: B5,
    pub rd: B5,
    pub rt: B5,
    pub rs: B5,
    #[bits = 6]
    pub opcode: OpCode,
}

#[bitfield]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ITypeInstruction {
    pub imm: B16,
    pub rt: B5,
    pub rs: B5,
    #[bits = 6]
    pub opcode: OpCode,
}

#[bitfield]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct JTypeInstruction {
    pub offset: B26,
    #[bits = 6]
    pub opcode: OpCode,
}

#[derive(Clone, Copy)]
pub union Instruction {
    pub i: ITypeInstruction,
    pub j: JTypeInstruction,
    pub r: RTypeInstruction,
}

impl From<u32> for Instruction {
    fn from(bits: u32) -> Self {
        unsafe { std::mem::transmute(bits) }
    }
}

impl From<Instruction> for u32 {
    fn from(bits: Instruction) -> Self {
        unsafe { std::mem::transmute(bits) }
    }
}

impl Instruction {
    pub fn opcode(&self) -> OpCode {
        unsafe { self.j }.opcode()
    }

    pub fn instr_type(&self) -> InstructionType {
        use InstructionType::*;

        match self.opcode() {
            OpCode::LW | OpCode::SW | OpCode::ADDI | OpCode::BEQZ => I,
            OpCode::MATH => R,
            OpCode::JALR | OpCode::HALT => J,
        }
    }

    unsafe fn as_i_unchecked(&self) -> &ITypeInstruction {
        unsafe { &self.i }
    }

    unsafe fn as_j_unchecked(&self) -> &JTypeInstruction {
        unsafe { &self.j }
    }

    unsafe fn as_r_unchecked(&self) -> &RTypeInstruction {
        unsafe { &self.r }
    }

    pub fn as_i(&self) -> Result<&ITypeInstruction> {
        if let InstructionType::I = self.instr_type() {
            Ok(unsafe { self.as_i_unchecked() })
        } else {
            Err(anyhow!("Not a i type instruction"))
        }
    }

    pub fn as_r(&self) -> Result<&RTypeInstruction> {
        if let InstructionType::R = self.instr_type() {
            Ok(unsafe { self.as_r_unchecked() })
        } else {
            Err(anyhow!("Not a r type instruction"))
        }
    }

    pub fn as_j(&self) -> Result<&JTypeInstruction> {
        if let InstructionType::J = self.instr_type() {
            Ok(unsafe { self.as_j_unchecked() })
        } else {
            Err(anyhow!("Not a j type instruction"))
        }
    }

    pub fn fill(imm: u32) -> u32 {
        imm
    }

    pub fn jalr(offset: i16) -> Self {
        Self {
            j: JTypeInstruction::new()
                .with_opcode(OpCode::JALR)
                .with_offset(offset as u32),
        }
    }

    pub fn halt() -> Self {
        Self {
            j: JTypeInstruction::new().with_opcode(OpCode::HALT),
        }
    }

    pub fn math(func: MathFunc, args: (u8, u8, u8)) -> Self {
        Self {
            r: RTypeInstruction::new()
                .with_opcode(OpCode::MATH)
                .with_func(func)
                .with_rs(args.1)
                .with_rt(args.2)
                .with_rd(args.0),
        }
    }

    pub fn i_type(op: OpCode, args: (u8, u8, i16)) -> Self {
        debug_assert!(
            matches!(op, OpCode::ADDI | OpCode::BEQZ | OpCode::LW | OpCode::SW),
            "Op code {:?} is not ADDI | BEQZ | LW | SW",
            op
        );

        Self {
            i: ITypeInstruction::new()
                .with_opcode(op)
                .with_rt(args.0)
                .with_rs(args.1)
                .with_imm(args.2 as u16),
        }
    }
}
