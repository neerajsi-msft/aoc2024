use core::fmt;
use std::collections::VecDeque;
use std::fs;
use std::error::Error;
use std::num::Saturating;
use std::u64;
use itertools::Itertools;
use num_traits::SaturatingSub;
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

struct PositionIterator<I, J, F>
{
    data: I,
    predicate: F,
    current_iter: Option<J>,
    row: usize,
    col: usize,
}

impl<I: fmt::Debug, J: fmt::Debug, F> fmt::Debug for PositionIterator<I, J, F>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PositionIterator")
         .field("data", &self.data)
         .field("current_iter", &self.current_iter)
         .field("row", &self.row)
         .field("col", &self.col)
         .finish()
    }
}

impl<I, J, F> Iterator for PositionIterator<I, J, F>
where
    I: Iterator,
    I::Item: IntoIterator<IntoIter = J>,
    J: Iterator,
    F: Fn(J::Item) -> bool,
{
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut iter) = self.current_iter {
                while let Some(item) = iter.next() {
                    self.col += 1;
                    if (self.predicate)(item) {
                        return Some((self.row, self.col - 1));
                    }
                }
                self.current_iter = None;
                self.row += 1;
                self.col = 0;
            }

            match self.data.next() {
                Some(next_iterable) => {
                    self.current_iter = Some(next_iterable.into_iter());
                }
                None => return None,
            }
        }
    }
}

pub trait Iterable2d : Iterator
    where Self: Sized,
          Self::Item: IntoIterator
{
    fn positions2d<P>(self, predicate: P) -> PositionIterator<Self, <Self::Item as IntoIterator>::IntoIter, P>
        where P: Fn(<<Self as Iterator>::Item as IntoIterator>::Item) -> bool
    {
        PositionIterator{
            data: self,
            predicate,
            current_iter: None,
            row: 0,
            col: 0
        }
    }
}

impl<T> Iterable2d for T where T: Iterator<Item: IntoIterator> + Sized {}

const DIRECTION: [[i64;2]; 4] = [
    [0, -1], 
    [0, 1],
    [-1, 0],
    [1, 0]
];

#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
enum DirectionName {
    W = 0,
    E = 1,
    N = 2,
    S = 3,
}

const DIRECTION_COUNT: usize = 4;

fn direction_vector(direction: DirectionName) -> VectorType {
    to_vector2(&DIRECTION[direction as usize])
}

fn next_pos(pos: VectorType, direction: DirectionName) -> VectorType {
    pos + direction_vector(direction)
}

type Location = [usize;2];

#[derive(Debug, Default)]
struct DirectionIterator {
    location: Location,
    rows: usize,
    cols: usize,
    current_dir: usize,
}

impl Iterator for DirectionIterator {
    type Item = Location;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_dir < DIRECTION.len() {
            let d = DIRECTION[self.current_dir];
            self.current_dir += 1;

            let new_loc: [Option<usize>;2] = std::array::from_fn(|a| self.location[a].checked_add_signed(d[a] as isize));
                    
            if let [Some(r), Some(c)] = new_loc {
                if (r < self.rows) && (c < self.cols) {
                    return Some([r, c]);
                }
            }
        }

        None
    }
}

impl std::iter::FusedIterator for DirectionIterator {}


#[derive(Debug, Error)]
enum PuzzleError {
    #[error("Parsing error: {0}")]
    ParseError(String),
}

type VectorType = Vector2<i64>;

fn to_vector2<T>(val: &[T;2]) -> Vector2<T> 
    where T: Clone + Copy
{
    Vector2::new(val[0], val[1])
}

fn to_vector2_cast(val: &[usize;2]) -> Vector2<i64> 
{
    Vector2::new(val[0] as i64, val[1] as i64)
}

macro_rules! index {
    ($m:expr, $v:expr) => { ($m)[($v).x as usize][($v).y as usize] };
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromPrimitive)]
enum MapSlot {
    Start = b'S' as isize,
    End = b'E' as isize,
    Wall = b'#' as isize,
    Empty = b'.' as isize,
}

struct Puzzle {
    map: Vec<Vec<MapSlot>>,
    start: VectorType,
    end: VectorType,
    rows: usize,
    cols: usize,
}

fn draw_map(map: &[Vec<MapSlot>], pos: VectorType, cells: &[Vec<CellInfo>]) {
    for (x, r) in map.iter().enumerate() {
        for (y, &s) in r.iter().enumerate() {
            let loc = to_vector2_cast(&[x, y]);
            let c = 
                if pos == loc {
                    '@'
                } else {
                    match s {
                        MapSlot::Start | MapSlot::End | MapSlot::Wall => s as isize as u8 as char,
                        MapSlot::Empty => {
                            match index!(cells, loc).visited {
                                VisitState::New => ' ',
                                VisitState::Done { .. } => 'd',
                                VisitState::Started { .. } => '?',
                            }
                        }
                    }
                };
            print!("{c}");
        }

        println!();
    }
}


