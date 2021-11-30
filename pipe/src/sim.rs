use crate::state::*;
use assembler::instr::*;

/// Read the next instruction and perform branch prediction
pub fn fetch(program_counter: usize, inst_memory: &[Instruction]) -> (usize, FetchDecode) {
    let instr = *inst_memory
        .get(program_counter / 4)
        // If we are reading out of bound (past a halt) then read a nop
        // C version does Instruction::from(0), but when later stages use this it may panic
        .unwrap_or(&Instruction::from(0));

    let mut pc = program_counter + 4;

    let pc_next = if let OpCode::BEQZ = instr.opcode() {
        let branch = instr.as_i().unwrap();
        let offset = branch.imm() as i16;
        if offset.is_positive() {
            // predict branch not taken if jumping forward
            pc
        } else {
            // predict branch taken
            // TODO Check conversion
            let offset = isize::from(offset) as usize;
            let old_pc = pc;
            pc = pc.wrapping_add(offset);
            old_pc
        }
    } else {
        pc
    };
    (pc, FetchDecode { instr, pc_next })
}

/// Read registers, possible stalling if we need to wait for a load
pub fn decode(state: &State) -> (Option<(usize, FetchDecode)>, DecodeExecute) {
    let instr = state.fet_dec.instr;
    let old_instr = state.dec_exc.instr;

    // SAFETY: ITypeInstructions are invalid iff. their opcode is invalid. But Instructions are guaranteed to have valid opcodes. Not using checked version because we're disregarding the opcode
    let i_instr = unsafe { &instr.i };

    // By default, forward the output of ifid
    let default = (
        None,
        DecodeExecute {
            instr,
            pc_next: state.fet_dec.pc_next,
            // TODO forwarding here?
            read_reg_a: state.registers[i_instr.rs() as usize],
            read_reg_b: state.registers[i_instr.rt() as usize],
            offset: i_instr.imm() as i16,
        },
    );

    // Alternatively, keep ifid as is and set idex to a nop
    let alt = (
        Some((
            state.program_counter,
            FetchDecode {
                instr,
                pc_next: state.fet_dec.pc_next,
            },
        )),
        DecodeExecute::nop(),
    );

    if matches!(old_instr.opcode(), OpCode::LW) && {
        // If we the previous instruction was a load
        let load = old_instr.as_i().unwrap();
        let dst = load.rt(); // rt or rs?
                             // And it was into a register we're reading from
        (matches!(instr.opcode(), OpCode::MATH) && (dst == i_instr.rs() || dst == i_instr.rt()))
            || (!matches!(instr.opcode(), OpCode::HALT) && i_instr.rs() == dst)
    } {
        // Then we have to stall
        alt
    } else {
        default
    }
}

pub fn execute(state: &State) -> (Option<(usize, FetchDecode, DecodeExecute)>, ExecuteMemory) {
    let (instr, read_reg_a, read_reg_b) = {
        let instr = state.dec_exc.instr;

        // SAFETY: All instructions are valid in the i format
        let i = unsafe { instr.i };
        let r1 = i.rs();
        let r2 = i.rt();

        let (ex_a, ex_b) = forward_to_exc(state.exc_mem.instr, state.exc_mem.alu_result, r1, r2);

        // Avoid waw hazard
        let ex_b = if instr.opcode() != state.exc_mem.instr.opcode() {
            ex_b
        } else {
            None
        };

        let (mem_a, mem_b) = forward_to_exc(state.mem_wrt.instr, state.mem_wrt.write_data, r1, r2);

        let (wrt_a, wrt_b) = forward_to_exc(state.wrt_end.instr, state.wrt_end.write_data, r1, r2);

        // The forward function doesn't handle forwarding from store
        let wrt_b = if matches!(state.wrt_end.instr.opcode(), OpCode::SW) {
            let old_wb = state.wrt_end.instr.as_i().unwrap();

            if r2 != 0 && r2 == old_wb.rt() {
                Some(state.wrt_end.write_data)
                // TODO? Some(math);
            } else {
                None
            }
        } else {
            wrt_b
        };

        let read_reg_a = ex_a.or(mem_a).or(wrt_a).unwrap_or(state.dec_exc.read_reg_a);
        let read_reg_b = ex_b.or(mem_b).or(wrt_b).unwrap_or(state.dec_exc.read_reg_b);

        (instr, read_reg_a, read_reg_b)
    };

    let mut extra = None;

    let (alu_result, read_reg) = match instr.opcode() {
        OpCode::MATH => {
            let instr = *{
                // TODO how to handle data?
                if instr.as_data().is_ok() {
                    Instruction::nop()
                } else {
                    instr
                }
            }
            .as_r()
            .unwrap();

            let (alu, read) = if instr == RTypeInstruction::nop() {
                (0, 0)
            } else {
                let fun = instr.func().as_fn();
                (fun(read_reg_a, read_reg_b), read_reg_b)
            };
            (alu, read)
        }
        OpCode::LW => {
            let i = instr.as_i().unwrap();
            (
                u32::wrapping_add(read_reg_a, i.imm() as u32), // TODO: imm or imm_i32?
                state.registers[i.rt() as usize],
            )
        }
        OpCode::SW => {
            let i = instr.as_i().unwrap();
            (u32::wrapping_add(read_reg_a, i.imm() as u32), read_reg_b)
        }
        OpCode::ADDI => {
            let i = instr.as_i().unwrap();
            (
                u32::wrapping_add(read_reg_a, i.imm_as_i32() as u32),
                state.dec_exc.read_reg_b,
            )
        }
        OpCode::BEQZ => {
            use std::convert::TryInto;

            let offs = state.dec_exc.offset();
            if (offs > 0 && read_reg_a == 0) || (offs < 0 && read_reg_a != 0) {
                /* Incorrect branch prediction */
                let program_counter = state.dec_exc.pc_next.wrapping_add(offs as usize);

                /* Wipe out the previous stages in the pipeline */

                let fet = FetchDecode {
                    instr: Instruction::nop(),
                    pc_next: 0,
                };

                let dec = DecodeExecute::nop();
                extra = Some((program_counter, fet, dec))
            }

            let i = instr.as_i().unwrap();
            (
                u32::wrapping_add(
                    state.dec_exc.pc_next.try_into().unwrap(),
                    i.imm_as_i32() as u32,
                ),
                state.registers[i.rt() as usize],
            )
        }
        OpCode::HALT => (0, 0),
        OpCode::JALR => unimplemented!(),
    };

    (
        extra,
        ExecuteMemory {
            instr,
            alu_result,
            read_reg,
        },
    )
}

