//! The fields of the instruction types are in reverse order

use std::{fmt::Display, mem::transmute};

use modular_bitfield::{
    bitfield,
    prelude::{B16, B26, B5},
    BitfieldSpecifier,
};
use strum_macros::EnumString;

#[derive(BitfieldSpecifier, EnumString, Clone, Copy, Debug)]
#[strum(serialize_all = "lowercase")]
#[bits = 6]
#[repr(u8)]
pub enum OpCode {
    LW = 0x23,
    SW = 0x2B,
    ADDI = 0x8,
    /// ADD, SLL, SRL, SUB, AND, ADD
    MATH = 0x00,
    BEQZ = 0x04,
    JALR = 0x13,
    HALT = 0x3F,
}

#[derive(Debug, BitfieldSpecifier, EnumString, Clone, Copy)]
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
#[bitfield]
#[derive(Clone, Copy)]
pub struct RTypeInstruction {
    #[bits = 6]
    func: MathFunc,
    /// shamt isn't used in this implementation
    shamt: B5,
    rd: B5,
    rt: B5,
    rs: B5,
    #[bits = 6]
    opcode: OpCode,
}

#[bitfield]
#[derive(Clone, Copy)]
pub struct ITypeInstruction {
    imm: B16,
    rt: B5,
    rs: B5,
    #[bits = 6]
    opcode: OpCode,
}

#[bitfield]
#[derive(Clone, Copy)]
pub struct JTypeInstruction {
    offset: B26,
    #[bits = 6]
    opcode: OpCode,
}

#[derive(Clone, Copy)]
pub union Instruction {
    i: ITypeInstruction,
    j: JTypeInstruction,
    r: RTypeInstruction,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bits: u32 = unsafe { transmute(*self) };
        // let bits = bits.reverse_bits();
        write!(f, "{:08x}", bits)
    }
}

impl Instruction {
    pub fn opcode(&self) -> OpCode {
        unsafe { self.j }.opcode()
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
