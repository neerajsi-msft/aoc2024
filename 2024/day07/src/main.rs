use bit_set::BitSet;
use itertools::Itertools;
use core::str;
use petgraph::graphmap::{DiGraphMap, GraphMap};
use rayon::prelude::*;
use scan_fmt::scan_fmt;
use std::{
    cell::{self, RefCell}, env, error::Error, fs::File, io::{BufRead, BufReader, Read}
};
use thiserror::Error;

fn time_it<T>(name: &str, func: impl FnOnce() -> T) -> T {
    let start = std::time::Instant::now();

    let ret = func();

    let elapsed = start.elapsed();
    println!("{}: {} seconds", name, elapsed.as_secs_f32());
    ret
}

#[derive(Debug, Clone, Error)]
#[error("Command line error {0}")]
struct CommandLineError(String);

impl CommandLineError {
    fn new(msg: &str) -> Self {
        CommandLineError {
            0: String::from(msg),
        }
    }
}

#[derive(Debug, Clone, Error, PartialEq)]
enum PuzzleError {
    #[error("Parse error at line {0}")]
    ParseError(usize),
    #[error("Parse error at line {0}: {1}")]
    ParseErrorWithStr(usize, String),
}

type PuzzleLine = (u64, Vec<u64>);

#[derive(Debug, Clone)]
struct Puzzle {
    lines: Vec<PuzzleLine>,
}

impl Puzzle {}

fn parse_puzzle<'a>(
    lines: impl Iterator<Item = Result<impl AsRef<str> + 'a, impl Error + 'static>>,
) -> Result<Puzzle, Box<dyn Error>> {
    let map = lines
        .enumerate()
        .map(|(line_no, l)| {
            l.map(|l| -> Result<_, Box<dyn Error>> {
                let mut values = l.as_ref().split_ascii_whitespace();

                let Some(sum) = values.next() else {Err(PuzzleError::ParseError(line_no))?};
                let Ok(sum) = scan_fmt!(sum, "{}:", u64) else { Err(PuzzleError::ParseErrorWithStr(line_no, sum.into()))? };

                let operands: Vec<u64> = values.map(|op| op.parse::<u64>() ).try_collect()?;

                Ok((sum, operands))
            })?
        })
        .try_collect()?;

    return Ok(Puzzle {lines: map});
}

fn part1_can_insert_operators(line: &PuzzleLine) -> bool {
    let (sum, operands) = line;

    let sum = *sum;

    assert!(operands.len() != 0);

    assert!(operands.len() < usize::BITS as usize);

    let max_operators = 1usize << (operands.len() - 1);

    for operators in 0..max_operators {
        let mut operands = operands.iter();
        let &first_operand = operands.next().unwrap();

        let value = operands.enumerate().fold(first_operand,
            |acc, (op_index, &operand)| {
                match ((operators >> op_index) & 1) == 1 {
                    true => acc * operand,
                    false => acc + operand,
                }
            });
        
        if value == sum {
            return true;
        }
    }

    false
}

fn solve_part1(puzzle: &Puzzle) {
    let sum: u64 = puzzle.lines.iter()
        .filter(|l|  part1_can_insert_operators(l))
        .map(|l| l.0)
        .sum();

    dbg!(sum);
}

fn try_divide(dividend: u64, divisor: u64) -> Option<u64>
{
    if divisor == 0 { return None };

    let result = dividend / divisor;

    if dividend == result * divisor {
        Some(result)
    } else {
        None
    }
}

fn try_strip_digits(strip_from: u64, digits: u64) -> Option<u64>
{
    if digits == 0 {
        return None
    }

    let mut strip_from = strip_from;
    let mut digits = digits;
    while digits != 0 {
        if (strip_from % 10) != (digits % 10) {
            return None;
        }

        strip_from /= 10;
        digits /= 10;
    }

    Some(strip_from)
}

fn try_subtract(minuend: u64, subtrahend: u64) -> Option<u64>
{
    if minuend < subtrahend {
        None
    } else {
        Some(minuend - subtrahend)
    }

}

fn part2_can_insert_operators_recursive<const Part2: bool>(result: u64, operands: &[u64]) -> bool {
    // base case
    if operands.len() == 1 {
        return result == operands[0];
    };

    let (&last, remaining) = operands.split_last().unwrap();

    if Part2 {
        if let Some(strip_result) = try_strip_digits(result, last) {
            if part2_can_insert_operators_recursive::<Part2>(strip_result, remaining) {
                return true;
            }
        }
    }

    if let Some(divide_result) = try_divide(result, last) {
        if part2_can_insert_operators_recursive::<Part2>(divide_result, remaining) {
            return true;
        }
    }

    try_subtract(result, last).map_or(false, 
        |res| part2_can_insert_operators_recursive::<Part2>(res, remaining))
}

fn concatenate_digits(a: u64, b: u64) -> u64
{
    let mut multiplier = 1;
    while (multiplier <= b) {
        multiplier *= 10;
    }

    a * multiplier + b
}

fn part2_can_insert_operators(line: &PuzzleLine) -> bool {
    let (sum, operands) = line;

    let sum = *sum;

    return part2_can_insert_operators_recursive::<true>(sum, operands);

    // This doesn't work for some reason.
    /*
    let max_operators = 3usize.pow(operands.len() as u32 - 1);

    for operators in 0..max_operators {
        let mut acc = operands[0];
        let mut operators = operators;
        for op_index in 1..operands.len() {
            let next_op = operands[op_index];
            acc = match operators % 3 {
                0 => acc + next_op,
                1 => acc * next_op,
                2 => concatenate_digits(acc, next_op),
                _ => unreachable!(),
            };

            operators /= 3;
        }

        if sum == acc {
            return true
        }
    }
    */

    false
}


fn solve_part2(puzzle: &Puzzle) -> u64 {
    let part2_sum: u64 = puzzle.lines.iter()
        .filter(|l|  part2_can_insert_operators(l))
        .map(|l| l.0)
        .sum();

    part2_sum
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    if args.len() > 2 {
        return Err(CommandLineError::new("Unexpected arg count.").into());
    }

    let file_name = args.nth(1).unwrap_or("input_sample.txt".into());

    println!("Opening file {}", file_name);

    let reader = BufReader::new(File::open(file_name)?);
    let lines = reader.lines();

    let puzzle = parse_puzzle(lines)?;
    solve_part1(&puzzle);

    let max_line = puzzle.lines.iter().map(|l| l.1.len()).max().unwrap();
    dbg!(max_line);

    let part2_solution = time_it("part2(serial)", ||solve_part2(&puzzle));

    dbg!(part2_solution);

    Ok(())
}
