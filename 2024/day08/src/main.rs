use bit_set::BitSet;
use itertools::Itertools;
use core::str;
use petgraph::graphmap::{DiGraphMap, GraphMap};
use rayon::prelude::*;
use scan_fmt::scan_fmt;
use std::{
    cell::{self, RefCell}, collections::HashMap, env, error::Error, fs::File, io::{BufRead, BufReader, Read}
};
use std::num::NonZeroU8;
use thiserror::Error;
use anyhow::Result;
use vecmath::*;

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
    #[error("Parse error at line {0} ('{1}')")]
    ParseError(usize, String),
}

type PuzzleLine = (u64, Vec<u64>);

#[derive(Debug, Clone)]
struct Puzzle {
    map: Vec<Vec<Option<NonZeroU8>>>,
    rows: usize,
    cols: usize,
    coordinates: HashMap<NonZeroU8, Vec<Vector2<isize>>>
}

impl Puzzle {}

fn parse_puzzle<'a>(
    lines: impl Iterator<Item = Result<impl AsRef<str> + 'a, std::io::Error>>,
) -> anyhow::Result<Puzzle> {
    let map: Vec<Vec<Option<NonZeroU8>>> = lines
        .enumerate()
        .map(|(line_no, l)| -> anyhow::Result<_> {
            let l = l?;
            let line: Result<Vec<Option<NonZeroU8>>, _> = l.as_ref().chars().map( |c| {
                match c {
                    '0'..='9' | 'a'..='z' | 'A'..='Z' => Ok(Some(NonZeroU8::new(c as u8).unwrap())),
                    '.' => Ok(None),
                    _ => Err(())
                }.map_err(|()| PuzzleError::ParseError(line_no, l.as_ref().into()))
            }).try_collect();
            Ok(line?)
        })
        .try_collect()?;

    let rows = map.len();
    let cols = map[0].len();
    assert!(map.iter().all(|r| r.len() == cols));

    let mut coordinates: HashMap<_, Vec<_>> = HashMap::new();
    for r in 0..map.len() {
        for c in 0..map[r].len() {
            let Some(key) = map[r][c] else { continue };
            coordinates.entry(key).or_default().push([r as isize, c as isize]);
        }
    }

    return Ok(Puzzle {map: map, rows, cols, coordinates});
}

fn solve_parts(puzzle: &Puzzle) -> (usize, usize) {
    let mut bitmap_part1 = BitSet::with_capacity(puzzle.rows * puzzle.cols);
    let mut bitmap_part2 = bitmap_part1.clone();
    puzzle.coordinates.values()
        .map(|v|  v.iter().tuple_combinations())
        .flatten()
        .for_each(|(a, b)| {
            let delta = vec2_sub(*b, *a);

            let side_lobes = [vec2_add(*b, delta), vec2_sub(*a, delta)];

            let is_point_in_bounds = |pt: Vector2<isize>| {
                let rows = puzzle.rows as isize;
                let cols = puzzle.cols as isize;

                (0..rows).contains(&pt[0]) &&
                (0..cols).contains(&pt[1])
            };

            let mark_point = |bitmap: &mut BitSet, pt: Vector2<isize>| {
                let x = pt[0] as usize;
                let y = pt[1] as usize;
                bitmap.insert(x * puzzle.cols + y);
            };

            for lobe in side_lobes {
                if is_point_in_bounds(lobe) {
                    mark_point(&mut bitmap_part1, lobe);
                }
            }

            for dir in [-1isize, 1isize] {
                let v = vec2_scale(delta, dir);
                let mut pt = *a;
                while is_point_in_bounds(pt) {
                    mark_point(&mut bitmap_part2, pt);
                    pt = vec2_add(pt, v);
                }
            }
        });

    (bitmap_part1.len(), bitmap_part2.len())
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
    let (part1_result, part2_result) = time_it("Part1", || solve_parts(&puzzle));

    dbg!(part1_result);
    dbg!(part2_result);

    Ok(())
}
