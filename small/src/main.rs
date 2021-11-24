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

    println!("{}", state);

    run(&mut state)?;

    Ok(())
}

fn run<const REGS: usize>(state: &mut State<REGS>) -> Result<()> {
    loop {
        use std::ops::{Add, BitAnd, BitOr, Shl, Shr, Sub};
        // TODO mem needs to be byte addressable
        let instr: Instruction = state.memory[state.program_counter / 4].into();
        state.program_counter += 4;

        match dbg!(instr.opcode()) {
            OpCode::MATH => {
                let instr = instr.as_r()?;
                let fun: fn(u32, u32) -> u32 = match instr.func() {
                    MathFunc::ADD => Add::add,
                    MathFunc::SLL => Shl::shl,
                    MathFunc::SRL => Shr::shr,
                    MathFunc::SUB => Sub::sub,
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
                // Note the mapping between rt/rs and 0/1
                let reg0 = instr.rt() as usize;
                let reg1 = instr.rs() as usize;
                let imm = convert_num(instr.imm() as i16);
                match instr.opcode() {
                    OpCode::LW => {
                        state.registers[reg0 as usize] =
                            state.memory[(state.registers[reg1] + imm) as usize >> 2]
                    }
                    OpCode::SW => {
                        state.memory[(state.registers[reg1] + imm) as usize >> 2] =
                            state.registers[reg0]
                    }
                    OpCode::ADDI => state.registers[reg0] = state.registers[reg1] + imm, // i flipped the regs from the c version
                    OpCode::BEQZ => {
                        if state.registers[reg0] == 0 {
                            state.program_counter += imm as usize;
                        }
                    }
                    _ => unreachable!(),
                }
            }
            OpCode::JALR => unimplemented!(),
            OpCode::NOP => {}
            OpCode::HALT => {
                state.num_executed_instructions += 1;
                println!("machine halted");
                println!(
                    "total of {} instructions executed",
                    state.num_executed_instructions
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

/// Feel free to pass in a u16 with (num as i16)
pub fn convert_num(num: i16) -> u32 {
    /* convert a 16 bit number into a 32-bit Sun number */

    if num < 0 {
        let num = num as u32;
        num - (2u32).pow(16)
    } else {
        num as u32
    }
}
