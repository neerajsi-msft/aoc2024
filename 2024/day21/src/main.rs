use std::{
    array::{self}, cell::RefCell, collections::HashMap, fmt::{Display, Formatter}, fs, mem, sync::LazyLock, time::Instant
};

use arrayvec::ArrayVec;
use clap::Parser;
use itertools::Itertools;
use memoize::memoize;

#[derive(Parser, Debug)]
#[command(about)]
/// Simulate robots moving around a toroidal field.
struct Args {
    #[arg(default_value = "input_sample.txt")]
    input_file: String,

    #[arg(short, long, default_value_t = false)]
    debug: bool,

    #[arg(short='p', long, default_value_t = 2)]
    dpad_depth: usize,
}

const NUMERIC_KEY_PAD: [&str; 4] = ["789", "456", "123", " 0A"];

const D_PAD: [&str; 2] = [" ^A", "<v>"];

fn find_char_pos(strs: &[&str], ch: char) -> [usize; 2] {
    for (r, row) in strs.iter().enumerate() {
        for (c, found_ch) in row.chars().enumerate() {
            if ch == found_ch {
                return [r, c];
            }
        }
    }

    panic!("Missing character {ch}");
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
enum DPad {
    A = 0,
    UP = 1,
    DOWN = 2,
    LEFT = 3,
    RIGHT = 4,
}

const fn dpad_to_char(dpad: DPad) -> char {
    use DPad::*;

    match dpad {
        A => 'A',
        UP => '^',
        DOWN => 'v',
        LEFT => '<',
        RIGHT => '>',
    }
}

impl Display for DPad {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", dpad_to_char(*self))
    }
}

#[derive(Debug, Clone, Copy)]
enum Move {
    MyMove(DPad),
    ParentMove(usize, DPad)
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Move::MyMove(m) => write!(f, "{}", m),
            Move::ParentMove(_l, m) => write!(f, "[{}]", m),
        }
    }
}

const DPAD_MAP: [[isize; 2]; 6] = [[0, 2], [0, 1], [1, 1], [1, 0], [1, 2], [0, 0]];

const DIMENSION_TO_DIRECTIONS: [[DPad; 2]; 2] = [[DPad::UP, DPad::DOWN], [DPad::LEFT, DPad::RIGHT]];

type MemoMap = Vec<HashMap<Vec<DPad>, usize>>;

