//! The fields of the instruction types are in reverse order

use std::fmt::Display;

use anyhow::{anyhow, Result};
use modular_bitfield::{
    bitfield,
    prelude::{B16, B26, B5},
    BitfieldSpecifier,
};
use strum_macros::{Display, EnumIter, EnumString};

#[derive(BitfieldSpecifier, EnumString, Display, Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Debug, BitfieldSpecifier, EnumString, EnumIter, Display, Clone, Copy, PartialEq, Eq)]
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

impl MathFunc {
    pub fn as_fn(&self) -> fn(u32, u32) -> u32 {
        use std::ops::{BitAnd, BitOr, Shl, Shr};
        match self {
            MathFunc::ADD => u32::wrapping_add,
            MathFunc::SUB => u32::wrapping_sub,
            MathFunc::SLL => Shl::shl,
            MathFunc::SRL => Shr::shr,
            MathFunc::AND => BitAnd::bitand,
            MathFunc::OR => BitOr::bitor,
        }
    }
}

pub enum InstructionType {
    J,
    I,
    R,
}

#[bitfield]
#[derive(Clone, Copy, PartialEq, Eq)]
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

impl RTypeInstruction {
    pub fn nop() -> Self {
        Self::math(MathFunc::ADD, (0, 0, 0))
    }

    pub fn math(func: MathFunc, args: (u8, u8, u8)) -> Self {
        Self::new()
            .with_opcode(OpCode::MATH)
            .with_func(func)
            .with_rs(args.1)
            .with_rt(args.2)
            .with_rd(args.0)
    }
}

impl Display for RTypeInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.func(),
            self.rd(),
            self.rs(),
            self.rt()
        )
    }
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

impl ITypeInstruction {
    pub fn imm_as_i32(&self) -> i32 {
        i32::from(self.imm() as i16)
    }
}

impl Display for ITypeInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.opcode(),
            self.rt(),
            self.rs(),
            self.imm_as_i32()
        )
    }
}

#[bitfield]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct JTypeInstruction {
    pub offset: B26,
    #[bits = 6]
    pub opcode: OpCode,
}

impl Display for JTypeInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.opcode())?;
        if let OpCode::JALR = self.opcode() {
            write!(f, " {}", self.offset())?;
        }
        Ok(())
    }
}

impl JTypeInstruction {
    pub fn jalr(offset: i16) -> Self {
        Self::new()
            .with_opcode(OpCode::JALR)
            // TODO is this the right conversion? It currently doesn't matter because j.offset() is never used
            .with_offset(offset as u32)
    }

    pub fn halt() -> Self {
        Self::new().with_opcode(OpCode::HALT)
    }
}

#[derive(Clone, Copy)]
pub union Instruction {
    pub i: ITypeInstruction,
    pub j: JTypeInstruction,
    pub r: RTypeInstruction,
}

impl Default for Instruction {
    fn default() -> Self {
        Self::nop()
    }
}

// TODO: Delete this
impl From<u32> for Instruction {
    fn from(bits: u32) -> Self {
        // SAFETY: This is not safe. the u32 may be an invalid value. We need to check that the opcode is valid and (if an r type instruction that func is valid )
        JTypeInstruction {
            bytes: u32::to_le_bytes(bits),
        }
        .into()
    }
}

impl From<Instruction> for u32 {
    fn from(bits: Instruction) -> Self {
        // SAFETY: u32 and Instruction have the same size and alignment, and the valid values of instruction are a subset of the valid values of u32.
        unsafe { std::mem::transmute(bits) }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(bits) = self.as_data() {
            write!(f, "data: {}", bits)
        } else {
            match self.instr_type() {
                InstructionType::J => self.as_j().unwrap().fmt(f),
                InstructionType::I => self.as_i().unwrap().fmt(f),
                InstructionType::R => self.as_r().unwrap().fmt(f),
            }
        }
    }
}

impl From<ITypeInstruction> for Instruction {
    fn from(i: ITypeInstruction) -> Self {
        Self { i }
    }
}

impl From<JTypeInstruction> for Instruction {
    fn from(j: JTypeInstruction) -> Self {
        Self { j }
    }
}

impl From<RTypeInstruction> for Instruction {
    fn from(r: RTypeInstruction) -> Self {
        Self { r }
    }
}

impl Instruction {
    pub fn opcode(&self) -> OpCode {
        // SAFETY: We are only reading opcode, which every instruction type should have in the same place. Any other bits are safely ignored as j.imm
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

    pub fn as_data(self) -> Result<u32, Self> {
        let r = self.as_r();
        match r {
            Ok(r) => {
                let func_is_invalid = r.func_or_err().is_err();
                if func_is_invalid {
                    Ok(self.as_u32())
                } else {
                    Err(self)
                }
            }
            Err(_) => Err(self),
        }
    }

    pub fn as_u32(self) -> u32 {
        let bytes = match self.instr_type() {
            InstructionType::J => self.as_j().unwrap().into_bytes(),
            InstructionType::I => self.as_i().unwrap().into_bytes(),
            InstructionType::R => self.as_r().unwrap().into_bytes(),
        };
        u32::from_le_bytes(bytes)
    }

    pub fn as_i(&self) -> Result<&ITypeInstruction> {
        if let InstructionType::I = self.instr_type() {
            // SAFETY: We just checked instruction type
            Ok(unsafe { &self.i })
        } else {
            Err(anyhow!("Not a i type instruction"))
        }
    }

    pub fn as_r(&self) -> Result<&RTypeInstruction> {
        if let InstructionType::R = self.instr_type() {
            // SAFETY: We just checked instruction type
            Ok(unsafe { &self.r })
        } else {
            Err(anyhow!("Not a r type instruction"))
        }
    }

    pub fn as_j(&self) -> Result<&JTypeInstruction> {
        if let InstructionType::J = self.instr_type() {
            // SAFETY: We just checked instruction type
            Ok(unsafe { &self.j })
        } else {
            Err(anyhow!("Not a j type instruction"))
        }
    }

    pub fn fill(imm: u32) -> u32 {
        imm
    }

    pub fn jalr(offset: i16) -> Self {
        Self {
            j: JTypeInstruction::jalr(offset),
        }
    }

    pub fn halt() -> Self {
        JTypeInstruction::halt().into()
    }

    pub fn nop() -> Self {
        RTypeInstruction::nop().into()
    }

    pub fn math(func: MathFunc, args: (u8, u8, u8)) -> Self {
        RTypeInstruction::math(func, args).into()
    }

    pub fn i_type(op: OpCode, args: (u8, u8, i16)) -> Self {
        debug_assert!(
            matches!(op, OpCode::ADDI | OpCode::BEQZ | OpCode::LW | OpCode::SW),
            "Op code {:?} is not ADDI | BEQZ | LW | SW",
            op
        );

        ITypeInstruction::new()
            .with_opcode(op)
            .with_rt(args.0)
            .with_rs(args.1)
            .with_imm(args.2 as u16)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nop_print() -> Result<()> {
        use std::io::{Cursor, Write};
        let mut c = Cursor::new(vec![0; 4]);
        writeln!(c, "{}", Instruction::nop())?;
        assert_eq!(&c.get_ref()[..], b"nop\n");
        Ok(())
    }
}
