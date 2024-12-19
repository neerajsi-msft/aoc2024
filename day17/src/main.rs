use std::{collections::VecDeque, error::Error, fs, result::Result};
use clap::Parser;
use itertools::Itertools;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use scan_fmt::scan_fmt;

#[derive(clap::clap_derive::Parser, Debug)]
#[command(about)]
/// Simulate robots moving around a toroidal field.
struct Args {
    #[arg(default_value = "input_sample.txt")]
    input_file: String,

    #[arg(short, long, default_value_t = false)]
    debug: bool,

    #[arg(short, long)]
    initial_a: Option<MachineWord>,

    #[arg(short, long, default_value_t = false)]
    part2: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
enum RegNum {
    A = 0,
    B = 1,
    C = 2
}

const A: usize = 0;
const B: usize = 1;
const C: usize = 2;


const REG_COUNT: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
enum Instr {
    Adv = 0,
    Bxl = 1,
    Bst = 2,
    Jnz = 3,
    Bxc = 4,
    Out = 5,
    Bdv = 6,
    Cdv = 7
}

/*
const INSTR_COUNT: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpType {
    Lit,
    Div,
    Combo,
    Reg,
    Ignored
}
*/

use Instr::*;

struct DisplayableInstruction {
    instr: u8,
    operand: u8
}

impl std::fmt::Display for DisplayableInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let instr = Instr::from_u8(self.instr).unwrap();
        write!(f, "{:?} {}: ", instr, self.operand)?;

        fn fmt_lit(out: &mut std::fmt::Formatter<'_>, operand: u8) -> std::fmt::Result {
            write!(out, "#{}", operand)
        }

        fn fmt_combo(out: &mut std::fmt::Formatter<'_>, operand: u8) -> std::fmt::Result {
            match operand {
                0..=3 => fmt_lit(out, operand),
                4..=6 => write!(out, "{}", (b'A' + operand - 4) as char),
                _ => write!(out, "invalid operand: {}", operand),
            }
        }

        let fmt_div = |out: &mut std::fmt::Formatter<'_>, dest| {
            write!(out, "{dest} = A / (1 << ")?;
            fmt_combo(out, self.operand)?;
            write!(out, ")")
        };

        match instr {
            Adv => fmt_div(f, 'A'),
            Bdv => fmt_div(f, 'B'),
            Cdv => fmt_div(f, 'C'),
            Bxl => {write!(f, "B ^= ")?; fmt_lit(f, self.operand)},
            Bst => {write!(f, "B = ")?; fmt_combo(f, self.operand)?; write!(f, " % 8")},
            Jnz => {write!(f, "jnz ")?; fmt_lit(f, self.operand)},
            Bxc => write!(f, "B ^= C"),
            Out => {write!(f, "Out: ")?; fmt_combo(f, self.operand)?; write!(f, " % 8")},
        }
    }
}

#[derive(Debug)]
struct Puzzle {
    machine_code: Vec<u8>,
    initial_registers: [MachineWord; REG_COUNT],
}

type MachineWord = u64;

#[derive(Debug, Clone)]
struct MachineState {
    registers: [MachineWord; REG_COUNT],
    pc: usize
}

fn get_combo_operand(operand: u8, state: &MachineState) -> MachineWord
{
    match operand {
        0..=3 => operand as MachineWord,
        4..=6 => state.registers[(operand - 4) as usize],
        _ => panic!("Invalid operand: {operand}")
    }
}

fn do_div_instr(operand: u8, state: &MachineState) -> MachineWord
{
    let operand = get_combo_operand(operand, state);
    
    state.registers[RegNum::A as usize] / (1 << operand)
}

