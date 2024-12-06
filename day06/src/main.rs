use bit_set::BitSet;
use core::str;
use petgraph::graphmap::{DiGraphMap, GraphMap};
use rayon::prelude::*;
use scan_fmt::scan_fmt;
use std::{
    cell::{self, RefCell}, env,
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Read},
};
use thiserror::Error;

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


// Note, these directions are carefully laid out, zero is up.
// Incrementing direction represents a right turn.
const DIRECTIONS: [(i8, i8); 4] = [
    (-1, 0), // Up
    (0, 1),  // Right
    (1, 0),  // Down
    (0, -1), // Left
];

fn change_direction(dir: usize) -> usize
{
    (dir + 1) % DIRECTIONS.len()
}

#[derive(Debug, Clone, Default)]
struct ObstacleMatrix {
    rows: Vec<Vec<usize>>,
    cols: Vec<Vec<usize>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Position {
    location: (usize, usize),
    direction: usize,
}

impl Position {
    fn new(location: (usize, usize), direction: usize) -> Self {
        Position{location, direction}
    }
}

impl ObstacleMatrix {
    fn new(rows: usize, cols: usize) -> Self {
        let mut row_vec = Vec::new();
        row_vec.resize_with(rows, Default::default);

        let mut col_vec = Vec::new();
        col_vec.resize_with(cols, Default::default);
        
        ObstacleMatrix{rows: row_vec, cols: col_vec}
    }

    fn add(&mut self, pos: (usize, usize)) {
        let (row, col) = (&mut self.rows[pos.0], &mut self.cols[pos.1]);

        let bad_insert = |dim: &[usize], val| {
            dim.last().map_or(false, |&v| v >= val)
        };

        if bad_insert(row, pos.1) ||
           bad_insert(col, pos.0) {

            panic!("Non sorted row insert at {pos:?}");
        }

        row.push(pos.1);
        col.push(pos.0);

        ()
    }

    fn next_pos(&self, pos: &Position) -> Option<Position> {
        let v = DIRECTIONS[pos.direction];

        let (search_dim, dim_val, dim_dir) = 
            if v.0 == 0i8 {
                (&self.rows[pos.location.0], pos.location.1, v.1 > 0)
            } else {
                (&self.cols[pos.location.1], pos.location.0, v.0 > 0)
            };

        let dim_pos = search_dim.binary_search(&dim_val).expect_err("Expected not to be exactly at a obstacle");

        assert!(search_dim.get(dim_pos).map_or(true, |&v| v > dim_val));

        let new_dim_val =
        if !dim_dir {
                if dim_pos == 0 {
                    return None;
                }
                
                search_dim[dim_pos - 1] + 1
            } else {
                *search_dim.get(dim_pos)? - 1
            };
            
        let new_dir = change_direction(pos.direction);
        if v.0 == 0 {
            Some(Position::new((pos.location.0, new_dim_val),new_dir))
        } else {
            Some(Position::new((new_dim_val, pos.location.1), new_dir))
        }
    }
}

fn override_next_pos(old_pos: &Position, new_pos: &Option<Position>, override_pos: &(usize, usize)) -> Option<Position>
{
    let v = DIRECTIONS[old_pos.direction];

    let get_updated_val = |old_val, new_val, override_val, dir| -> Option<usize> {
        match new_val {
            None => {
                match (old_val < override_val, dir > 0) {
                (true, true) => Some(override_val - 1),
                (false, false) => Some(override_val + 1),
                _ => None
                }
            }
            Some(new_val) => {
                if old_val < override_val && override_val <= new_val {
                    assert!(dir > 0);
                    Some(override_val - 1)
                } else if new_val <= override_val && override_val < old_val {
                    assert!(dir < 0);
                    Some(override_val + 1)
                } else {
                    Some(new_val)
                }
            }
        }
    };

    let new_direction = change_direction(old_pos.direction);
    if v.0 != 0 {
        if old_pos.location.1 != override_pos.1 {
            return *new_pos;
        }

        return Some(
            Position::new(
                (get_updated_val(old_pos.location.0, new_pos.map(|p| p.location.0), override_pos.0, v.0)?,
                          override_pos.1),
                          new_direction
            )
        );

    } else {
        if (old_pos.location.0 != override_pos.0) {
            return *new_pos;
        }

        return Some(
            Position::new(
                (override_pos.0,
                          get_updated_val(old_pos.location.1, new_pos.map(|p| p.location.1), override_pos.1, v.1)?),
                          new_direction
                )
                        
            
            );
    }
}

#[derive(Debug, Clone)]
struct Puzzle {
    rows: usize,
    cols: usize,

    guard_loc: (usize, usize),
    obstacle_set: BitSet,
    obstacle_matrix: ObstacleMatrix,
}

impl Puzzle {
    fn cell_index(&self, pos: &(usize, usize)) -> usize {
        assert!(pos.0 < self.rows);
        assert!(pos.1 < self.cols);

        return pos.0 * self.cols + pos.1;
    }

