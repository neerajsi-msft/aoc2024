use core::{str};
use std::{
    env, error::Error, fs::{File}, io::{BufRead, BufReader, Read}
};
use thiserror::Error;
use petgraph::graphmap::{GraphMap, DiGraphMap};
use scan_fmt::scan_fmt;

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

#[derive(Debug, Clone)]
struct Puzzle {
}

impl Puzzle {
}

fn parse_puzzle<'a>(lines: impl Iterator<Item = Result<impl AsRef<str> + 'a, impl Error + 'static>>) -> Result<Puzzle, Box<dyn Error>>
{
    let collected_lines = lines.collect::<Result<Vec<_>, _>>()?;

    collected_lines.iter().for_each(|s| println!("{}", s.as_ref()));
    return Ok(Puzzle{});
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    if args.len() != 2 {
        return Err(CommandLineError::new("Unexpected arg count.").into());
    }

    let file_name = args.nth(1).unwrap();

    println!("Opening file {}", file_name);

    let reader = BufReader::new(File::open(file_name)?);
    let lines = reader
        .lines();
    
    let puzzle = parse_puzzle(lines)?;

    Ok(())
}
