use std::{collections::HashMap, env, str::from_utf8};

use neerajsi::*;
use itertools::Itertools;

fn hash(c: u8, hash: u8) -> u8
{
    hash.wrapping_add(c).wrapping_mul(17)
}

fn hash_slice(s: &[u8]) -> usize
{
    s.iter().fold(0, |hv, c| hash(*c, hv)) as usize
}

fn split_slice_once<F, T>(slice: &[T], pred: F) -> Option<(&[T], &[T])>
where
    F: FnMut(&T) -> bool,
{
    let index = slice.iter().position(pred)?;
    Some((&slice[..index], &slice[index + 1..]))
}

fn main() {
    let mut input = read_stdin_input();
    
    input.retain(|i| !(*i as char).is_ascii_whitespace());

    let debug = env::args().nth(1).is_some();

    let part1: usize =
        input.split(|v| *v == b',')
             .map(|s| {
                hash_slice(s)
             })
             .sum();

    dbg!(part1);

    let mut buckets = vec![HashMap::new(); 256];
    for (i, instr) in input.split(|v| *v == b',').enumerate() {
        
        let (label, value) = split_slice_once(instr, |v| matches!(*v, b'='|b'-')).unwrap();
        let hash = hash_slice(label);
        let bucket = &mut buckets[hash];
        if value.is_empty() {
            bucket.remove(label);
        } else {
            bucket.entry(label).or_insert((i, value)).1 = value;
        }
    }

    let mut part2 = 0;
    for (i, b) in buckets.iter().enumerate() {
        let box_no = i + 1;
        let sorted = b.iter()
            .map(|e| {
                (e.1.0, from_utf8(e.1.1).unwrap().parse::<usize>().unwrap(), e.0)
            })
            .sorted()
            .collect_vec();

        part2 += sorted.iter().enumerate()
            .map(|(i, e)| {
                let slot_no = i + 1;
                let focal_length = e.1;
                let power = box_no * slot_no * focal_length;
                if debug {
                    println!("{}: box={box_no} slot={slot_no} fl={focal_length} = {power}",
                             from_utf8(e.2).unwrap());
                }

                power
            })
            .sum::<usize>();
    }

    dbg!(part2);
}