    fn is_obstacle(&self, pos: &(usize, usize)) -> bool {
        self.obstacle_set.contains(self.cell_index(pos))
    }
}

fn parse_puzzle<'a>(
    lines: impl Iterator<Item = Result<impl AsRef<str> + 'a, impl Error + 'static>>,
) -> Result<Puzzle, Box<dyn Error>> {
    let map = lines
        .map(|l| l.map(|s| s.as_ref().as_bytes().to_vec()))
        .collect::<Result<Vec<_>, _>>()?;

    let rows = map.len();
    let cols = map[0].len();

    assert!(map.iter().all(|l| l.len() == rows));

    let cell_count = rows * cols;

    let mut obstacle_set = BitSet::with_capacity(cell_count);
    let mut guard_loc = None;
    let mut obstacle_matrix = ObstacleMatrix::new(rows, cols);

    for i in 0..rows {
        for j in 0..cols {
            match (map[i][j]) {
                b'.' => {}
                b'#' => {
                    obstacle_set.insert(i * cols + j);
                    obstacle_matrix.add((i, j));
                }
                b'^' => {
                    if guard_loc.is_none() {
                        guard_loc = Some((i, j));
                    } else {
                        Err(PuzzleError::MultipleGuards(i, j))?
                    }
                }
                _ => Err(PuzzleError::UnexpectedCharacter(i, j))?,
            }
        }
    }

    let Some(guard_loc) = guard_loc else {
        Err(PuzzleError::NoGuard)?
    };

    return Ok(Puzzle {
        rows,
        cols,
        guard_loc,
        obstacle_set,
        obstacle_matrix,
    });
}


