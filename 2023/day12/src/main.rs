use std::{cell::RefCell, collections::HashMap, io::BufRead};

use itertools::Itertools;
use neerajsi::*;

fn main() {
    let input_raw = read_stdin_input();
    let input = std::str::from_utf8(&input_raw).unwrap();
    let lines = input.lines();

    let map = lines.map(|l| {
        let (s, n) = l.split_once(' ').unwrap();
        let groups = n.split(',').map(|n| n.parse::<usize>().unwrap()).collect_vec();
        (s, groups)
    })
    .collect_vec();

    fn arragements<'a>(mut remaining_str: &'a [u8], remaining_conditions: &'a [usize], debug: bool, memo: &RefCell<HashMap<(&'a [u8], &'a [usize]), usize>>) -> usize {
        while let Some(stripped) = remaining_str.strip_prefix(&[b'.']) {
            remaining_str = stripped
        }

        if let Some(&res) = memo.borrow().get(&(remaining_str, remaining_conditions)) {
            return res;
        }

        if debug {
            println!("\t\tarragements: {} {remaining_conditions:?}", std::str::from_utf8(remaining_str).unwrap());
        }

        if remaining_conditions.is_empty() {
            if remaining_str.iter().any(|&c| c == b'#') {
                if debug {
                    println!(" -> 0 {}", line!());
                }
                return 0;
            }

            if debug {
                println!(" -> 1");
            }
            return 1;
        }

        let strip_question = if let Some(b'?') = remaining_str.get(0) {
            arragements(remaining_str.split_first().unwrap().1, remaining_conditions, debug, memo)
        } else {
            0
        };

        let (&conds_first, conds_rest) = remaining_conditions.split_first().unwrap();
        
        assert_ne!(conds_first, 0);

        if remaining_str.len() < conds_first { 
            if debug {
                println!(" -> {strip_question} {}", line!());
            }
            return strip_question;
        }

        if remaining_str.iter().take(conds_first).any(|&v| v == b'.') {

            if debug {
                println!(" -> {strip_question} {}", line!());
            }
            return strip_question;
        }


        let skip_count = 
            if let Some(&gap) = remaining_str.get(conds_first) {
                if gap == b'#' {
                    if debug { println!(" -> 0, {}", line!()); }
                    return strip_question;
                }

                conds_first + 1
            } else {
                conds_first
            };

        if debug {println!(" -> recurse1");}
        
        let res = arragements(remaining_str.split_at(skip_count).1, conds_rest, debug, memo) + strip_question;

        memo.borrow_mut().insert((remaining_str, remaining_conditions), res);
        return res;
    }

    let debug = false;
    let part1 = map.iter().map(|(l, conds)| {
        let memo = RefCell::new(HashMap::new());
        let arranges = arragements(l.as_bytes(), &conds, debug, &memo);
        println!("{l} {conds:?}: {arranges}");
        println!();

        let repeated = std::iter::repeat_n(l, 5).join("?");
        let repeated_conds = conds.repeat(5);

        let repeated_arranges = arragements(repeated.as_bytes(), &repeated_conds, debug, &memo);
        println!("{repeated} {repeated_conds:?}: {repeated_arranges}");
        println!();

        [arranges, repeated_arranges]
    })
    .sum_multiple();

    dbg!(part1);

}