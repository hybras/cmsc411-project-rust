use std::{fs::File, io::BufReader};

use anyhow::{Context, Result};

use pipe::{state::*, sim::*};

fn main() -> Result<()> {
    use std::env::args;

    let file = args().nth(1).context("Bad cli args")?;
    let file = File::open(file)?;
    let mem = BufReader::new(file);

    let state = State::with_memory(mem);
    run(state)?;
    Ok(())
}

fn run(mut state: State) -> Result<()> {
    loop {
        print!("{}", state);
        let instructions_count = state.instructions_count + 1;

        let f = fetch(state.program_counter, &state.inst_memory);

        let (maybe_f, dec_exc) = decode(&state);
        let (program_counter, fet_dec) = maybe_f.unwrap_or(f);

        let (maybe, exc_mem) = execute(&state);
        let (program_counter, fet_dec, dec_exc) =
            maybe.unwrap_or((program_counter, fet_dec, dec_exc));

        let mem_wrt = memory(&state.exc_mem, &mut state.data_memory);

        let (halt, wrt_end) = writeback(&mut state);

        state = State {
            program_counter,
            instructions_count,
            fet_dec,
            dec_exc,
            exc_mem,
            mem_wrt,
            wrt_end,
            ..state
        };

        if halt {
            break;
        }
    }
    Ok(())
}
