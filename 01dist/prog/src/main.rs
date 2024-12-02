use std::{
    collections::HashMap,
    env,
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
    iter::zip,
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

#[derive(Debug, Clone, Error)]
enum ParseError {
    #[error("Parsing error on line {0}")]
    LineError(u32),
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    if args.len() != 2 {
        return Err(CommandLineError::new("Unexpected arg count.").into());
    }

    let file_name = args.nth(1).unwrap();

    println!("Opening file {}", file_name);

    let mut a: Vec<i32> = Vec::new();
    let mut b: Vec<i32> = Vec::new();

    let mut line_number = 0u32;
    let reader = BufReader::new(File::open(file_name)?);
    for line in reader.lines() {
        line_number += 1;
        let line = line?;

        let get_fn = |x: Option<&str>| -> Result<i32, Box<dyn Error>> {
            match x {
                Some(i_str) => i_str.parse::<i32>().map_err(|e| e.into()),
                None => Err(ParseError::LineError(line_number).into()),
            }
        };

        let mut vals = line.split_ascii_whitespace();
        a.push(get_fn(vals.next())?);
        b.push(get_fn(vals.next())?);
        if vals.next().is_some() {
            return Err(ParseError::LineError(line_number).into());
        }
    }

    dist_sum_abs_diff(&a, &b);
    dist_similarity_score(&a, &b);
    Ok(())
}

fn dist_sum_abs_diff(a_in: &Vec<i32>, b_in: &Vec<i32>) {
    let mut a = a_in.clone();
    let mut b = b_in.clone();

    a.sort();
    b.sort();

    let dist: u32 = zip(a, b).map(|(ai, bi)| return ai.abs_diff(bi)).sum();

    println!("#1 sum_abs_diff {}", dist);
}

fn dist_similarity_score(a: &Vec<i32>, b: &Vec<i32>) {
    let mut map = HashMap::<i32, i32>::new();

    b.iter().for_each(|x| {
        *map.entry(*x).or_insert(0) += 1;
    });

    let similarity: i32 = a.iter().map(|x| map.get(x).unwrap_or(&0) * x).sum();

    println!("#2 join_count {}", similarity);
}