fn draw_path(puzzle: &Puzzle, cells: &[Vec<CellInfo>]) {
    let mut map: Vec<Vec<_>> = puzzle.map.iter()
        .map(|r| r.iter()
                .map(|s| char::from_u32(*s as isize as u32).unwrap()).collect()
            )
        .collect();

    let mut pos = puzzle.start;
    let mut reached_end = false;
    let mut direction = DirectionName::E;

    for _ in 0..1000 {
        if index!(map, pos) == 'E' {
            reached_end = true;
            break;
        }

        // Skip costing walls.
        if index!(map, pos) == '#' {
            continue;
        }

        let new_dir = cost_step(pos, cells, direction).1;
        let arrow = match new_dir {
            DirectionName::E => '>',
            DirectionName::S => 'v',
            DirectionName::W => '<',
            DirectionName::N => '^',
        };

        index!(map, pos) = arrow;
        (pos, direction) = (next_pos(pos, new_dir), new_dir);
    }

    for r in map {
        for c in r {
            print!("{c}");
        }

        println!();
    }

    if !reached_end {
        println!("Did not reach end!");
    }

}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum Cost {
    Known(u64),
    Infinite,
    Cycle,
    Wall
}

impl<> std::ops::Add<u64> for Cost {
    type Output = Self;
    
