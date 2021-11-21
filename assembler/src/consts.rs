pub const MAXLINELENGTH: u32 = 1000;
pub const MAXNUMLABELS: u32 = 10000;
pub const MAXLABELLENGTH: u32 = 10;
pub const VALUE_32K: u32 = 32768;
pub const IMMEDIATE_MASK: u32 = 0xFFFF;

pub const OP_SHIFT: u32 = 26;
pub const A_SHIFT: u32 = 21;
pub const B_SHIFT: u32 = 16;
pub const D_SHIFT: u32 = 11;

pub const LW_OP: u32 = 0x23;
pub const SW_OP: u32 = 0x2B;
pub const ADDI_OP: u32 = 0x8;
pub const ADD_OP: u32 = 0x0;
pub const SLL_OP: u32 = 0x0;
pub const SRL_OP: u32 = 0x0;
pub const SUB_OP: u32 = 0x0;
pub const AND_OP: u32 = 0x0;
pub const OR_OP: u32 = 0x0;
pub const BEQZ_OP: u32 = 0x4;
pub const JALR_OP: u32 = 0x13;
pub const HALT_OP: u32 = 0x3F;

pub const ADD_FUNC: u32 = 0x20;
pub const SLL_FUNC: u32 = 0x4;
pub const SRL_FUNC: u32 = 0x6;
pub const SUB_FUNC: u32 = 0x22;
pub const AND_FUNC: u32 = 0x24;
pub const OR_FUNC: u32 = 0x25;
