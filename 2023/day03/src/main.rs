use std::{cmp::min, collections::HashMap};

use arrayvec::ArrayVec;
use itertools::Itertools;
use neerajsi::*;
use regex::Regex;

fn main() {
    let input_raw = read_stdin_input();
    let input = std::str::from_utf8(&input_raw).unwrap();

    let lines = input.lines().collect_vec();

    let num_regex = Regex::new(r#"\d+"#).unwrap();

    let mut part_sum = 0;

    let mut gears: HashMap<[usize;2], ArrayVec<usize, 3>> = HashMap::new();
    for (r, l) in lines.iter().enumerate() {
        let r_start = r.saturating_sub(1);
        let r_end = min(r+2, lines.len());
        for m in num_regex.find_iter(l) {
            let c_start = m.start().saturating_sub(1);
            let c_end = min(m.end() + 1, l.len());
            let mut is_part = false;
            let mut gear_pos = None;
            for r in r_start..r_end {
                for c in c_start..c_end {
                    match lines[r].as_bytes()[c] {
                        b'0'..=b'9' | b'.' => {}
                        b'*' => { 
                            is_part = true;
                            gear_pos = Some([r, c]);
                        }
                        _ => { is_part = true }
                    }
                }
            }
        
            println!("Match at {r} {:?}: {is_part} {gear_pos:?}", m.range());

            if is_part {
                let part_no =  m.as_str().parse::<usize>().unwrap();
                if let Some(gear_pos) = gear_pos {
                    let _ = gears.entry(gear_pos).or_default().try_push(part_no);
                }
    
                part_sum += part_no;
            }
        }
    }

    dbg!(part_sum);

    let gear_scores = gears.values().map(|v| if v.len() == 2 { v[0] * v[1] } else { 0 }).sum::<usize>();

    dbg!(gear_scores);

}
