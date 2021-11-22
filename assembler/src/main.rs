pub mod instr;

use std::{collections::HashMap, fs::File, io::{BufRead, BufReader, BufWriter}, path::PathBuf};

use argh::FromArgs;
use anyhow::Result;

/// error: usage: %s <assembly-code-file> <machine-code-file>
#[derive(FromArgs)]
struct Args {
    /// the input mips assembly file
    #[argh(option, short = 'i')]
    input: PathBuf,
    /// the assembled machine code file
    #[argh(option, short = 'o')]
    output: PathBuf,
}

fn main() -> Result<()> {
    let Args {input, output} = argh::from_env::<Args>();
    let input = File::open(input)?;
    let output = File::open(output)?;
    let output = BufWriter::new(output);
    
    // First Pass
    let labels = get_labels(&input);

    Ok(())
}

fn get_labels(input: &File) -> HashMap::<String, u32> {
    let input = BufReader::new(input);
    input
    .lines()
    .enumerate()
        .filter_map(|(line_num, line)| {
            let line = line.unwrap();
            let is_labeled = line.starts_with("\t");
            if is_labeled {
                Some((line.split_once("\t").unwrap().0.to_owned(), (line_num as u32) << 4))
            } else {
                None
            }
        })
        .collect()
}