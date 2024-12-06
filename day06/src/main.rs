use core::{str};
use std::{
    cell, env, error::Error, fs::File, io::{BufRead, BufReader, Read}
};
use thiserror::Error;
use petgraph::graphmap::{GraphMap, DiGraphMap};
use scan_fmt::scan_fmt;
use bit_set::BitSet;

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

#[derive(Debug, Clone, Error, PartialEq)]
enum PuzzleError {
    #[error("Unexpected character in puzzle at ({0},{1})")]
    UnexpectedCharacter(usize, usize),
    #[error("No guard found in puzzle")]
    NoGuard,
    #[error("A second guard was found at ({0},{1})")]
    MultipleGuards(usize, usize),
    #[error("The guard returned to the initial position and direction")]
    ReturnedToInitialPosition,
}

#[derive(Debug, Clone)]
struct Puzzle {
    rows: usize,
    cols: usize,

    guard_loc: (usize, usize),
    obstacle_set: BitSet,
}

impl Puzzle {

    fn cell_index(&self, pos: (usize, usize)) -> usize {
        assert!(pos.0 < self.rows);
        assert!(pos.1 < self.cols);

        return pos.0 * self.cols + pos.1;
    }

    fn is_obstacle(&self, pos: (usize, usize)) -> bool {
        self.obstacle_set.contains(self.cell_index(pos))
    }
}

fn parse_puzzle<'a>(lines: impl Iterator<Item = Result<impl AsRef<str> + 'a, impl Error + 'static>>) -> Result<Puzzle, Box<dyn Error>>
{
    let map = lines.map(|l| l.map(|s| s.as_ref().as_bytes().to_vec())).collect::<Result<Vec<_>, _>>()?;

    let rows = map.len();
    let cols = map[0].len();

    assert!(map.iter().all(|l| l.len() == rows));

    let cell_count = rows * cols;

    let mut obstacle_set = BitSet::with_capacity(cell_count);
    let mut guard_loc = None;

    for i in 0..rows {
        for j in 0..cols {
            match (map[i][j]) {
                b'.' => {},
                b'#' => {obstacle_set.insert(i * cols + j);}
                b'^' => { if guard_loc.is_none() { guard_loc = Some((i, j)); } else { Err(PuzzleError::MultipleGuards(i, j))? } }
                _ => { Err(PuzzleError::UnexpectedCharacter(i, j))? }
            }
        }
    }

    let Some(guard_loc) = guard_loc else { Err(PuzzleError::NoGuard)? };

    return Ok(Puzzle{rows, cols, guard_loc, obstacle_set});
}

// Note, these directions are carefully laid out, zero is up.
// Incrementing direction represents a right turn.
const DIRECTIONS: [(i8, i8);4] = [
    (-1, 0),    // Up
    (0, 1),     // Right
    (1, 0),     // Down
    (0, -1)     // Left
];

fn new_loc(puzzle: &Puzzle, cur_loc: &(usize, usize), direction: usize) -> Option<(usize, usize)>
{
    let direction = DIRECTIONS[direction];

    let new_loc = (cur_loc.0.wrapping_add_signed(direction.0.into()), cur_loc.1.wrapping_add_signed(direction.1.into()));

    if new_loc.0 >= puzzle.rows || new_loc.1 >= puzzle.cols {
        None
    } else {
        Some(new_loc)
    }
}


#[derive(Debug, Clone,PartialEq)]
enum IterationResult {
    Escaped,
    CallbackReturned,
    ReturnedToInitialPosition
}

fn IterateThroughPuzzle<'a> (puzzle: &Puzzle, f: &mut impl FnMut((usize, usize), usize) -> bool) -> IterationResult
{
    const INITIAL_DIRECTION: usize = 0usize;

    let mut cur_dir = INITIAL_DIRECTION;
    let mut cur_loc = puzzle.guard_loc;

    if !f(cur_loc, cur_dir) {
        return IterationResult::CallbackReturned;
    }

    loop {
        let Some(new_loc) = new_loc(puzzle, &cur_loc, cur_dir) else {break};

        if puzzle.is_obstacle(new_loc) {
            cur_dir += 1;
            cur_dir %= DIRECTIONS.len();
        } else {
            if !f(new_loc, cur_dir) {
                return IterationResult::CallbackReturned;
            }

            cur_loc = new_loc;

            if (cur_dir == INITIAL_DIRECTION) && (cur_loc == puzzle.guard_loc) {
                return IterationResult::ReturnedToInitialPosition;
            }
        }
    }

    IterationResult::Escaped
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    if args.len() > 2 {
        return Err(CommandLineError::new("Unexpected arg count.").into());
    }

    let file_name = args.nth(1).unwrap_or("input_sample.txt".into());


    println!("Opening file {}", file_name);

    let reader = BufReader::new(File::open(file_name)?);
    let lines = reader
        .lines();
    
    let puzzle = parse_puzzle(lines)?;

    let mut visited = BitSet::with_capacity(puzzle.obstacle_set.capacity());
    let mut mark_visited = |pos, _| {visited.insert(puzzle.cell_index(pos)); true};
    
    if IterateThroughPuzzle(&puzzle, & mut mark_visited) != IterationResult::Escaped {
        Err(PuzzleError::ReturnedToInitialPosition)?
    } 

    let visited_count = visited.len();

    dbg!(visited_count);

    let mut new_puzzle = puzzle.clone();
    let mut cycle_position_count = 0usize;
    
    const DIRECTION_COUNT: usize = 4;

    let mut visited_with_directions = BitSet::with_capacity(puzzle.rows * puzzle.cols * DIRECTION_COUNT);
    let start = std::time::Instant::now();
    for i in 0..puzzle.rows {
        for j in 0..puzzle.cols {
            let pos = (i, j);
            if pos == puzzle.guard_loc ||
               puzzle.is_obstacle(pos) {

                continue;   
            }

            // If the cell wasn't visited in the first place, it would not be visited
            // again, so an obstacle will do nothing
            if !visited.contains(puzzle.cell_index(pos)) {
                continue;
            }

            visited_with_directions.clear();

            // Keep going until we visit the same square facing the same direction
            // (i.e. until insert returns false, which means the item is already in the set.)
            let mut mark_visited_with_directions = |pos, dir| {
                visited_with_directions.insert(puzzle.cell_index(pos) * DIRECTION_COUNT + dir)
            };

            assert!(new_puzzle.obstacle_set.insert(puzzle.cell_index(pos)));

            let res = IterateThroughPuzzle(&new_puzzle, &mut mark_visited_with_directions);

            assert!(new_puzzle.obstacle_set.remove(puzzle.cell_index(pos)));

            if res != IterationResult::Escaped {
                cycle_position_count += 1;
            }
        }    
    }

    dbg!(start.elapsed().as_secs_f32());

    dbg!(cycle_position_count);

    Ok(())
}