    fn add(self, rhs: u64) -> Self::Output {
        use Cost::*;
        match self {
            Known(v) => Known(v+rhs),
            Infinite | Cycle | Wall => self
        }   
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum VisitState {
    #[default]
    New,
    Started{min_costs: [Option<Cost>; DIRECTION_COUNT]},
    Done{min_costs: [Cost; DIRECTION_COUNT]}
}

#[derive(Debug, Default, Clone, Copy)]
struct CellInfo {
    visited: VisitState,
}

fn cost_step(pos: VectorType, cells: &[Vec<CellInfo>], from_dir: DirectionName) -> (Cost, DirectionName) {
    let VisitState::Done{min_costs} = index!(cells, pos).visited else { panic!("Cell at {pos:?} not visited!")};
    
    min_costs.iter().enumerate().map(|(d, &c)| {
        let c = if from_dir as usize == d {
            c
        } else if opposite_dir(from_dir) as usize == d {
            c + 1000 * 2
        } else {
            c + 1000
        };

        (c, DirectionName::from_usize(d).unwrap())
    }).min_by_key(|v| v.0).unwrap()
}

fn turns(direction: DirectionName) -> [DirectionName; 2]
{
    use DirectionName::*;

    match direction {
        N | S => [W, E],
        W | E => [N, S]
    }
}

fn opposite_dir(direction: DirectionName) -> DirectionName {
    use DirectionName::*;
    match direction {
        N => S,
        S => N,
        E => W,
        W => E,
    }
}

fn solve_part1_bfs(puzzle: &Puzzle, args: &Args) -> u64
{
    use DirectionName::*;
    use MapSlot::*;

    #[derive(Debug, Clone)]
    struct BfsCellInfo {
        costs: [Saturating<u64>; 4],
        in_queue: bool,
        on_shortest_path: [bool; 4],
    }

    impl Default for BfsCellInfo {
        fn default() -> Self {
            BfsCellInfo{costs:[Saturating(u64::MAX); 4], in_queue: false, on_shortest_path: [false;4]}
        }
    }

    let rows = puzzle.map.len();
    let cols = puzzle.map[0].len();

    let mut cells = vec![vec![BfsCellInfo::default(); cols]; rows];

    let mut bfs_queue: VecDeque<VectorType> = VecDeque::new();

    let end_pos = puzzle.end;

    index!(cells, end_pos).costs = [Saturating(0); 4];

    let push_neighbors = |puzzle: &Puzzle, pos: VectorType, bfs_queue: &mut VecDeque<VectorType>, cells: &mut [Vec<BfsCellInfo>]| {
        if pos == puzzle.start {
            return;
        }

        for d in [W,E,N,S] {
            let neighbor_pos = next_pos(pos, d);
            if index!(puzzle.map, neighbor_pos) == Wall {
                continue;
            }

            let source_dir = opposite_dir(d);
            let cost_from = index!(cells, pos).costs[source_dir as usize] + Saturating(1);

            // let's see if we can decrease any costs.

            let neighbor_cell = &mut index!(cells, neighbor_pos);
            let mut decreased = false;
            for (nd, cost) in neighbor_cell.costs.iter_mut().enumerate() {
                let nd = DirectionName::from_usize(nd).unwrap();
                let new_cost = 
                    if nd == source_dir {
                        cost_from
                    } else if nd == d {
                        cost_from + Saturating(2 * 1000)
                    } else {
                        assert!(turns(source_dir).contains(&nd));
                        cost_from + Saturating(1000)
                    };

                if new_cost < *cost {
                    decreased = true;
                    *cost = new_cost;
                }
            }

            if args.debug && decreased {
                println!("Visited {neighbor_pos:?} from {d:?}. new costs: {:?} queued:{:?}", neighbor_cell.costs, !neighbor_cell.in_queue);
            }
            
            if decreased && !neighbor_cell.in_queue {
                neighbor_cell.in_queue = true;
                bfs_queue.push_back(neighbor_pos);
            }
        }
    };

    push_neighbors(puzzle, end_pos, &mut bfs_queue, &mut cells);

    while let Some(pos) = bfs_queue.pop_front() {
        push_neighbors(puzzle, pos, &mut bfs_queue, &mut cells);
        index!(cells, pos).in_queue = false;
    }

    assert!(cells.iter().flatten().all(|c| !c.in_queue));

    dbg!(index!(cells, puzzle.start).costs);

    let cost_from_start =  index!(cells, puzzle.start).costs[W as usize].0;

    dbg!(cost_from_start);

    index!(cells, puzzle.start).on_shortest_path[W as usize] = true;
    index!(cells, puzzle.start).in_queue = true;
    index!(cells, puzzle.end).on_shortest_path = [true;4];

    // Now reconstruct the path and count the nodes.
    bfs_queue.push_back(puzzle.start);
    
    while let Some(pos) = bfs_queue.pop_back() {
        assert!(index!(cells, pos).on_shortest_path.iter().any(|x| *x));
        index!(cells, pos).in_queue = false;
        
        let incoming_shortest_paths = index!(cells, pos).on_shortest_path;
        for d1 in incoming_shortest_paths.iter().positions(|x| *x) {
            let cost_from = index!(cells,pos).costs[d1].0 - 1;
            assert_ne!(cost_from, u64::MAX - 1);

            let d1 = DirectionName::from_usize(d1).unwrap();

            if args.debug {
                println!("Reconstructing at {pos:?} to {d1:?}. Cost: {cost_from:?} Shortest paths: {incoming_shortest_paths:?}");
            }

            for d2 in [W,E,N,S] {
                let needed_cost = 
                    if d1 == d2 { cost_from }
                    else if d1 == opposite_dir(d2) { cost_from.saturating_sub(2 * 1000) }
                    else { cost_from.saturating_sub(1000) };
                
                let neighbor_pos = next_pos(pos, d2);
                let neighbor = &mut index!(cells, neighbor_pos);
                let neighbor_cost = neighbor.costs[d2 as usize].0;

                println!("\tneighbor({d2:?}) at {neighbor_pos:?}. Cost: {neighbor_cost:?} Needed: {needed_cost}");

                if neighbor_cost == needed_cost &&
                   !neighbor.on_shortest_path[d2 as usize] {

                    neighbor.on_shortest_path[d2 as usize] = true;
                    if !neighbor.in_queue {
                        neighbor.in_queue = true;
                        bfs_queue.push_back(neighbor_pos);
                    }                    
                }
            }

        }

    }

    fn is_on_shortest_path(c: &BfsCellInfo) -> bool { c.on_shortest_path.iter().any(|v| *v) }

    if args.debug {
        for r in 0..rows {
            for c in 0..cols {
                let ch = 
                    match puzzle.map[r][c] {
                        Start => 'S',
                        End => 'E',
                        Wall => '#',
                        Empty => {
                            if is_on_shortest_path(&cells[r][c]) { 'O' } else { ' ' }
                        }
                    };

                print!("{ch}");
            }
            println!();
        }
    }

    let path_cell_count = cells.iter().positions2d(is_on_shortest_path).count();

    dbg!(path_cell_count);

    cost_from_start
}

fn solve_part1(puzzle: &Puzzle, args: &Args) -> usize
{
    use DirectionName::*;

    let rows = puzzle.map.len();
    let cols = puzzle.map[0].len();

    let mut cells = vec![vec![CellInfo::default(); cols]; rows];

    /*
    // Let's try a recursive dynamic programming solution.
    fn cost_path_recursive(puzzle: &Puzzle, pos: VectorType, from_dir: DirectionName, cells: &mut Vec<Vec<CellInfo>>) -> Cost {
        if !(0..puzzle.rows as i64).contains(&pos.x) || !(0..puzzle.cols as i64).contains(&pos.y) {
            return Cost::Infinite;
        }

        if index!(cells, pos).visiting {
            return Cost::Infinite;
        }

        match index!(puzzle.map, pos) {
            MapSlot::Start | MapSlot::Empty => {},
            MapSlot::Wall => return Cost::Infinite,
            MapSlot::End => {
                assert!(pos == puzzle.end);
                return Cost::Known(0)
            }
        }
        
        if index!(cells, pos).min_cost.is_none() {
            index!(cells, pos).visiting = true;

            let mut costs = [Cost::Infinite; 4];

            for t in [N, S, E, W] {
                costs[t as usize] = cost_path_recursive(puzzle, next_pos(pos, t), t, cells) + 1;
            }

            index!(cells, pos).visiting = false;

            index!(cells, pos).min_cost = Some(costs);
        };

        cost_step(pos, &cells, from_dir).0
    }
    */

    let mut visit_stack: Vec<VectorType> = Vec::new();
    
    fn try_push_node (
        puzzle: &Puzzle,
        pos: VectorType,
        from_dir: DirectionName,
        visit_stack: &mut Vec<VectorType>,
        cells: &mut [Vec<CellInfo>]) -> Option<Cost> {

        if !(0..puzzle.rows as i64).contains(&pos.x) || !(0..puzzle.cols as i64).contains(&pos.y) {
            return Some(Cost::Wall);
        }

        match index!(cells, pos).visited {
            VisitState::Done{min_costs: _} => {
                Some(cost_step(pos, &cells, from_dir).0)
            }

            VisitState::Started{min_costs:_} => {
                return Some(Cost::Cycle)
            }

            VisitState::New => {
                match index!(puzzle.map, pos) {
                    MapSlot::Start | MapSlot::Empty => {
                        index!(cells, pos).visited = VisitState::Started{min_costs: [None; 4]};
                        visit_stack.push(pos);
                        None
                    },
                    MapSlot::Wall => {
                        Some(Cost::Wall)
                    },
                    MapSlot::End => {
                        assert!(pos == puzzle.end);
                        index!(cells, pos).visited = VisitState::Done{min_costs: [Cost::Known(0); 4]};
                        Some(Cost::Known(0))
                    }
                }
            }
        }
    }

    let root_pushed = try_push_node(puzzle, puzzle.start, DirectionName::E, &mut visit_stack, &mut cells);
    assert_eq!(root_pushed, None);

    while let Some(&node_pos) = visit_stack.last() {
        let VisitState::Started { mut min_costs } = index!(cells, node_pos).visited else {
            panic!("Unexpected node state at {node_pos:?}: {:?}", index!(cells, node_pos).visited);
        };

        
        let old_costs = min_costs;

        for t in [N, S, E, W] {
            if min_costs[t as usize].is_none() {
                min_costs[t as usize] = try_push_node(puzzle, next_pos(node_pos, t), t, &mut visit_stack, &mut cells);
            }            
        }

        if args.debug {
            println!("Node: {node_pos:?}. [WENS] Old Costs: {old_costs:?} New Costs: {min_costs:?}");
            draw_map(&puzzle.map, node_pos, &cells);
            println!();
        }

        if min_costs.iter().all(|c| c.is_some()) {
            let min_costs = min_costs.map(Option::unwrap);
            index!(cells, node_pos).visited = 
                if min_costs.iter().all(|&c| c == Cost::Cycle) {
                    // If this node is completely cyclical, reset it to New so it could
                    // be visited from another direction.
                    VisitState::New
                } else {
                    // Otherwise, mark the cyclical directions as infinite.
                    let min_costs = min_costs.map(|c| match c {Cost::Cycle => Cost::Infinite, _ => c});
                    VisitState::Done{min_costs}
                };

            visit_stack.pop();
        } else {
            index!(cells, node_pos).visited = VisitState::Started { min_costs };
        }
    }

    draw_path(puzzle, &cells);

    let cost = cost_step(puzzle.start, &cells, E).0;
    match cost {
        Cost::Infinite | Cost::Cycle | Cost::Wall => usize::MAX,
        Cost::Known(c) => c as usize,
    }

}

fn solve_part2(puzzle: &Puzzle, args: &Args) -> usize
{
    0
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

    let map = str.lines()
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

    let start= map.iter().positions2d(|v| *v == MapSlot::Start).exactly_one().expect("bad start");

    let end = map.iter().positions2d(|v| *v == MapSlot::End).exactly_one().expect("bad end");

    let start = start.into();
    let end = end.into();

    let rows = map.len();
    let cols = map[0].len();

    assert!(map.iter().all(|m| m.len() == cols));

    let puzzle = Puzzle{map, start: to_vector2_cast(&start), end: to_vector2_cast(&end), rows, cols};

    let part1 = solve_part1_bfs(&puzzle, &args);
    
    let part2 = solve_part2(&puzzle, &args);
    
    
    dbg!(part1);
    dbg!(part2);

    Ok(())
}
