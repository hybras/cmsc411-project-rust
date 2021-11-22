pub const OP_SHIFT: u32 = 26;
pub const A_SHIFT: u32 = 21;
pub const B_SHIFT: u32 = 16;
pub const D_SHIFT: u32 = 11;

#[derive(Debug)]
#[repr(u8)]
enum OpCode {
    LW = 0x23,
    SW = 0x2B,
    ADDI = 0x8,
    /// ADD, SLL, SRL, SUB, AND, ADD
    MATH = 0x0,
    BEQZ = 0x4,
    JALR = 0x13,
    HALT = 0x3F,
}

#[repr(u8)]
enum MathFunc {
    ADD = 0x20,
    SLL = 0x4,
    SRL = 0x6,
    SUB = 0x22,
    AND = 0x24,
    OR = 0x25,
}

// This is the only encode instruction w/o type safety. All of the others use appropriate integers types to ensure values are in range
fn encode_r_instr(op: OpCode, args: (u32, u32, u32)) -> u32 {
    let op = op as u32;
    (op << OP_SHIFT) | (args.1 << A_SHIFT) | (args.2 << B_SHIFT) | (args.0 << D_SHIFT)
}

fn encode_math_instr(func: MathFunc, args: (u8, u8, u8)) -> u32 {
    let op = OpCode::MATH;
    let func = func as u32;
    encode_r_instr(op, (args.0 as u32, args.1 as u32, args.2 as u32)) | func
}

fn encode_jalr(address: u16) -> u32 {
    encode_r_instr(OpCode::JALR, (0, address as u32, 0))
}

fn encode_halt() -> u32 {
    encode_r_instr(OpCode::HALT, (0, 0, 0))
}

fn encode_i_instr(op: OpCode, args: (u8, u8, i16)) -> u32 {
    debug_assert!(
        matches!(op, OpCode::ADDI | OpCode::BEQZ | OpCode::LW | OpCode::SW),
        "Op code {:?} is not ADDI | BEQZ | LW | SW", op
    );
    let imm = args.2 as u32;
    let op = op as u32;
    op | ((args.1 as u32) << A_SHIFT) | ((args.0 as u32) << B_SHIFT) | imm
}

fn encode_fill(imm: u32) -> u32 {
    imm
}