fn forward_to_exc(
    old_instr: Instruction,
    old_save: u32,
    r1: u8,
    r2: u8,
) -> (Option<u32>, Option<u32>) {
    match old_instr.opcode() {
        OpCode::MATH => {
            let old_instr = old_instr.as_r().unwrap();

            (
                if r1 != 0 && r1 == old_instr.rd() {
                    Some(old_save)
                } else {
                    None
                },
                if r2 != 0 && r2 == old_instr.rd() {
                    Some(old_save)
                } else {
                    None
                },
            )
        }
        OpCode::LW | OpCode::BEQZ | OpCode::ADDI => {
            let old_instr = old_instr.as_i().unwrap();
            (
                if r1 != 0 && r1 == old_instr.rt() {
                    Some(old_save)
                } else {
                    None
                },
                if r2 != 0 && r2 == old_instr.rt() {
                    Some(old_save)
                } else {
                    None
                },
            )
        }
        OpCode::SW | OpCode::JALR | OpCode::HALT => (None, None),
    }
}

/// Store and (forwarding for Load) is performed in this stage
pub fn memory(exc_mem: &ExecuteMemory, data_memory: &mut [u32]) -> MemoryWrite {
    let instr = exc_mem.instr;

    let write_data = match instr.opcode() {
        OpCode::LW => data_memory[(exc_mem.alu_result / 4) as usize],
        OpCode::SW => {
            let store = instr.as_i().unwrap();
            let addr = store.imm_as_i32() as u32;
            let offset = store.rs().into();
            let val_to_store = exc_mem.read_reg;
            data_memory[(addr.wrapping_add(offset) / 4) as usize] = val_to_store;
            val_to_store
        }
        OpCode::ADDI | OpCode::BEQZ | OpCode::HALT | OpCode::MATH => exc_mem.alu_result,
        OpCode::JALR => unimplemented!(),
    };

    MemoryWrite { instr, write_data }
}

/// Write back to registers
pub fn writeback(state: &mut State) -> (bool, WriteEnd) {
    let instr = state.mem_wrt.instr;
    let wbe = state.mem_wrt;

    match instr.opcode() {
        OpCode::LW | OpCode::ADDI => {
            let instr = instr.as_i().unwrap();
            state.registers[instr.rt() as usize] = wbe.write_data;
        }

        OpCode::SW | OpCode::BEQZ => { /* nop */ }

        OpCode::MATH => {
            let instr = instr.as_r().unwrap();
            state.registers[instr.rd() as usize] = wbe.write_data;
        }
        OpCode::HALT => {
            println!("machine halted");
            println!("total of {} cycles executed", state.instructions_count);
        }
        OpCode::JALR => unimplemented!(),
    }

    (matches!(instr.opcode(), OpCode::HALT), wbe)
}
