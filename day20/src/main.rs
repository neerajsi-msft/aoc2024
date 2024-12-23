use arrayvec::ArrayVec;
use clap::Parser;
use itertools::Itertools;
use neerajsi::Iterable2d;
use neerajsi::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use thiserror::Error;

#[derive(Debug, Error)]
enum PuzzleError {
    #[error("Parsing error: {0}")]
    ParseError(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromPrimitive)]
enum MapSlot {
    Start = b'S' as isize,
    End = b'E' as isize,
    Wall = b'#' as isize,
    Empty = b'.' as isize,
}

struct Puzzle {
    map: Vec<Vec<MapSlot>>,
    start: Location,
    end: Location,
    rows: usize,
    cols: usize,
}

use neerajsi::CardinalDirectionName::*;

fn solve_part1_again(puzzle: &Puzzle, cutoff: usize, max_td: usize, args: &Args, _timings: &mut TimingBuffer) -> usize {
    let grid = Grid::new(puzzle.rows, puzzle.cols);

    let mut costs = vec2d!(puzzle.rows, puzzle.cols, None);

    let mut track = Vec::new();
    let mut pos = puzzle.start;
    let mut cost = 0usize;
    loop {
        index2d_array!(costs, pos).replace(cost);
        cost += 1;

        track.push(pos);
        if pos == puzzle.end {
            break;
        }

        pos = [W, E, N, S]
            .iter()
            .filter_map(|&d| {
                let next_pos = grid.add_cardinal(pos, d).unwrap();
                if Some(&next_pos) == track.iter().nth_back(1) {
                    return None;
                }
                if index2d_array!(puzzle.map, next_pos) != MapSlot::Wall {
                    return Some(next_pos);
                }

                None
            })
            .exactly_one()
            .unwrap();
    }

    let len = track.len();

    let mut cheats: BTreeMap<usize, Vec<(Location, Location)>> = BTreeMap::new();

    for i in 0..(len - 2) {
        for j in (i + 2)..len {
            let (a, b) = (track[i], track[j]);
            let td = taxicab_distance(a, b);
            let orig_cost = j - i;
            if orig_cost <= td {
                continue;
            }

            let savings = orig_cost - td;
            if td <= max_td && savings >= cutoff {
                cheats.entry(savings).or_default().push((a, b));
            }
        }
    }

    if args.debug {
        for (savings, cheats) in cheats.iter() {
            let count = cheats.len();
            println!("{count} cheats save {savings}");
            println!("\t{:?}", cheats.iter().format(","));
        }
    }

    cheats.values().map(|v| v.len()).sum()
}

fn solve_part1(puzzle: &Puzzle, args: &Args, _timings: &mut TimingBuffer) -> usize {
    let rows = puzzle.rows;
    let cols = puzzle.cols;
    let grid = Grid::new(rows, cols);

    let mut costs = vec2d!(puzzle.rows, puzzle.cols, None);

    let mut pos = puzzle.start;
    let mut cost = 0usize;
    let mut from_dir = W;
    loop {
        index2d_array!(costs, pos).replace(cost);
        cost += 1;

        if pos == puzzle.end {
            break;
        }

        (pos, from_dir) = [W, E, N, S]
            .iter()
            .filter_map(|&d| {
                if d != from_dir {
                    let next_pos = grid.add_cardinal(pos, d).unwrap();
                    if index2d_array!(puzzle.map, next_pos) != MapSlot::Wall {
                        return Some((next_pos, opposite_dir_cardinal(d)));
                    }
                }

                None
            })
            .exactly_one()
            .unwrap();
    }

    let mut cheats: BTreeMap<usize, Vec<(Location, Location)>> = BTreeMap::new();

    for wall in costs.iter().positions2d(|c| *c == None) {
        type CostAndLoc = (usize, Location);

        let mut dirs: ArrayVec<CostAndLoc, 4> = neighbors_cardinal(&costs, wall.into())
            .filter_map(|(l, v)| v.map(|c| (c, l)))
            .collect();

        if dirs.is_empty() {
            continue;
        }

        let mut add_cheats = |dirs: &mut [CostAndLoc]| {
            if dirs.len() < 2 {
                return;
            };
            dirs.sort();

            let mut it = dirs.iter().dedup();
            while let Some(dir1) = it.next() {
                for dir2 in it.clone() {
                    let delta = dir2.0 - dir1.0;
                    let steps = taxicab_distance(dir2.1, dir1.1);
                    if delta > steps {
                        cheats
                            .entry(delta - steps)
                            .or_default()
                            .push((dir1.1, dir2.1));
                    }
                }
            }
        };

        add_cheats(&mut dirs);

        /*
        let mut dirs2: ArrayVec<CostAndLoc, 16> = grid.neighbors_iter_cardinal(wall.into(), &[E, S])
            .filter_map(
                |l|
                    if index2d_array!(costs, l).is_none() {
                        Some(neighbors_cardinal(&costs, l).filter_map(|(l2, c)| c.map(|c| (c, l2))))
                    } else {
                        None
                    }
                )
            .flatten()
            .chain(dirs)
            .collect();

        add_cheats(&mut dirs2);
        */
    }

    if args.debug {
        for (savings, cheats) in cheats.iter() {
            let count = cheats.len();
            println!("{count} cheats save {savings}");
            println!("\t{:?}", cheats.iter().format(","));
        }

        print!("{:4}|", "");
        for c in 0..cols {
            print!("{c:4}|");
        }
        println!();
        for r in 0..rows {
            print!("{r:4}|");
            for c in 0..cols {
                if let Some(cost) = costs[r][c] {
                    print!("{cost:4},");
                } else {
                    print!("****,");
                }
            }

            println!();
        }
    }

    cheats.range(100..).map(|c| c.1.len()).sum()
}

#[derive(Parser, Debug)]
#[command(about)]
/// Simulate robots moving around a toroidal field.
struct Args {
    #[arg(default_value = "input_sample.txt")]
    input_file: String,

    #[arg(short, long, default_value_t = false)]
    debug: bool,

    #[arg(short, long, default_value_t = 2)]
    cutoff: usize,

    #[arg(short, long, default_value_t = 2)]
    max_td: usize,

}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let str = fs::read_to_string(&args.input_file)?;

    let map = str
        .lines()
        .map(|l| {
            l.chars()
                .map(|c| {
                    MapSlot::from_u32(c as u32)
                        .ok_or_else(|| PuzzleError::ParseError(format!("Unknown char {c}")))
                })
                .collect::<Result<Vec<MapSlot>, PuzzleError>>()
        })
        .collect::<Result<Vec<Vec<MapSlot>>, PuzzleError>>()?;

    let start = map
        .iter()
        .positions2d(|v| *v == MapSlot::Start)
        .exactly_one()
        .expect("bad start");

    let end = map
        .iter()
        .positions2d(|v| *v == MapSlot::End)
        .exactly_one()
        .expect("bad end");

    let start = start.into();
    let end = end.into();

    let rows = map.len();
    let cols = map[0].len();

    assert!(map.iter().all(|m| m.len() == cols));

    let puzzle = Puzzle {
        map,
        start: start,
        end,
        rows,
        cols,
    };

    let mut timings = TimingBuffer::new();

    let part1 = time_it("part1", || solve_part1(&puzzle, &args, &mut timings));

    dbg!(part1);

    let part1_again = time_it("part1_again", || solve_part1_again(&puzzle, args.cutoff, args.max_td, &args, &mut timings));
    dbg!(part1_again);

    Ok(())
}
