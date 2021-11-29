use assembler::instr::Instruction;
use std::{fmt::Display, io::BufRead};

#[derive(Clone, Default)]
pub struct State {
    pub inst_memory: Vec<Instruction>,
    /// Memory *should* be byte addressable, but in the tests loads and stores are always word aligned
    pub data_memory: Vec<u32>,
    pub registers: [u32; 32],
    pub program_counter: usize,
    pub instructions_count: usize,

    // Forwarding contents
    pub fet_dec: FetchDecode,
    pub dec_exc: DecodeExecute,
    pub exc_mem: ExecuteMemory,
    pub mem_wrt: MemoryWrite,
    pub wrt_end: WriteEnd,
}

impl State {
    pub fn with_memory(memory: impl BufRead) -> Self {
        let data_memory: Vec<u32> = memory
            .lines()
            .enumerate()
            .map(|(idx, line)| {
                let line = line.unwrap();
                let bits: u32 = u32::from_str_radix(&line, 16).unwrap();
                println!("memory[{}]={:x}", idx, bits);
                bits
            })
            .collect();

        println!("{} memory words", data_memory.len());

        let inst_memory: Vec<Instruction> = data_memory
            .iter()
            .map(|word| Instruction::from(*word))
            .collect();

        println!("\tinstruction memory:");
        for (key, &val) in inst_memory.iter().enumerate() {
            println!("\t\tinstrMem[ {} ] = {}", key, val);
        }

        Self {
            inst_memory,
            data_memory,
            ..Default::default()
        }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "@@@\nstate before cycle {} starts",
            self.instructions_count
        )?;
        writeln!(f, "\tpc {}", self.program_counter)?;

        writeln!(f, "\tdata memory:")?;
        for (key, &val) in self.data_memory.iter().enumerate() {
            writeln!(f, "\t\tdataMem[ {} ] {}", key, val as i32)?;
        }

        writeln!(f, "\tregisters:")?;
        for (key, &val) in self.registers.iter().enumerate() {
            writeln!(f, "\t\treg[ {} ] {}", key, val as i32)?;
        }

        writeln!(f, "\tIFID:")?;
        writeln!(f, "\t\tinstruction {}", self.fet_dec.instr)?;
        writeln!(f, "\t\tpcPlus1 {}", self.fet_dec.pc_next)?;
        writeln!(f, "\tIDEX:")?;
        writeln!(f, "\t\tinstruction {}", self.dec_exc.instr)?;
        writeln!(f, "\t\tpcPlus1 {}", self.dec_exc.pc_next)?;
        writeln!(f, "\t\treadRegA {}", self.dec_exc.read_reg_a as i32)?;
        writeln!(f, "\t\treadRegB {}", self.dec_exc.read_reg_b as i32)?;
        writeln!(f, "\t\toffset {}", self.dec_exc.offset as i32)?;
        writeln!(f, "\tEXMEM:")?;
        writeln!(f, "\t\tinstruction {}", self.exc_mem.instr)?;
        writeln!(f, "\t\taluResult {}", self.exc_mem.alu_result as i32)?;
        writeln!(f, "\t\treadRegB {}", self.exc_mem.read_reg as i32)?;
        writeln!(f, "\tMEMWB:")?;
        writeln!(f, "\t\tinstruction {}", self.mem_wrt.instr)?;
        writeln!(f, "\t\twriteData {}", self.mem_wrt.write_data as i32)?;
        writeln!(f, "\tWBEND:")?;
        writeln!(f, "\t\tinstruction {}", self.wrt_end.instr)?;
        writeln!(f, "\t\twriteData {}", self.wrt_end.write_data as i32)?;
        Ok(())
    }
}

#[derive(Default, Clone, Copy)]
pub struct FetchDecode {
    pub instr: Instruction,
    pub pc_next: usize,
}

#[derive(Default, Clone, Copy)]
pub struct DecodeExecute {
    pub instr: Instruction,
    pub pc_next: usize,
    pub read_reg_a: u32,
    pub read_reg_b: u32,
    pub offset: u32,
}

impl DecodeExecute {
    pub fn nop() -> Self {
        Self {
            // SAFETY: ITypeInstructions are invalid iff. their opcode is invalid. But Instructions are guaranteed to have valid opcodes. Not using checked version because we're disregarding the opcode
            offset: unsafe { Instruction::nop().i }.imm_as_i32() as u32, // offset = MathFunc::Add = 32,
            ..Default::default()
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct ExecuteMemory {
    pub instr: Instruction,
    pub alu_result: u32,
    pub read_reg: u32,
}

#[derive(Default, Clone, Copy)]
pub struct MemoryWrite {
    pub instr: Instruction,
    pub write_data: u32,
}

pub type WriteEnd = MemoryWrite;

#[test]
fn data_print() {
    use std::io::Cursor;
    let asm = br#"20420064
fc000000
000010e1
"#;
    let asm = Cursor::new(asm);
    let _ = State::with_memory(asm);
}
