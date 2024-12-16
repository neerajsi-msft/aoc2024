use core::fmt;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::env;
use std::error;
use std::fs;
use std::error::Error;
use std::iter::Map;
use std::mem;
use itertools::Itertools;
use scan_fmt::scan_fmt;
use clap::Parser;
use thiserror::Error;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use nalgebra::Vector2;

#[cfg(test)]
mod tests{

    #[test]
    fn negative_modulus() {
        let x = -10;
        let y = 7;
        println!("{x} % {y} = {}", x % y);
    }

}


const DIRECTION: [[i64;2]; 4] = [
    [0, -1], 
    [0, 1],
    [-1, 0],
    [1, 0]
];

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum DirectionName {
    W = 0,
    E = 1,
    N = 2,
    S = 3,
}

fn direction_vector(direction: DirectionName) -> Vector2<i64> {
    to_vector2(&DIRECTION[direction as usize])
}

#[derive(Debug, Error)]
enum PuzzleError {
    #[error("Parsing error: {0}")]
    ParseError(String),
    #[error("Too many or too few robots")]
    RobotError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromPrimitive)]
enum MapSlot {
    Empty = b'.' as isize,
    Robot = b'@' as isize,
    Box = b'O' as isize,
    Wall = b'#' as isize,
    BoxL = b'[' as isize,
    BoxR = b']' as isize
}

struct Puzzle {
    map: Vec<Vec<MapSlot>>,
    directions: Vec<DirectionName>,
    robot_start: [i64; 2],
}

fn to_vector2<T>(val: &[T;2]) -> Vector2<T> 
    where T: Clone + Copy
{
    Vector2::new(val[0], val[1])
}

fn draw_map(map: &[Vec<MapSlot>]) {
    for r in map {
        for c in r {
            let c = *c as isize;
            let c = (c as u8) as char;
            print!("{c}");
        }

        println!();
    }
}

fn score(map: &[Vec<MapSlot>]) -> usize {

    let mut score = 0;
    for (r, m) in map.iter().enumerate() {
        for (c, s) in m.iter().enumerate() {
            if let MapSlot::Box | MapSlot::BoxL = *s {
                score += r * 100 + c;
            }
        }
    }
    score
}

macro_rules! index {
    ($m:expr, $v:expr) => { ($m)[($v).x as usize][($v).y as usize] };
}

fn solve_part1(puzzle: &Puzzle, args: &Args) -> usize
{
    let mut map = puzzle.map.clone();

    let mut robot_pos = to_vector2(&puzzle.robot_start);

    draw_map(&map);

    for &d in &puzzle.directions {
        let dv = direction_vector(d);

        assert!(index!(map, robot_pos) == MapSlot::Robot);

        let mut p = robot_pos;
        loop {
            p += dv;
            match index!(map, p) {
                MapSlot::Empty => {
                    // We can make a move
                    while p != robot_pos {

                        let prev = p - dv;
                        index!(map, p) = index!(map, prev);
                        p = prev;
                    }

                    index!(map, robot_pos) = MapSlot::Empty;
                    robot_pos += dv;
                    break;
                },

                MapSlot::Wall => {
                    // hit a wall no more to make
                    break;
                },

                MapSlot::Box => {continue},

                MapSlot::Robot => {
                    panic!("Multiple robots?");
                },

                MapSlot::BoxL | MapSlot::BoxR => {
                    panic!("Not expecting wide boxes in part1");
                }
            }
        }

        if args.debug {
            println!("MOVE: {d:?} {dv:?}");
            draw_map(&map);
        }
    }


    score(&map)
}

