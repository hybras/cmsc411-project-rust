use assembler::instr::{Instruction, MathFunc, OpCode};

use std::convert::TryFrom;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
};

use anyhow::Result;
use argh::FromArgs;

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

type Labels = HashMap<String, u16>;

fn main() -> Result<()> {
    let Args {
        input: input_path,
        output,
    } = argh::from_env::<Args>();
    let input = File::open(&input_path)?;
    let output = File::create(output)?;
    let mut output = BufWriter::new(output);

    // First Pass
    let labels = get_labels(&input);
    let input = File::open(&input_path)?;
    write_instructions(&input, &mut output, &labels)?;
    Ok(())
}

fn get_labels(input: &File) -> Labels {
    let input = BufReader::new(input);
    input
        .lines()
        .map(|it| it.unwrap())
        .enumerate()
        .filter_map(|(line_num, line)| {
            let (label, _opcode, _toks) = parse_label_opcode(&line);
             if !label.is_empty() {
                Some((
                    label.to_owned(),
                    (line_num * 4) as u16, // narrowing conversion
                ))
            } else {
                None
            }
        })
        .collect()
}

fn parse_label_opcode(line: &str) -> (&str, &str, impl Iterator<Item = &str>) {
    let mut toks = line.split('\t');
    (toks.next().unwrap(), toks.next().unwrap(), toks)
}

fn write_instructions(input: &File, output: &mut BufWriter<File>, labels: &Labels) -> Result<()> {
    let input = BufReader::new(input);
    for (line_num, line) in input.lines().enumerate() {
        let line = &(line.unwrap());
        let (_label, op, mut toks) = parse_label_opcode(line);
        let instr = if let Ok(func) = op.parse::<MathFunc>() {
            let a0 = toks.next().unwrap().parse().unwrap();
            let a1 = toks.next().unwrap().parse().unwrap();
            let a2 = toks.next().unwrap().parse().unwrap();
            Instruction::math(func, (a0, a1, a2))
        } else if let Ok(op) = op.parse::<OpCode>() {
            match op {
                OpCode::ADDI | OpCode::LW | OpCode::SW | OpCode::BEQZ => {
                    let a0 = toks.next().unwrap().parse().unwrap();
                    let a1 = toks.next().unwrap().parse().unwrap();
                    let imm = toks.next().unwrap();
                    let imm = if let OpCode::BEQZ = op {
                        imm.parse()
                            .or_else(|_| {
                                i16::try_from(labels[imm])
                                    .map(|addr| addr - (line_num as i16) * 4 - 4)
                            })
                            .unwrap()
                    } else {
                        parse_imm(imm, labels)
                    };

                    Instruction::i_type(op, (a0, a1, imm))
                }
                OpCode::JALR => {
                    let offs_or_label = toks.next().unwrap();
                    let offs = parse_imm(offs_or_label, labels);
                    Instruction::jalr(offs)
                }
                OpCode::HALT => {
                    Instruction::halt()
                }
                OpCode::MATH => panic!("MATH is not a assembly instruction. Parsing was already handled for math instructions"),
            }
        } else if op == ".fill" {
            let fill: i32 = toks.next().unwrap().parse().unwrap();
            (fill as u32).into()
        } else {
            panic!("unrecognized opcode {} at line {}", op, line_num + 1)
        };
        let instr = u32::from(instr);
        writeln!(output, "{:08x}", instr)?;
    }
    Ok(())
}

fn parse_imm(imm: &str, labels: &Labels) -> i16 {
    imm.parse().or_else(|_| i16::try_from(labels[imm])).unwrap()
}
