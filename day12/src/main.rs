use core::num;
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
use itertools::izip;
use itertools::Itertools;
use arrayvec::ArrayVec;

fn time_it<T>(name: &str, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();

    let ret = f();
    
    let elapsed = start.elapsed();
    println!("{name} took: {elapsed:?}");
    
    ret
}

const DIRECTION: [[isize;2]; 4] = [
    [0isize, -1isize], 
    [0isize, 1isize],
    [-1isize, 0isize],
    [1isize, 0isize]
];

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum DIRECTION_NAME {
    W = 0,
    E = 1,
    N = 2,
    S = 3,
}

type Location = [usize;2];

struct IncludeMissingDirections{}
struct DontIncludeMissingDirections{}

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
        while (self.current_dir < DIRECTION.len()) {
            let d = DIRECTION[self.current_dir];
            self.current_dir += 1;

            let new_loc: [Option<usize>;2] = std::array::from_fn(|a| self.location[a].checked_add_signed(d[a]));
                    
            if let [Some(r), Some(c)] = new_loc {
                if (r < self.rows) && (c < self.cols) {
                    return Some([r, c]);
                }
            }
        }

        None
    }
}

impl FusedIterator for DirectionIterator {}

struct DirectionIteratorWithMissingDirections(DirectionIterator);

impl Iterator for DirectionIteratorWithMissingDirections {
    type Item = Option<Location>;

    fn next(&mut self) -> Option<Self::Item> {
        while (self.0.current_dir < DIRECTION.len()) {
            let d = DIRECTION[self.0.current_dir];
            self.0.current_dir += 1;

            let new_loc: [Option<usize> ;2] = std::array::from_fn(|a| self.0.location[a].checked_add_signed(d[a]));
                    
            if let [Some(r), Some(c)] = new_loc {
                if (r < self.0.rows) && (c < self.0.cols) {
                    return Some(Some([r, c]));
                }
            }

            return Some(None);
        }

        None
    }
}

#[derive(Debug)]
struct Puzzle {
    map: Vec<Vec<u8>>,
    rows: usize,
    cols: usize
}

impl Puzzle {
    fn directions_at(&self, location: Location) -> DirectionIterator {
        DirectionIterator{location, rows: self.rows, cols: self.cols, ..Default::default()}
    }

    fn directions_at_with_missing(&self, location: Location) -> DirectionIteratorWithMissingDirections {
        DirectionIteratorWithMissingDirections{0: self.directions_at(location)}
    }

    fn cell_index(&self, location: Location) -> usize {
        assert!(location[0] < self.rows);
        assert!(location[1] < self.cols); 
        location[0] * self.cols + location[1]
    }

    fn value_at(&self, location: Location) -> u8 {
        self.map[location[0]][location[1]]
    }

    fn value_at_mut(&mut self, location: Location) -> &mut u8 {
        &mut self.map[location[0]][location[1]]
    }

