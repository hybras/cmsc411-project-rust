use std::{fmt::Display, io::BufRead};

use assembler::instr::Instruction;

#[derive(Clone, Default)]
pub struct State {
    pub inst_memory: Vec<u32>,
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
                println!("memory[{}]={:08x}", idx, bits);
                bits
            })
            .collect();

        println!("{} memory words", data_memory.len());

        let inst_memory = data_memory.clone();

        println!("\tinstruction memory:");
        for (key, &val) in inst_memory.iter().enumerate() {
            println!("\t\tinstrMem[ {} ] {}", key, Instruction::from(val));
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
        writeln!(f, "\tpc={}", self.program_counter)?;

        writeln!(f, "\tdata memory:")?;
        for (key, &val) in self.data_memory.iter().enumerate() {
            writeln!(f, "\t\tdataMem[ {} ] {}", key, val as i32)?;
        }

        writeln!(f, "\tregisters:")?;
        for (key, &val) in self.registers.iter().enumerate() {
            writeln!(f, "\t\treg[ {} ]  {}", key, val as i32)?;
        }

        writeln!(f)?;
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
