use std::{collections::VecDeque, error::Error, fs, usize};
use clap::Parser;
use thiserror::Error;
use neerajsi::*;
use clap_derive::Parser;
use scan_fmt::scan_fmt;
use itertools::Itertools;
use neerajsi::index2d_array as index2d;

#[derive(Debug, Error)]
enum PuzzleError {
    #[error("Parsing error: {0}")]
    ParseError(String),
}

#[derive(Parser, Debug)]
#[command(about)]
/// Simulate robots moving around a toroidal field.
struct Args {
    #[arg(default_value = "input_sample.txt")]
    input_file: String,

    #[arg(short, long, default_value_t = false)]
    sample: bool,

    #[arg(short='t', long, default_value_t = 1024)]
    step_count: usize,

    #[arg(short, long, default_value_t = false)]
    debug: bool,
}

const DIMENSIONS_SAMPLE: [usize;2] = [7, 7];

const DIMENSIONS: [usize;2] = [71, 71];

fn grid<T>(dimensions: [usize; 2], value: T) -> Vec<Vec<T>>
    where T: Clone + Copy
{
    vec2d!(dimensions[0], dimensions[1], value)
}

fn solve_part1(wall_list: &[(usize, usize)], dimensions: [usize;2], step_count: usize, args: &Args) -> usize
{
    let mut wall_map = grid(dimensions, false);
    for &w in wall_list.iter().take(step_count) {
        let w: [usize;2] = w.into();
        index2d!(wall_map, w) = true;
    }

    let wall_map = wall_map;

    let mut cost_map = grid(dimensions, usize::MAX);
    let mut in_queue_map = grid(dimensions, false);

    let mut bfs_queue = VecDeque::new();
    let start = [0, 0];
    let end = [dimensions[0] - 1, dimensions[1] - 1];
    bfs_queue.push_back(end);
    index2d!(in_queue_map, end) = true;
    index2d!(cost_map, end) = 0;

    while let Some(pos) = bfs_queue.pop_front() {
        let cost = index2d!(cost_map, pos) + 1;
        assert_ne!(cost, usize::MAX);
        for (n, &wall) in neighbors_cardinal(&wall_map, pos) {
            if wall { continue };

            let neighbor_cost = &mut index2d!(cost_map, n);
            if cost < *neighbor_cost {
                *neighbor_cost = cost;
                
                if !std::mem::replace(&mut index2d!(in_queue_map, n), true) {
                    bfs_queue.push_back(n);
                }
            }
        }

        index2d!(in_queue_map, pos) = false;
    }

    if args.debug {
        for r in 0..dimensions[0] {
            for c in 0..dimensions[1] {
                let pos = [r, c];
                if index2d!(wall_map, pos) {
                    print!("#");
                } else if index2d!(cost_map, pos) != usize::MAX {
                    print!("*");
                } else {
                    print!(".");
                }
            }
            println!();
        }
        println!();
    }
    
    index2d!(cost_map, start)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let str = fs::read_to_string(&args.input_file)?;

    let wall_list: Vec<(usize, usize)> = str.lines()
        .map(|l| {
            scan_fmt!(l, "{},{}", usize, usize)
        })
        .try_collect()?;

    let dimensions = if args.sample { DIMENSIONS_SAMPLE } else { DIMENSIONS };

    let part1 = solve_part1(&wall_list, dimensions, args.step_count, &args);

    dbg!(part1); 

    if !args.debug{
        let indexes = (0..wall_list.len()).collect_vec();
    
        let index = indexes.partition_point(|&i| {
            solve_part1(&wall_list, dimensions, i, &args) != usize::MAX
        });
    
        dbg!(index);
        dbg!(wall_list[index - 1]);
    }

    Ok(())
}