fn solve_part2(puzzle: &Puzzle, args: &Args) -> usize
{
    use MapSlot::*;
    use DirectionName::*;

    let mut map = puzzle.map.iter()
        .map(
            |r| r.iter().flat_map(
                |c| {
                    match c {
                        Empty => [Empty, Empty],
                        Box => [BoxL, BoxR],
                        Wall => [Wall, Wall],
                        Robot => [Robot, Empty],
                        BoxL | BoxR => panic!("Unexpected item {c:?}")
                    }
                }
            ).collect::<Vec<_>>()
        ).collect::<Vec<Vec<_>>>();

        
        let mut robot_pos = to_vector2(&puzzle.robot_start);
    robot_pos.y *= 2;

    draw_map(&map);
    dbg!(robot_pos);
    
    const ONE_OVER:Vector2<i64> = Vector2::new(0, 1);
    
    let mut move_list: Vec<Vector2<i64>> = Vec::new();
    let mut move_set = HashSet::new();
    for &d in &puzzle.directions {
        let dv = direction_vector(d);

        move_list.clear();


        let mut expand_point = 0;
        move_list.push(robot_pos);

        assert!(index!(map, robot_pos) == Robot);

        'a: while expand_point < move_list.len() {
            let p = move_list[expand_point];
            let np = p + dv;
            match index!(map, p) {
                Empty => {},
                Robot => {
                    assert!(expand_point == 0);
                    assert!(p == robot_pos);
                    move_list.push(np);
                },
                Wall => { break 'a; }
                BoxL => {
                    match d {
                        N|S => {
                            move_list.extend_from_slice(
                                &[np, np + ONE_OVER]
                            );
                        }
                        E|W => {
                            move_list.push(np);
                        }
                    }
                }
                BoxR => {
                    match d {
                        N|S => {
                            move_list.extend_from_slice(
                                &[np, np - ONE_OVER]
                            );
                        }
                        E|W => {
                            move_list.push(np);
                        }
                    }
                }

                Box => {
                    panic!("Unexpected box!");
                }
            }
            
            expand_point += 1;
        }

        if expand_point != move_list.len() {
            if args.debug {
                println!("{d:?} {dv:?} No move.");
                println!("\t{expand_point} {move_list:?}");
            }

            continue;
        }

        move_set.clear();
        move_list.retain_mut(|m| move_set.insert(*m) );

        for m in move_list.iter().skip(1).rev() {
            let op = m - dv;
            let t = index!(map, op);
            index!(map, op) = index!(map, m);
            index!(map, m) = t;
        }
        
        if args.debug {
            println!("MOVE: {d:?} {dv:?}");
            println!("\t{move_list:?}");
            draw_map(&map);
        }

        robot_pos = move_list[1];
    }
    
    score(&map)
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
    let map =  lines.by_ref()
        .take_while(|l| {
            !l.trim_ascii().is_empty()
        })
        .map(|l| {
            l.chars().map(
                |c| {
                    MapSlot::from_u32(c as u32).ok_or_else(|| PuzzleError::ParseError(format!("Unknown char {c}")))
                }
            )
            .collect::<Result<Vec<MapSlot>, PuzzleError>>()
        })
        .collect::<Result<Vec<Vec<MapSlot>>, PuzzleError>>()?;

    //dbg!(&map);

    let directions: Vec<DirectionName> = lines
        .flat_map(|l|
            l.chars().map(
                |c| -> Result<DirectionName, PuzzleError> {
                    Ok(match c {
                        '<' => DirectionName::W,
                        '>' => DirectionName::E,
                        '^' => DirectionName::N,
                        'v' => DirectionName::S,
                        _ => Err(PuzzleError::ParseError(format!("Unknown direction {c}")))?,
                    })
                }
            ))
            .collect::<Result<Vec<DirectionName>, PuzzleError>>()?;

    let mut find_robot = 
        map.iter().enumerate()
           .filter_map(|(r, row)|
                row.iter().position(|m| *m == MapSlot::Robot)
                          .map(|c| (r,c))
            );

    let r1 = find_robot.next();
    let r2 = find_robot.next();
    dbg!(r1);
    dbg!(r2);
    
    let (Some(robot), None) = (r1, r2) else {
        return Err(PuzzleError::RobotError.into());
    };

    let puzzle = Puzzle{map, directions, robot_start: [robot.0 as i64, robot.1 as i64]};

    let part1 = solve_part1(&puzzle, &args);
    
    let part2 = solve_part2(&puzzle, &args);
    
    
    dbg!(part1);
    dbg!(part2);

    Ok(())
}