    fn value_at_external<'a, TElement>(&self, array: &'a [TElement], location: Location) -> &'a TElement
        where TElement: Clone + Copy + Sized
    {
        &array[self.cell_index(location)]
    }

    fn value_at_external_mut<'a, TElement>(&self, array: &'a mut [TElement], location: Location) -> &'a mut TElement
    {
        &mut array[self.cell_index(location)]
    }

    fn cell_count(&self) -> usize {
        self.rows * self.cols
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct RegionInfo {
    id: char,
    area: usize,
    perimeter: usize,
    wall_count: usize,
}

#[derive(Debug, Clone, Copy, Default)]
struct CellInfo {
    region_id: usize,
    wall_at: [bool; 4],
}

fn solve_part1(
    puzzle: &Puzzle
) -> (usize, usize) {
    // let mut visited_set = BitSet::with_capacity(puzzle.cell_count());
    let cell_count = puzzle.cell_count();
    let mut regions: Vec<RegionInfo> = Vec::new();
    let mut bfs_queue = VecDeque::new();
    let mut cell_infos = vec!(None; cell_count);
    
    for r in 0..puzzle.rows {
        for c in 0..puzzle.cols {
            let loc = [r,c];

            if puzzle.value_at_external(&cell_infos, loc).is_some() {
                continue;
            }

            let id = puzzle.value_at(loc);
            let mut region_info = RegionInfo::default();
            region_info.id = id as char;
            let region_id = regions.len();
            
            bfs_queue.push_back(loc);
            while let Some(loc) = bfs_queue.pop_front() {
                assert_eq!(puzzle.value_at(loc), id);

                let cell_info = puzzle.value_at_external_mut(&mut cell_infos, loc);                

                if cell_info.is_some() { continue };
                
                region_info.area += 1;
                let mut wall_at: [bool; 4] = Default::default();
                for (index, neighbor) in puzzle.directions_at_with_missing(loc).enumerate() {
                    if neighbor.is_some_and(|l| puzzle.value_at(l) == id) {
                        bfs_queue.push_back(neighbor.unwrap());
                    } else {
                        region_info.perimeter += 1;
                        wall_at[index] = true;
                    }
                }


                *cell_info = Some(CellInfo{region_id, wall_at});
            }

            regions.push(region_info);
        }
    }

    #[derive(Debug)]
    struct WallFindState {
        dirs: [DIRECTION_NAME; 2],
        prev_region_id: Option<usize>,
        prev_walls: [bool; 2]
    }

    impl WallFindState {
        fn new(dirs: [DIRECTION_NAME; 2]) -> Self {
            WallFindState{dirs, prev_region_id: Default::default(), prev_walls: Default::default()}
        }

        fn reset(&mut self) {
            self.prev_region_id = None;
            self.prev_walls = Default::default();
        }
    }

    let extend_wall = |r, c, wall_finder: &mut WallFindState, regions: &mut [RegionInfo]| {
        let cell_info = puzzle.value_at_external(&cell_infos, [r,c]).unwrap();
        if wall_finder.prev_region_id.is_none_or(|id| id != cell_info.region_id) {
            wall_finder.reset();
            wall_finder.prev_region_id = Some(cell_info.region_id);
        }

        let region = &mut regions[cell_info.region_id];
        let new_walls = wall_finder.dirs.map(|dir| cell_info.wall_at[dir as usize]);
        for (old, new) in izip!(wall_finder.prev_walls, new_walls) {
            if !old && new {
                region.wall_count += 1;
            }
        }

        wall_finder.prev_walls = new_walls;
    };

    // Find all horizontal walls
    let mut wall_finder = WallFindState::new([DIRECTION_NAME::N, DIRECTION_NAME::S]);
    for r in 0..puzzle.rows {
        wall_finder.reset();
        for c in 0..puzzle.cols {
            extend_wall(r, c, &mut wall_finder, &mut regions);
        }
    }

    // Find all vertical walls
    let mut wall_finder = WallFindState::new([DIRECTION_NAME::W, DIRECTION_NAME::E]);
    for c in 0..puzzle.cols {
        wall_finder.reset();
        for r in 0..puzzle.rows {
            extend_wall(r, c, &mut wall_finder, &mut regions);
        }
    }

    /*
    let filtered_regions = regions.iter().enumerate()
        .filter_map(|(idx, r)| {
            if r.area != 0 {
                Some((r.id, r.area, r.perimeter)) 
            } else {
                None
            }
        }).collect::<Vec<_>>();

    println!("regions: {filtered_regions:?}");
    */

    regions.iter().map(|r| (r.area * r.perimeter, r.area * r.wall_count)).fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1))
}

fn solve_part2(
    puzzle: &Puzzle,
) -> u64 {

    0
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_name = env::args().nth(1).unwrap_or("input_sample.txt".into());

    let str = fs::read_to_string(file_name)?;
    let map: Vec<Vec<u8>> = str
        .trim_ascii()
        .lines()
        .map(|l| l.chars().map(|c| c as u8).collect())
        .collect();

    let rows = map.len();
    let cols = map[0].len();
    assert!(map.iter().all(|r| r.len() == cols));

    let puzzle = Puzzle{map, rows, cols};
    
    //let part1_6 = time_it("part1 (6)", || solve_part1(&puzzle, 6, true));
    //dbg!(part1_6);


    let (part1, part2) = time_it("part1", || solve_part1(&puzzle));
    dbg!(part1);

    dbg!(part2);
    Ok(())
}
