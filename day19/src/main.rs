use clap_derive::Parser;
use clap::Parser;
use regex::Regex;
use std::collections::HashSet;
use std::{collections::HashMap, ops::Deref};
use std::error::Error;
use std::fs;
use std::result::Result;
use thiserror::Error;
use itertools::Itertools;
use neerajsi::{time_it, time_it_buffered, TimingBuffer};

#[derive(Debug, Error)]
enum PuzzleError {
    #[error("Input error {0}")]
    InputError(String)
}

#[derive(Parser, Debug)]
#[command(about)]
/// Simulate robots moving around a toroidal field.
struct Args {
    #[arg(default_value = "input_sample.txt")]
    input_file: String,

    #[arg(short, long, default_value_t = false)]
    debug: bool,
}

fn part1(towels: &[&str], haystacks: &[&str], args: &Args, timing: &mut TimingBuffer) -> usize
{

    let mut matches = 0;
    
    let regex = time_it_buffered(timing, "part1 regex build", || {
         let patterns = format!("^({})+?$", towels.join("|"));
         Regex::new(&patterns).unwrap()    
    });
    

    for check in haystacks {
        let does_match = regex.is_match(check);
        if args.debug {
            println!("'{check}' -> {does_match}");
        }

        matches += does_match as usize;
    }

    matches
}

fn part2(towels: &[&str], haystacks: &[&str], args: &Args) -> u64
{
    let mut memo: HashMap<String, u64> = HashMap::new();
    let mut total_count = 0;

    for h in haystacks {
        fn count_recursive(h: &str, towels: &[&str], memo: &mut HashMap<String, u64>) -> u64 {
            if let Some(&count) = memo.get(h) {
                return count;
            }

            if h.is_empty() {
                return 1;
            }

            let mut count = 0;
            for t in towels {
                if let Some(suffix) = h.strip_prefix(t) {
                    count += count_recursive(suffix, towels, memo);
                }
            }

            memo.insert(h.into(), count);

            count
        }

        let count = count_recursive(h, &towels, &mut memo);
        if args.debug {
            println!("{count}: {h}");
        }

        total_count += count;
    }

    total_count
}

fn char_to_alpha(ch: char) -> u8 {
    match ch {
        'b' => 0,
        'g' => 1,
        'r' => 2,
        'u' => 3,
        'w' => 4,
        _ => panic!("unexpected char: {ch}")
    }
}

const ALPHABET_SIZE: usize = 5;

#[derive(Debug, Default)]
struct StateTableElement {
    out_transitions: [Option<u16>; ALPHABET_SIZE],
    is_accepting_state: bool,
    has_transitions: bool,
}

fn part2_automata(towels: &[&str], haystacks: &[&str], args: &Args, timing: &mut TimingBuffer) -> u64 {

    let state_table = time_it_buffered(timing, "build automata", ||{
        let mut state_table = Vec::new();
        state_table.push(StateTableElement::default());
        state_table[0].is_accepting_state = true;
        
        for t in towels {
            let mut cur_element_idx = 0;
            for c in t.chars().map(char_to_alpha) {
                let new_idx = state_table.len();
                let out_transition = &mut state_table[cur_element_idx].out_transitions[c as usize];
                cur_element_idx =
                    if let Some(cur_idx) = *out_transition {
                        assert!(state_table[cur_element_idx].has_transitions);
                        cur_idx as usize
                    } else {
                        *out_transition = Some(new_idx as u16);
                        state_table[cur_element_idx].has_transitions = true;
                        state_table.push(StateTableElement::default());
                        new_idx
                    };
            }

            assert_ne!(cur_element_idx, 0);
            state_table[cur_element_idx].is_accepting_state = true;
        }

        assert!(state_table.iter().all(|s| s.has_transitions || s.is_accepting_state));

        state_table
    });

    let mut total_count = 0;
    let mut active_states = HashMap::with_capacity(state_table.len());
    let mut new_active_states= active_states.clone();
    for h in haystacks {
        active_states.insert(0u16, 1usize);
        for c in h.chars().map(char_to_alpha) {
            for (&active_state, &count) in &active_states {
                if let Some(new_state_idx) = state_table[active_state as usize].out_transitions[c as usize] {
                    let new_state = &state_table[new_state_idx as usize];
                    if new_state.is_accepting_state {
                        *new_active_states.entry(0).or_default() += count;
                    }

                    if new_state.has_transitions {
                        *new_active_states.entry(new_state_idx).or_default() += count;
                    }
                }
            }

            active_states.clear();
            std::mem::swap(&mut new_active_states, &mut active_states);
        }

        let count: usize = active_states.get(&0).map(|v| *v).unwrap_or_default();
        active_states.clear();

        if args.debug {
            println!("{count}: {h}");
        }

        total_count += count as u64;
    }

    total_count
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let str = fs::read_to_string(&args.input_file)?;

    let mut lines = str.lines();

    let patterns = lines.next().ok_or(PuzzleError::InputError("missing patterns".into()))?;

    assert_eq!(lines.next(), Some(""));

    let towels = patterns.split(", ").collect_vec();

    let haystacks = lines.collect_vec();

    let mut timings = TimingBuffer::new();

    let part1 = time_it("part1 (regex)", || part1(&towels, &haystacks, &args, &mut timings));
    
    timings.dump();
    
    dbg!(part1);



    let part2 = time_it("part2 (naive)", || part2(&towels, &haystacks, &args));

    dbg!(part2);

    let part2_automata = time_it("part2-auto", || part2_automata(&towels, &haystacks, &args, &mut timings));

    dbg!(part2_automata);
    Ok(())
}