fn dpad_moves(
    in_moves: &[DPad],
    dpads_left: usize,
    out_moves: Option<&RefCell<Vec<Move>>>,
    memoized: &mut MemoMap,
) -> usize {
    use DPad::*;

    if let Some(&v) = memoized[dpads_left].get(in_moves) {
        return v;
    }

    let mut move_count = 0;
    let mut pos = DPAD_MAP[A as usize];
    for &m in in_moves {
        if dpads_left == 0 {
            if let Some(out_moves) = out_moves {
                out_moves.borrow_mut().push(Move::ParentMove(dpads_left, m));   
            }
        }

        let new_pos = DPAD_MAP[m as usize];
        
        let delta: [isize; 2] = array::from_fn(|i| new_pos[i] - pos[i]);
        
        
        let dimension_orders = [[1,0], [0,1]];
        
        let mut child_move_counts = [None, None];
        let mut saved_out_move_start = [0, 0];
        'a: for (i, dimension_order) in dimension_orders.iter().enumerate() {
            let mut temp_moves: ArrayVec<_, 8> = ArrayVec::new();
            if let Some(out_moves) = out_moves {
                saved_out_move_start[i] = out_moves.borrow().len();
            }
            
            let mut dbg_pos = pos;
            for &d in dimension_order {
                let steps = delta[d].abs();
                let direction = DIMENSION_TO_DIRECTIONS[d][(delta[d] >= 0) as usize];
                for _i in 0..steps {
                    temp_moves.push(direction);
                }
    
                dbg_pos[d] = pos[d] + delta[d];
                if D_PAD[dbg_pos[0] as usize].as_bytes()[dbg_pos[1] as usize] == b' ' {
                    continue 'a;
                }
            }
    
            temp_moves.push(DPad::A);
            if dpads_left != 0 {
                child_move_counts[i] = Some(dpad_moves(&temp_moves, dpads_left - 1, out_moves, memoized));
                
            } else {
                if let Some(out_moves) = out_moves {
                    out_moves.borrow_mut().extend(temp_moves.iter().map(|m| Move::MyMove(*m)));
                }

                child_move_counts[i] = Some(temp_moves.len());
            }
        }

        move_count += match child_move_counts {
            [Some(a), Some(b)] => {
                if b < a {
                    if let Some(out_moves) = out_moves  {
                        out_moves.borrow_mut().drain(saved_out_move_start[0]..saved_out_move_start[1]);
                    }

                    b
                } else {
                    if let Some(out_moves) = out_moves  {
                        out_moves.borrow_mut().truncate(saved_out_move_start[1]);
                    }

                    a
                }
            }

            _ => {
                child_move_counts[0].or(child_move_counts[1]).unwrap()
            }
        };

        pos = new_pos;
    }

    // only memoize if not debugging
    if out_moves.is_none() {
        memoized[dpads_left].insert(in_moves.to_vec(), move_count);
    }

    move_count
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let start = Instant::now();

    let str = fs::read_to_string(&args.input_file)?;
    let codes: Vec<Vec<u8>> = str
        .lines()
        .map(|l| {
            l.chars()
                .map(|c| {
                    let c = c as u8;
                    match c {
                        b'0'..=b'9' => c - b'0',
                        b'A' => 0xA,
                        _ => panic!("Unexpected char {c}"),
                    }
                })
                .collect()
        })
        .collect();

    let pos_map = (0..=0xA)
        .map(|n| {
            let ch = if n <= 9 { b'0' + n } else { b'A' + n - 0xA } as char;
            find_char_pos(&NUMERIC_KEY_PAD, ch).map(|a| a as isize)
        })
        .collect_vec();

    println!("init: {:?}", start.elapsed());

    let mut part1 = 0;

    let mut memoized = MemoMap::new();
    memoized.resize(args.dpad_depth, HashMap::new());

    for code in codes {
        
        let debug_moves = RefCell::new(Vec::new());

        let debug_moves_opt = if args.debug { Some(&debug_moves) } else { None };

        let mut move_list: Vec<DPad> = Vec::new();
        let mut pos = pos_map[0xa];
        let mut move_count = 0;
        for &c in &code {
            let new_pos = pos_map[c as usize];

            let delta: [isize; 2] = array::from_fn(|i| new_pos[i] - pos[i]);

            let dimension_orders = [[1, 0], [0, 1]];

            let mut move_lists: Option<(usize, ArrayVec<_, 8>, usize)> = None;
            let mut saved_out_moves = [0, 0];
            'a: for (i, &dimension_order) in dimension_orders.iter().enumerate() {
                let mut temp_move_list: ArrayVec<_, 8> = ArrayVec::new();
                let mut dbg_pos = pos;

                saved_out_moves[i] = debug_moves.borrow().len();

                for d in dimension_order {
                    let steps = delta[d].abs();
                    let direction = DIMENSION_TO_DIRECTIONS[d][(delta[d] >= 0) as usize];
                    for _i in 0..steps {
                        temp_move_list.push(direction);
                    }
    
                    dbg_pos[d] = pos[d] + delta[d];
                    if NUMERIC_KEY_PAD[dbg_pos[0] as usize].as_bytes()[dbg_pos[1] as usize] == b' ' {
                        continue 'a;
                    }
                }
    
                temp_move_list.push(DPad::A);

                let temp_cost = dpad_moves(&temp_move_list, args.dpad_depth - 1, debug_moves_opt, &mut memoized);

                if move_lists.as_ref().is_none_or(|l| l.0 > temp_cost) {
                    move_lists = Some((temp_cost, temp_move_list, i));
                }
            }

            let move_lists = move_lists.unwrap();
            move_count += move_lists.0;
            move_list.extend_from_slice(&move_lists.1);
            if move_lists.2 == 1 {
                debug_moves.borrow_mut().drain(saved_out_moves[0]..saved_out_moves[1]);
            } else if saved_out_moves[0] != saved_out_moves[1] {
                debug_moves.borrow_mut().truncate(saved_out_moves[1]);
            }

            pos = new_pos;
        }
        
        let numeric_value = code.iter().fold(0usize, |acc, &n| if n < 10 { acc * 10 + n as usize } else { acc });

        part1 += move_count * numeric_value;

        if args.debug {
            let debug_moves = debug_moves.take();
            assert_eq!(debug_moves.iter().filter(|m| matches!(m, Move::MyMove(_))).count(), move_count);

            println!("\t{}", debug_moves.iter().format(""));

            let inter_moves = RefCell::new(Vec::new());
            dpad_moves(&move_list, 0, Some(&inter_moves), &mut memoized);
            let inter_moves = inter_moves.take();

            println!(
                "\t({}) {}",
                inter_moves.len(),
                inter_moves.iter().format("")
            );

            println!("\t({}) {}", move_list.len(), move_list.iter().format(""));

        }

        println!("{code:?}: {move_count} {numeric_value}");
    }

    dbg!(part1);

    Ok(())
}
