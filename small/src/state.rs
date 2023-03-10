use std::{fmt::Display, io::BufRead};

#[derive(Default)]
pub struct State {
    /// Memory *should* be byte addressable, but in the tests loads and stores are always word aligned
    pub memory: Vec<u32>,
    pub registers: [u32; 32],
    pub program_counter: usize,
    pub num_executed_instructions: usize,
}

impl State {
    pub fn with_memory(memory: impl BufRead) -> Self {
        let memory = memory
            .lines()
            .enumerate()
            .map(|(idx, line)| {
                let line = line.unwrap();
                let bits: u32 = u32::from_str_radix(&line, 16).unwrap();
                println!("memory[{}]={:08x}", idx, bits);
                bits
            })
            .collect();
        println!();
        Self {
            memory,
            ..Default::default()
        }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "state after cycle {}:", self.num_executed_instructions)?;
        writeln!(f, "\tpc={}", self.program_counter)?;

        writeln!(f, "\tmemory:")?;
        for (key, &val) in self.memory.iter().enumerate() {
            writeln!(f, "\t\tmem[{}] 0x{:x}\t({})", key, val, val as i32)?;
        }

        writeln!(f, "\tregisters:")?;
        for (key, val) in self.registers.iter().enumerate() {
            writeln!(f, "\t\treg[{}] 0x{:x}\t({})", key, val, *val as i32)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory() {
        use std::io::Cursor;
        let line = "deadbeef\n";
        let line = Cursor::new(line);
        let state = State::with_memory(line);
        assert_eq!(state.memory[0], 0xdeadbeef);
    }
}