fn new_loc(puzzle: &Puzzle, cur_loc: &(usize, usize), direction: usize) -> Option<(usize, usize)> {
    let direction = DIRECTIONS[direction];

    let new_loc = (
        cur_loc.0.wrapping_add_signed(direction.0.into()),
        cur_loc.1.wrapping_add_signed(direction.1.into()),
    );

    if new_loc.0 >= puzzle.rows || new_loc.1 >= puzzle.cols {
        None
    } else {
        Some(new_loc)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum IterationResult {
    Escaped,
    CallbackReturned,
    ReturnedToInitialPosition,
}

fn IterateThroughPuzzle<'a>(
    puzzle: &Puzzle,
    f: &mut impl FnMut((usize, usize), usize) -> bool,
) -> IterationResult {
    const INITIAL_DIRECTION: usize = 0usize;

    let mut cur_dir = INITIAL_DIRECTION;
    let mut cur_loc = puzzle.guard_loc;

    if !f(cur_loc, cur_dir) {
        return IterationResult::CallbackReturned;
    }

    loop {
        let Some(new_loc) = new_loc(puzzle, &cur_loc, cur_dir) else {
            break;
        };

        if puzzle.is_obstacle(&new_loc) {
            cur_dir = change_direction(cur_dir);
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

fn part2_can_place_obstacle(
    puzzle: &Puzzle,
    new_obstacle: &(usize, usize),
    visited: &BitSet
) -> bool {
    if *new_obstacle == puzzle.guard_loc || puzzle.is_obstacle(new_obstacle) {
        return false;
    }

    // If the cell wasn't visited in the first place, it would not be visited
    // again, so an obstacle will do nothing
    if !visited.contains(puzzle.cell_index(new_obstacle)) {
        return false;
    }
    true
}

fn part2_checkone(
    puzzle: &Puzzle,
    new_obstacle: &(usize, usize),
    visited: &BitSet,
    new_puzzle: &mut Puzzle,
    visited_with_directions: &mut BitSet,
) -> bool {
    if !part2_can_place_obstacle(puzzle, new_obstacle, visited) {
        return false;
    }

    visited_with_directions.clear();
    let mut last_dir = 1usize;

    //println!("Check:{:?}", new_obstacle);

    // Keep going until we visit the same square facing the same direction
    // (i.e. until insert returns false, which means the item is already in the set.)
    let mut mark_visited_with_directions =
        |pos, dir| {
            if dir != last_dir {
                //println!("\tturn:{:?}", (pos, dir));
                last_dir = dir;
            }
            visited_with_directions.insert(puzzle.cell_index(&pos) * DIRECTIONS.len() + dir)
        };

    assert!(new_puzzle.obstacle_set.insert(puzzle.cell_index(new_obstacle)));

    let res = IterateThroughPuzzle(&new_puzzle, &mut mark_visited_with_directions);

    assert!(new_puzzle.obstacle_set.remove(puzzle.cell_index(new_obstacle)));

    //println!("\tresult:{res:?}");

    res != IterationResult::Escaped
}

fn part2_checkone_jumping(
    puzzle: &Puzzle,
    new_obstacle: &(usize, usize),
    visited: &BitSet,
    visited_with_directions: &mut BitSet,    
    ) -> bool
{
    if !part2_can_place_obstacle(puzzle, new_obstacle, visited) {
        return false;
    }

    visited_with_directions.clear();

    //println!("Check:{:?}", new_obstacle);

    let mut cur_pos = Position::new(puzzle.guard_loc, 0);
    loop {
        //println!("\tturn:{cur_pos:?}");
        let new_pos = puzzle.obstacle_matrix.next_pos(&cur_pos);
        let new_pos = override_next_pos(&cur_pos, &new_pos, new_obstacle);
        let Some(new_pos) = new_pos else {
            //println!("\tescaped");
            return false;
        };

        if !visited_with_directions.insert(puzzle.cell_index(&new_pos.location) * DIRECTIONS.len() + new_pos.direction) {
            //println!("\tlooped");
            return true;
        }

        assert_ne!(new_pos, cur_pos);
        cur_pos = new_pos;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    if args.len() > 2 {
        return Err(CommandLineError::new("Unexpected arg count.").into());
    }

    let file_name = args.nth(1).unwrap_or("input_sample.txt".into());

    println!("Opening file {}", file_name);

    let reader = BufReader::new(File::open(file_name)?);
    let lines = reader.lines();

    let puzzle = parse_puzzle(lines)?;

    let mut visited = BitSet::with_capacity(puzzle.obstacle_set.capacity());
    let mut mark_visited = |pos, _| {
        visited.insert(puzzle.cell_index(&pos));
        true
    };

    if IterateThroughPuzzle(&puzzle, &mut mark_visited) != IterationResult::Escaped {
        Err(PuzzleError::ReturnedToInitialPosition)?
    }

    let visited_count = visited.len();

    dbg!(visited_count);

    let mut new_puzzle = puzzle.clone();
    let mut cycle_position_count = 0usize;

    let mut visited_with_directions =
        BitSet::with_capacity(puzzle.rows * puzzle.cols * DIRECTIONS.len());

    let mut serial_timer = stopwatch::Stopwatch::start_new();
    for i in 0..puzzle.rows {
        for j in 0..puzzle.cols {
            let pos = (i, j);
            cycle_position_count += part2_checkone(
                &puzzle,
                &pos,
                &visited,
                &mut new_puzzle,
                &mut visited_with_directions,
            ) as usize;
        }
    }

    serial_timer.stop();
    
    dbg!(cycle_position_count);
    
    let mut parallel_timer = stopwatch::Stopwatch::start_new();

    let par_cycle_count: usize = (0..puzzle.rows).into_par_iter().map(
        |i| -> usize {
            #[derive(Debug)]
            struct TlsData {
                new_puzzle: Puzzle,
                visited_with_directions: BitSet
            }

            #[derive(Debug)]
            enum TlsState {
                Uninitialized,
                Initialized(TlsData)
            }

            thread_local! {
                static TLS_STATE: RefCell<TlsState> = RefCell::new(TlsState::Uninitialized)
            }

            let mut cycle_count = 0usize;
            for j in 0..puzzle.cols {
                let pos = (i, j);

                TLS_STATE.with_borrow_mut(
                    |state| {
                        if matches!(state, TlsState::Uninitialized) {
                            let tls_data = TlsData{new_puzzle: puzzle.clone(),
                                 visited_with_directions:  BitSet::with_capacity(puzzle.rows * puzzle.cols * DIRECTIONS.len())};

                            *state = TlsState::Initialized(tls_data);
                        }

                        let TlsState::Initialized(data) = state else {panic!("Unexpected tls state")};

                        cycle_count += part2_checkone(&puzzle, &pos, &visited, &mut data.new_puzzle, &mut data.visited_with_directions) as usize;
                    }
                )
            }

            cycle_count
        }
    ).sum();

    parallel_timer.stop();
    
    dbg!(par_cycle_count);
    
    let mut jumping_timer = stopwatch::Stopwatch::start_new();
    let mut jumping_cycle_count = 0usize;
    for i in 0..puzzle.rows {
        for j in 0..puzzle.cols {
            let pos = (i, j);
            jumping_cycle_count += part2_checkone_jumping(
                &puzzle,
                &pos,
                &visited,
                &mut visited_with_directions,
            ) as usize;
        }
    }
    jumping_timer.stop();

    dbg!(jumping_cycle_count);

    let serial_time = serial_timer.elapsed().as_micros();
    let parallel_time = parallel_timer.elapsed().as_micros();
    let jumping_time = jumping_timer.elapsed().as_micros();
    dbg!(serial_time);
    dbg!(parallel_time);
    dbg!(serial_time / parallel_time);
    dbg!(jumping_time);

    Ok(())
}
