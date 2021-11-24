use std::{fs::File, io::BufReader};

use anyhow::{Context, Result};
use assembler::instr::*;
use small::state::State;

fn main() -> Result<()> {
    use std::env::args;

    let file = args().nth(1).context("Bad cli args")?;
    let file = File::open(file)?;
    let mem = BufReader::new(file);
    let mut state: State<32> = State::with_memory(mem);

    run(&mut state)?;

    Ok(())
}

fn run<const REGS: usize>(state: &mut State<REGS>) -> Result<()> {
    loop {
        use std::ops::{BitAnd, BitOr, Shl, Shr};
        // TODO mem needs to be byte addressable
        let instr: Instruction = state.memory[state.program_counter / 4].into();
        state.program_counter += 4;

        match dbg!(instr.opcode()) {
            OpCode::MATH => {
                let instr = instr.as_r()?;
                let fun: fn(u32, u32) -> u32 = match instr.func() {
                    MathFunc::ADD => u32::wrapping_add,
                    MathFunc::SUB => u32::wrapping_sub,
                    MathFunc::SLL => Shl::shl,
                    MathFunc::SRL => Shr::shr,
                    MathFunc::AND => BitAnd::bitand,
                    MathFunc::OR => BitOr::bitor,
                };
                state.registers[instr.rd() as usize] = fun(
                    state.registers[instr.rs() as usize],
                    state.registers[instr.rt() as usize],
                )
            }

            OpCode::LW | OpCode::SW | OpCode::ADDI | OpCode::BEQZ => {
                let instr = instr.as_i()?;
                // TODO Note the mapping between rt/rs and 0/1
                let reg0 = instr.rs() as usize;
                let reg1 = instr.rt() as usize;
                let imm = convert_num(instr.imm());
                match instr.opcode() {
                    OpCode::LW => {
                        state.registers[reg1] =
                            state.memory[(state.registers[reg0] + imm) as usize >> 2]
                    }
                    OpCode::SW => {
                        state.memory[(state.registers[reg0] + imm) as usize >> 2] =
                            state.registers[reg1]
                    }
                    OpCode::ADDI => state.registers[reg1] = state.registers[reg0] + imm,
                    OpCode::BEQZ => {
                        if state.registers[reg0] == 0 {
                            state.program_counter =
                                (state.program_counter as u32).wrapping_add(imm) as usize;
                        }
                    }
                    _ => unreachable!(),
                }
            }
            OpCode::JALR => unimplemented!(),
            OpCode::NOP => {}
            OpCode::HALT => {
                println!("machine halted");
                println!(
                    "total of {} instructions executed",
                    state.num_executed_instructions + 1 // halt counts as an instruction but doesn't add to the count
                );
                println!("{}", state);
                break;
            }
        }

        // r0 must always be 0. restore it if a rogue instruction modified it
        state.registers[0] = 0;
        println!("{}", state);
        state.num_executed_instructions += 1;
    }

    Ok(())
}

/// converts an i16 to i32, but inputs and outputs unsigned ints
pub fn convert_num(num: u16) -> u32 {
    /* convert a 16 bit number into a 32-bit Sun number */
    // pads the i16 with zeroes. if negative, pads it with 1's instead
    i32::from(num as i16) as u32
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert() {
        let conv = convert_num(u16::MAX);
        println!("{:x}", conv);
    }
}
