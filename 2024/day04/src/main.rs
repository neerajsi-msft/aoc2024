use core::str;
use std::{
    default, env,
    error::Error,
    fs::{self, File},
    io::{BufRead, BufReader, Read},
    mem,
    ops::Mul,
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

fn add_dir_to_point(pt: (usize, usize), dir: (i8, i8), distance: usize) -> Option<(usize, usize)> {
    if let (Some(nr), Some(nc)) = (
        pt.0.checked_add_signed(dir.0 as isize * distance as isize),
        pt.1.checked_add_signed(dir.1 as isize * distance as isize),
    ) {
        Some((nr, nc))
    } else {
        None
    }
}

fn lookup_value(rows: &[&[u8]], pt: Option<(usize, usize)>) -> Option<u8> {
    pt.and_then(|pt| rows.get(pt.0).and_then(|r| r.get(pt.1)))
        .copied()
}

fn check_point_wordsearch(rows: &[&[u8]], pt: (usize, usize)) -> usize {
    const SEARCH_STR: &[u8] = "XMAS".as_bytes();
    const DIRVECTORS: [(i8, i8); 8] = [
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (0, -1),
        (0, 1),
        (1, -1),
        (1, 0),
        (1, 1),
    ];

    let mut matches = 0usize;

    if rows[pt.0][pt.1] == SEARCH_STR[0] {
        for v in DIRVECTORS {
            let mut res = true;
            // println!("ptv: {:?}", (pt, v));
            for d in 1..SEARCH_STR.len() {
                let np = add_dir_to_point(pt, v, d);
                let value = lookup_value(rows, np);
                // println!("\t{:?}", (np, value.map(|v| v as char)));
                if Some(SEARCH_STR[d]) != value {
                    res = false;
                    break;
                }
            }

            if (res) {
                // println!("\tmatch!");
            }

            matches += res as usize;
        }
    }

    matches
}

fn check_point_x(rows: &[&[u8]], pt: (usize, usize)) -> bool {
    const DIRVECTORS: [[(i8, i8); 2]; 2] = [[(1, 1), (-1, -1)], [(-1, 1), (1, -1)]];

    if lookup_value(rows, Some(pt)) != Some(b'A') {
        return false;
    }

    let cells = DIRVECTORS.map(|v| v.map(|d| lookup_value(rows, add_dir_to_point(pt, d, 1))));

    cells
        .iter()
        .all(|&c| c == [Some(b'M'), Some(b'S')] || c == [Some(b'S'), Some(b'M')])
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    if args.len() != 2 {
        return Err(CommandLineError::new("Unexpected arg count.").into());
    }

    let file_name = args.nth(1).unwrap();

    println!("Opening file {}", file_name);

    let buf = fs::read(file_name)?;

    const DELIMITER: u8 = b'\n';

    let rows: Vec<&[u8]> = buf
        .split(|&x| x == DELIMITER)
        .map(|s| match s.split_last() {
            Some((&DELIMITER, rest)) => rest,
            _ => s,
        })
        .collect();

    dbg!(rows
        .iter()
        .map(|r| std::str::from_utf8(r).unwrap())
        .collect::<Vec<_>>());

    assert!(rows.iter().all(|r| r.len() == rows[0].len()));

    let mut matches = 0usize;
    let mut matches_x = 0usize;
    for r in 0..rows.len() {
        for c in 0..rows[r].len() {
            let pt = (r, c);
            matches += check_point_wordsearch(rows.as_slice(), pt);

            matches_x += check_point_x(rows.as_slice(), pt) as usize;
        }
    }

    dbg!(matches);
    dbg!(matches_x);

    Ok(())
}
