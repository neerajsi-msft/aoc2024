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

#[derive(Debug, Clone, Error)]
#[error("Parse error on line {0}")]
struct ParseError(usize);

#[derive(Debug, Clone)]
struct Puzzle {
    page_lists: Vec<Vec<u32>>,
    rule_graph: DiGraphMap<u32, ()>
}

impl Puzzle {
    fn can_be_before(&self, a: u32, b: u32) -> bool {
        // a can be before b if there's no rule that says
        // b must be before a.
        !self.rule_graph.contains_edge(b, a)
    }
}

fn parse_puzzle<'a, I>(lines: I) -> Puzzle
where
    I: IntoIterator<Item = String>,
{
    let mut iter = lines.into_iter();

    let rule_graph = iter
        .by_ref()
        .take_while(|l| !l.is_empty())
        .map(|l| {
            scan_fmt!(&l, "{d}|{d}", u32, u32).unwrap()
        }).collect();


    let page_lists = iter
        .map(|l| {
            l.split(',').map(|x| x.parse().unwrap()).collect()
        })
        .collect();

    return Puzzle{page_lists, rule_graph};
}

fn get_middle_number(v: &[u32]) -> u32 {
    v[v.len() / 2]
}

fn is_index_correctly_ordered(i: usize, v: &[u32], puzzle: &Puzzle) -> bool
{
    (i+1..v.len()).all(|j| puzzle.can_be_before(v[i], v[j]))
}

fn is_page_list_correctly_ordered(page_list: &[u32], puzzle: &Puzzle) -> bool {
    (0..page_list.len()).all(|i| is_index_correctly_ordered(i, page_list, puzzle))
}

fn solve_puzzle_part1(puzzle: &Puzzle) -> Vec<usize> {
    let solution_idxs: Vec<usize> = 
            puzzle.page_lists.iter()
                .enumerate()
                .filter_map(
                    |(list_index, page_list)| {
                        if is_page_list_correctly_ordered(page_list, puzzle) { Some(list_index) } else { None }
                    }
                ).collect();

    dbg!(&solution_idxs);
    let sum: u32 = 
        solution_idxs.iter().map(|i| {
            get_middle_number(&puzzle.page_lists[*i])
        }).sum();

    dbg!(sum);

    solution_idxs
}

fn solve_puzzle_part2(puzzle: &Puzzle, part1_solutions: &[usize])
{
    let mut part2: Vec<_> = puzzle.page_lists.iter().map(Vec::as_slice).collect();

    part1_solutions.iter().rev().for_each(|&i| {part2.remove(i);});

    let correctly_ordered_lists = part2.iter().map(
        |page_list| {
            let mut fixed = page_list.to_vec();
            for fixed_count in 0..fixed.len() {
                let remaining = &mut fixed[fixed_count..];
             
                let next_idx = (0..remaining.len()).find(|&i| is_index_correctly_ordered(i, remaining, puzzle)).unwrap();

                remaining[..=next_idx].rotate_right(1);
            }

            /*
            dbg!(page_list);
            dbg!(&fixed);
            */
            fixed
        }
    );

    let part2_sum = correctly_ordered_lists.map(|page_list| get_middle_number(&page_list)).sum::<u32>();

    dbg!(part2_sum);
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
        .lines()
        .flatten();
    
    let puzzle = parse_puzzle(lines);

    let part1_indexes = solve_puzzle_part1(&puzzle);

    solve_puzzle_part2(&puzzle, &part1_indexes);

    Ok(())
}
