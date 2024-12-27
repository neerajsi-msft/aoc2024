use std::cmp::min;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::error::Error;
use std::env;
use std::fs;
use std::ops::Index;
use std::path;
use std::slice::SliceIndex;
use std::thread::current;
use std::time::Instant;
use bit_set::BitSet;
use itertools::Itertools;

fn time_it<T>(name: &str, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();

    let ret = f();
    
    let elapsed = start.elapsed();
    println!("{name} took: {elapsed:?}");
    
    ret
}

const DIRECTION: [[isize;2]; 4] = [
    [-1isize, 0isize],
    [0isize, -1isize],
    [0isize, 1isize],
    [1isize, 0isize]
];

struct Puzzle {
    map: Vec<Vec<u8>>,
    rows: usize,
    cols: usize
}

#[derive(Debug, Default)]
struct DirectionIterator {
    location: [usize;2],
    rows: usize,
    cols: usize,
    current_dir: usize,
}

impl Iterator for DirectionIterator {
    type Item = [usize;2];

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

impl Puzzle {
    fn directions_at(&self, location: [usize;2]) -> DirectionIterator {
        DirectionIterator{location, rows: self.rows, cols: self.cols, ..Default::default()}
    }

    fn cell_index(&self, location: [usize; 2]) -> usize {
        assert!(location[0] < self.rows);
        assert!(location[1] < self.cols); 
        location[0] * self.cols + location[1]
    }

    fn value_at(&self, location: [usize;2]) -> u8 {
        self.map[location[0]][location[1]]
    }
}

fn compute_score(
    puzzle: &Puzzle,
    score_map: &[Vec<usize>]
    ) -> usize
{
    let mut score = 0;
    for r in 0..puzzle.rows {
        for c in 0..puzzle.cols {
            let loc = [r,c];
            
            if puzzle.value_at(loc) == 0 {
                score += score_map[loc[0]][loc[1]];
            }
        }
    }

    score
}

fn solve_part1_recurse (
    puzzle: &Puzzle,
    location: [usize; 2],
    visited: &mut BitSet,
    reachable_count: &mut [Vec<usize>]
    )
{

    let val = puzzle.value_at(location);
    if val == 0 {
        return;
    }

    puzzle.directions_at(location)
        .for_each(|l| {
            let loc_val = puzzle.value_at(l);
            if loc_val + 1 == val {
                if visited.insert(puzzle.cell_index(l)) {
                    reachable_count[l[0]][l[1]] += 1;
                    solve_part1_recurse(puzzle, l, visited, reachable_count);
                }
            }
        });
}

fn solve_part1(
    puzzle: &Puzzle
) -> usize {
    let mut reachable_count: Vec<Vec<usize>> = vec![vec![0;puzzle.cols];puzzle.rows];
    let mut visited = BitSet::with_capacity(puzzle.rows * puzzle.cols);
    for r in 0..puzzle.rows {
        for c in 0..puzzle.cols {
            let loc = [r,c];
            if puzzle.map[loc[0]][loc[1]] == 9 {
                visited.clear();
                solve_part1_recurse(puzzle, loc, &mut visited, &mut reachable_count);
            }
        }
    }

    compute_score(puzzle, &reachable_count)
}

fn solve_part2(
    puzzle: &Puzzle
) -> usize {
    let mut bfs_queue: VecDeque<[usize;2]> = VecDeque::new();
    let mut path_count = vec![vec![0usize; puzzle.cols]; puzzle.rows];
    let mut visited = BitSet::with_capacity(puzzle.rows * puzzle.cols);

    // seed the bfs queue with all the '9's.
    for r in 0..puzzle.rows {
        for c in 0..puzzle.cols {
            if (puzzle.value_at([r,c]) == 9) {
                bfs_queue.push_back([r,c]);
                path_count[r][c] = 1;
            }
        }
    }

    while let Some(location) = bfs_queue.pop_front() {
        let val = puzzle.map[location[0]][location[1]];
        let cur_path_count = path_count[location[0]][location[1]];
        puzzle.directions_at(location)
            .for_each(|l| {
                let loc_val = puzzle.value_at(l);
                if loc_val + 1 == val {
                    path_count[l[0]][l[1]] += cur_path_count;
                    if visited.insert(puzzle.cell_index(l)) {
                        bfs_queue.push_back(l);
                    }
                }
            });

        /*
        println!("location: {location:?}");
        for r in 0..puzzle.rows {
            for c in 0..puzzle.cols {
                print!("{},", path_count[r][c]);
            }

            println!();
        }
        */
    }

    compute_score(puzzle, &path_count)
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_name = env::args().nth(1).unwrap_or("input_sample.txt".into());

    let str = fs::read_to_string(file_name)?;
    let map: Vec<Vec<u8>> = str
        .trim_ascii()
        .lines()
        .map(|l| l.chars().map(|c| 
            match c {
                '0'..='9' => c as u8 - '0' as u8,
                _ => 10u8
            }).collect())
        .collect();

    let rows = map.len();
    let cols = map[0].len();
    assert!(map.iter().all(|r| r.len() == cols));

    let puzzle = Puzzle{map, rows, cols};

    let part1 = time_it("part1", || solve_part1(&puzzle));
    dbg!(part1);

    let part2 = time_it("part2", || solve_part2(&puzzle));
    dbg!(part2);
    Ok(())
}