fn step_program(puzzle: &Puzzle, state: &MachineState, output: &mut Vec<u8>, args: &Args) -> Option<MachineState>
{
    if state.pc > puzzle.machine_code.len() - 2 { return None };

    let instr = puzzle.machine_code[state.pc];
    let instr = Instr::from_u8(instr).unwrap();
    let operand = puzzle.machine_code[state.pc + 1];
    if args.debug {
        println!("{state:?}: {instr:?} {operand:?}")
    }
    
    let mut new_state = state.clone();
    new_state.pc += 2;
    match instr {
        Adv => {new_state.registers[A] = do_div_instr(operand, &new_state)},
        Bdv => {new_state.registers[B] = do_div_instr(operand, &new_state)},
        Cdv => {new_state.registers[C] = do_div_instr(operand, &new_state)},
        Bxl => {new_state.registers[B] ^= operand as MachineWord},
        Bst => {new_state.registers[B] = get_combo_operand(operand, &new_state) % 8},
        Jnz => {if new_state.registers[A] != 0 { new_state.pc = operand.into() }},
        Bxc => {new_state.registers[B] ^= new_state.registers[C]},
        Out => {output.push((get_combo_operand(operand, &new_state) % 8) as u8)}
    };

    Some(new_state)
}

fn run_program(puzzle: &Puzzle, args: &Args, a_value: MachineWord, mut output: Vec<u8>) -> Vec<u8> {
    let mut state = MachineState{registers: puzzle.initial_registers, pc: 0};
    state.registers[A] = a_value;
    
    while let Some(new_state) = step_program(puzzle, &state, &mut output, args) {
        state = new_state;
    }

    output
}

fn disassemble_program(puzzle: &Puzzle) {
    for (offs, instr) in puzzle.machine_code.chunks_exact(2).enumerate() {
        let [instr, operand] = instr else { unreachable!() };
        println!("{offs}: {instr} {operand} {}", DisplayableInstruction{instr: *instr, operand: *operand});
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::try_parse()?;

    let str = fs::read_to_string(&args.input_file)?;

    let mut line_iter = str.lines();
    let mut reg_count = 0;
    let mut initial_registers = [0 as MachineWord; REG_COUNT];
    for l in line_iter.by_ref().take(3) {
        let (reg, val) = scan_fmt!(l, "Register {}: {}", char, MachineWord)?;
        initial_registers[(reg as u8 - b'A') as usize] = val;
        reg_count += 1;
    }

    assert_eq!(reg_count, 3);

    assert_eq!(line_iter.next(), Some(""));

    let prog_line = line_iter.next().expect("Missing program line");

    const PROGRAM: &str = "Program: ";

    let (title, prog) = prog_line.split_at(PROGRAM.len());
    assert_eq!(title, PROGRAM);

    let machine_code: Vec<u8> = prog.split(',')
        .map(
            |s| s.parse::<u8>()
        )
        .try_collect()?;

    assert!(machine_code.iter().all(|c| *c <= 7));
   
    let puzzle = Puzzle{machine_code, initial_registers};

    println!("initial state: {puzzle:?}");

    disassemble_program(&puzzle);

    if !args.part2 {
        let a_value = args.initial_a.unwrap_or(puzzle.initial_registers[A]);
        let output = run_program(&puzzle, &args, a_value, Vec::new());
        println!("{}", output.iter().format(","));

    } else {
        let machine_code = &puzzle.machine_code;
        let mut solutions = Vec::new();
        let mut a_candidates: VecDeque<(MachineWord, usize)> = VecDeque::new();
        a_candidates.push_back((0, 1));
        let mut output = Vec::new();
        while let Some((candidate_a, output_len)) = a_candidates.pop_front()  {
            if output_len > puzzle.machine_code.len() {
                solutions.push(candidate_a);
                continue;
            }

            let output_len = output_len;

            let target_output = &machine_code[(machine_code.len() - output_len)..];

            for a_bits in 0..=7 {
                let a_value = (candidate_a << 3) | a_bits;
                output.clear();
                output = run_program(&puzzle, &args, a_value, output);

                if output == target_output {
                    a_candidates.push_back((a_value, output_len + 1));
                }
            }
        }

        solutions.sort();
        println!("part2: {solutions:?}");
    }

    Ok(())
}
