use std::array;
use std::cell::Cell;
use std::cmp::min;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::error::Error;
use std::env;
use std::fs;
use std::iter::FusedIterator;
use std::ops::Index;
use std::path;
use std::slice::SliceIndex;
use std::thread::current;
use std::time::Instant;
use bit_set::BitSet;
use derive_more::derive::Neg;
use derive_more::derive::Rem;
use itertools::izip;
use itertools::Itertools;
use arrayvec::ArrayVec;
use justerror::Error;
use num::traits::NumAssign;
use ::num::FromPrimitive;
use ::num::Rational64;
use ::num::Zero;
use ::num::One;
use scan_fmt::scan_fmt;
use nalgebra::*;

fn time_it<T>(name: &str, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();

    let ret = f();
    
    let elapsed = start.elapsed();
    println!("{name} took: {elapsed:?}");
    
    ret
}

#[Error]
enum PuzzleError {
    InputTooShort,
    ParseError{line_no: usize}
}

#[derive(Debug, Clone)]
struct Equation {
    terms: [u64; 3],
}

#[derive(Debug)]
struct EquationSystem {
    equations: [Equation; 2]
}

#[derive(Debug)]
struct Puzzle {
    equation_systems: Vec<EquationSystem>
}

impl Puzzle {
}

/*
#[derive(::derive_more::Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy,
         Add, Sub, Div, Neg,
         AddAssign, SubAssign, DivAssign, MulAssign,
         From)]
struct RationalField(Rational64);

impl Mul for RationalField {
    type Output = RationalField;
    fn mul(self, rhs: Self) -> Self::Output {
        RationalField(self.0.mul(rhs.0))
    }
}

impl Zero for RationalField {
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    fn set_zero(&mut self) {
        self.0.set_zero();
    }

    fn zero() -> Self {
        RationalField(Rational64::zero())
    }
}

impl One for RationalField {
    fn is_one(&self) -> bool {
        self.0.is_one()
    }

    fn one() -> Self {
        RationalField(Rational64::one())
    }

    fn set_one(&mut self) {
        self.0.set_one();
    }
}
*/

type RationalField = Rational64;

struct DisplayArray<'a>(&'a [[Rational64;3]]);

impl <'a> std::fmt::Display for DisplayArray<'a> {
    
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first_row = true;
        for r in self.0 {
            if !first_row {writeln!(f)?;};
            first_row = false;

            let mut first = true;
            for c in r {
                if !first {
                    write!(f, ", ")?;
                }
                first = false;
                write!(f, "{}", *c)?;
            }

        }

        Ok(())
    }
}

fn get_solutions(equation_systems: &[EquationSystem], part2: bool) -> u64
{
    let mut tokens = 0u64;
    for (index, system) in equation_systems.iter().enumerate() {
        let system = system.equations.clone();
        let mut system = system.map(|s| s.terms.map(|v| Rational64::from_u64(v).unwrap()));

        if part2 {
            for mut row in &mut system {
                row[2] += 10000000000000;
            }
        }

        if system[0][0].is_zero() {
            system.swap(0, 1);
        }

        if !system[0][0].is_zero() && !system[1][0].is_zero() {
            // Ax + By = C   ==>  Subtract r2 -= D/A r1
            // Dx + Ey = F

            let scale = system[1][0] / system[0][0];
            system[1] = std::array::from_fn(|i| system[1][i] - scale * system[0][i] );
            if !system[1][1].is_zero() {
                system[1] = system[1].map(|v| v / system[1][1]);
            }
        }

        // println!("rref: {}", DisplayArray(&system));

        // attempt back substitute.
        print!("equation {index}: ");
        if system[1][1].is_zero() {
            if system[1][2].is_zero() {
                println!("Underdetermined: {}", DisplayArray(&system[0..1]));
            } else {
                println!("No solution.");
            }
        } else {
            let y = system[1][2] / system[1][1];
            let x = system[0][2] - system[0][1] * y;
            let x = if !system[0][0].is_zero() { x / system[0][0] } else { Zero::zero() };
            println!("x:{x} y:{y}");

            if x.is_integer() &&
               y.is_integer() &&
               (x >= 0.into()) &&
               (y >= 0.into()) {

                if !part2 &&
                   (x <= 100.into()) &&
                   (y <= 100.into()) {

                    continue;
                }
 

                tokens += (x.to_integer() * 3 + y.to_integer()) as u64;
            }
        }
    }
    
    tokens
}

fn solve_part1(
    puzzle: &Puzzle
) -> u64
{
    get_solutions(&puzzle.equation_systems, false)
}

fn solve_part2(
    puzzle: &Puzzle,
) -> u64 {
    get_solutions(&puzzle.equation_systems, true)

}

fn main() -> Result<(), Box<dyn Error>> {
    let file_name = env::args().nth(1).unwrap_or("input_sample.txt".into());

    let str = fs::read_to_string(file_name)?;
    let mut equation_systems: Vec<EquationSystem> = Vec::new();
    
    let chunks = str
        .trim_ascii()
        .lines()
        .enumerate()
        .chunks(4);


    for mut chunk in chunks.into_iter() {
        let line1 = chunk.next().ok_or(PuzzleError::InputTooShort)?;
        let line2 = chunk.next().ok_or(PuzzleError::InputTooShort)?;
        let line3 = chunk.next().ok_or(PuzzleError::InputTooShort)?;
        let vars: [[u64;2];3] = [scan_fmt!(line1.1, "Button A: X+{}, Y+{}", u64, u64).map_err(|_| PuzzleError::ParseError { line_no: line1.0 })?.into(),
                                 scan_fmt!(line2.1, "Button B: X+{}, Y+{}", u64, u64).map_err(|_| PuzzleError::ParseError { line_no: line2.0 })?.into(),
                                 scan_fmt!(line3.1, "Prize: X={}, Y={})", u64, u64).map_err( |_| PuzzleError::ParseError { line_no: line3.0 })?.into()];

        let equations: [Equation; 2] = array::from_fn(|i| Equation{terms: [vars[0][i], vars[1][i], vars[2][i]]});
        equation_systems.push(EquationSystem{equations});
    }



    let puzzle = Puzzle{equation_systems};

    //let part1_6 = time_it("part1 (6)", || solve_part1(&puzzle, 6, true));
    //dbg!(part1_6);


    let part1 = time_it("part1", || solve_part1(&puzzle));
    dbg!(part1);
    
    let part2 = time_it("part2", || solve_part2(&puzzle));
    dbg!(part2);
    Ok(())
}
