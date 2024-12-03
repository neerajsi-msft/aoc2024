use core::str;
use std::{
    default, env, error::Error, fs::File, io::{BufRead, BufReader, Read}, mem, ops::Mul
};
use thiserror::Error;
use arrayvec::ArrayVec;

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

#[derive(Debug, Clone,Default)]
enum MulState {
    #[default]
    Start,
    M,
    U,
    L,
    LParen,
    FirstInt(ArrayVec<u8, 3>),
    Comma(u64),
    SecondInt(u64, ArrayVec<u8, 3>),
}

fn try_slice_to_int(s: &[u8]) -> Option<u64>
{
    str::from_utf8(s).ok().map(|s| s.parse::<u64>().ok()).flatten()
}

fn start_array_vec(ch: u8) -> ArrayVec<u8, 3>
{
    let mut v = ArrayVec::new();
    v.push(ch);
    v
}

fn get_operand(v: &ArrayVec<u8, 3>) -> Option<u64>
{
    try_slice_to_int(v.as_slice())
}

fn next_mul_state(state: &mut MulState, ch: u8) -> Option<u64>
{
    let mut value = None;

    *state = match (mem::take(state), ch) {
        (MulState::Start, b'm') => MulState::M,
        (MulState::M, b'u') => MulState::U,
        (MulState::U, b'l') => MulState::L,
        (MulState::L, b'(') => MulState::LParen,
        (MulState::LParen, b'0'..=b'9') => MulState::FirstInt(start_array_vec(ch)),
        (MulState::FirstInt(mut v), b'0'..=b'9') => 
            if v.try_push(ch).is_ok() {
                MulState::FirstInt(v)
            } else {
                MulState::Start
            },
        (MulState::FirstInt(v), b',') => {
            get_operand(&v).map_or(MulState::Start, |x| MulState::Comma(x))
        },
        (MulState::Comma(l), b'0'..=b'9') => {
            MulState::SecondInt(l, start_array_vec(ch))
        },
        (MulState::SecondInt(l, mut v), b'0'..=b'9') => {
            if (v.try_push(ch).is_ok()) {
                MulState::SecondInt(l, v)
            } else {
                MulState::Start
            }
        }
        (MulState::SecondInt(l, v ), b')') => {
            if let Some(r) = get_operand(&v) {
                value = Some(l as u64 * r as u64);
            }

            MulState::Start
        }
        _ => MulState::Start,
    };

    value
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    if args.len() != 2 {
        return Err(CommandLineError::new("Unexpected arg count.").into());
    }

    let file_name = args.nth(1).unwrap();

    println!("Opening file {}", file_name);

    let data = std::fs::read(file_name)?;

    let mut state = MulState::Start;

    let mut sum = 0u64;
    let mut ch_no = 0u32;
    let mut line_no = 1u32;
    let mut match_count = 0usize;
    let mut byte_no = 0usize;
    for &ch in &data {
        if ch == b'\n' {
            ch_no = 0;
            line_no += 1;
        }

        byte_no += 1;        
        ch_no += 1;
        if let Some(product) = next_mul_state(&mut state, ch) {
            sum += product;
            // println!("product at {byte_no} = {product}");
            match_count += 1;
        }
    }

    /*
    let sum : u32 = data.iter().enumerate().filter_map(
        |(index, ch)| {
            next_mul_state(& mut state, *ch)
        })
        .sum();
    */

    println!("part1: {sum}");
    println!("match_count: {match_count}");

    let mut sum = 0u64;
    let mut sum_enabled = 0u64;
    let mut match_count = 0usize;
    let re = regex::bytes::Regex::new(
        r#"(?x)
           (?<mul>mul\((?<op1>\d{1,3}),(?<op2>\d{1,3})\))|
           (?<do>do\(\))|
           (?<dont>don't\(\))"#)?;

    let mut enabled = true;
    for m in re.captures_iter(data.as_slice()) {
        if let Some(_) = m.name("mul") {
            let match2int = |m : Option<regex::bytes::Match> | -> Option<u64> { try_slice_to_int(m.unwrap().as_bytes()) };
    
            if let (Some(x), Some(y)) = (match2int(m.name("op1")), match2int(m.name("op2"))) {
                match_count += 1;
                let product = x*y;
                sum += product;
                if enabled {
                    sum_enabled += product;
                }

                //let pos = m.get(0).unwrap().end();
                // println!("product at {pos} = {product}");
            }
        } else if let Some(_) = m.name("do") {
            enabled = true;
        } else if let Some(_) = m.name("dont") {
            enabled = false;
        } else {
            panic!("Unexpected capture {:?}", m.get(0).unwrap());
        }
    }

    println!("Regex match_count:{match_count} sum:{sum} sum_enabled:{sum_enabled}");

    Ok(())
}