use clap_derive::Parser;
use clap::Parser;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::result::Result;
use thiserror::Error;
use itertools::Itertools;

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

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let str = fs::read_to_string(&args.input_file)?;

    let mut lines = str.lines();

    let patterns = lines.next().ok_or(PuzzleError::InputError("missing patterns".into()))?;

    assert_eq!(lines.next(), Some(""));

    let towels = patterns.split(", ").collect_vec();

    let patterns = format!("^({})+?$", towels.join("|"));
    let regex = Regex::new(&patterns)?;

    let haystacks = lines.collect_vec();

    let mut matches = 0;

    for check in &haystacks {
        let does_match = regex.is_match(check);
        if args.debug {
            println!("'{check}' -> {does_match}");
        }

        matches += does_match as usize;
    }

    dbg!(matches);

    let mut memo: HashMap<String, usize> = HashMap::new();
    let mut total_count = 0;

    for h in &haystacks {
        fn count_recursive(h: &str, towels: &[&str], memo: &mut HashMap<String, usize>) -> usize {
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

    dbg!(total_count);

    Ok(())
}
