use bit_set::BitSet;
use core::str;
use petgraph::graphmap::{DiGraphMap, GraphMap};
use rayon::prelude::*;
use scan_fmt::scan_fmt;
use std::{
    cell::{self, RefCell}, env,
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Read},
};
use thiserror::Error;

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
}

#[derive(Debug, Clone)]
struct Puzzle {
}

impl Puzzle {
}

fn parse_puzzle<'a>(
    lines: impl Iterator<Item = Result<impl AsRef<str> + 'a, impl Error + 'static>>,
) -> Result<Puzzle, Box<dyn Error>> {
    let map = lines
        .map(|l| l.map(|s| s.as_ref().as_bytes().to_vec()))
        .collect::<Result<Vec<_>, _>>()?;

    return Ok(Puzzle {
    });
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

    Ok(())
}
