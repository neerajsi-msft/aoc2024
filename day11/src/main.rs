use core::num;
use std::cmp::min;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::error::Error;
use std::env;
use std::fs;
use std::ops::Index;
use std::path;
use std::slice::SliceIndex;
use std::thread::current;
use std::time::Instant;
use bit_set::BitSet;
use itertools::Itertools;
use arrayvec::ArrayVec;

fn time_it<T>(name: &str, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();

    let ret = f();
    
    let elapsed = start.elapsed();
    println!("{name} took: {elapsed:?}");
    
    ret
}
struct Puzzle {
    stones: Vec<u64>
}


impl Puzzle {
}

fn try_split(x: u64) -> Option<[u64;2]> {
    let number_of_digits = x.ilog10() + 1;
    if (number_of_digits % 2 == 0) {
        let half = 10u64.pow(number_of_digits / 2);
        Some([x / half, x % half])
    } else {
        None
    }
}

#[test]
fn test_try_split() {
    assert_eq!(try_split(999), None);
    assert_eq!(try_split(9999), Some([99, 99]));
    assert_eq!(try_split(9009), Some([90, 09]));
    assert_eq!(try_split(10001), None);
}

fn stone_rule(stone: u64) -> (u64, Option<u64>)
{
    if stone == 0 {
        (1, None)
    } else if let Some(split) = try_split(stone) {
        (split[0], Some(split[1]))
    } else {
        (stone * 2024, None)
    }

}

fn solve_part1(
    stones: &[u64],
    blink_count: usize,
    debug: bool
) -> usize {
    
    let mut cur_stones = stones.to_vec();
    if (debug) {
        println!("{cur_stones:?}"); 
    }

    for _ in 0..blink_count {
        let mut stone_buffer = Vec::with_capacity(cur_stones.len() * 2);

        for stone in cur_stones {
            let (s1, s2) = stone_rule(stone);
            stone_buffer.push(s1);
            if s2.is_some() {
                stone_buffer.push(s2.unwrap());
            }
        }

        cur_stones = stone_buffer;
        if (debug) {
            println!("{cur_stones:?}"); 
        }
    }

    cur_stones.len()
}

fn solve_part2(
    puzzle: &Puzzle,
    blinks: usize
) -> u64 {
    let mut number_to_count: HashMap<u64, u64> = HashMap::new();

    puzzle.stones.iter().for_each(|s| *number_to_count.entry(*s).or_default() += 1);
    
    for _blink in 0..blinks {
        let mut new_number_to_count = HashMap::new();
        for (key, count) in number_to_count {
            let new_stones = stone_rule(key);
            *new_number_to_count.entry(new_stones.0).or_default() += count;
            if let Some(second) = new_stones.1 {
                *new_number_to_count.entry(second).or_default() += count;
            }
        }

        number_to_count = new_number_to_count;
    }

    number_to_count.iter().map(|v| *v.1).sum::<u64>()
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_name = env::args().nth(1).unwrap_or("input_sample.txt".into());

    let str = fs::read_to_string(file_name)?;
    let stones: Vec<u64> = str
        .trim_ascii()
        .split_ascii_whitespace()
        .map(str::parse::<u64>)
        .try_collect()?;

    let puzzle = Puzzle{stones};

    //let part1_6 = time_it("part1 (6)", || solve_part1(&puzzle, 6, true));
    //dbg!(part1_6);


    let part1 = time_it("part1", || solve_part1(&puzzle.stones, 25, false));
    dbg!(part1);

    /*
    let mut v = puzzle.stones[0];
    let mut v2 = None;
    dbg!(v);
    for i in 0..75 {
        let new = stone_rule(v);
        let new2 = v2.map(|s| stone_rule(s).0).or(new.1);

        v = new.0;
        v2 = new2;
        println!("{i}: {v:?} {v2:?}");
    }
    */

    let part2 = time_it("part2", || solve_part2(&puzzle, 75));
    dbg!(part2);
    Ok(())
}
