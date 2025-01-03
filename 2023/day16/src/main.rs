use std::{cmp::max, collections::BTreeMap, mem, str::from_utf8, time::Instant};

use itertools::Itertools;
use neerajsi::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemType {
    Splitter,
    Mirror([CardinalDirectionName; 2]),
}

fn main() {
    let debug = std::env::args().nth(1).is_some();

    let input_raw = read_stdin_input();
    let input = std::str::from_utf8(&input_raw).unwrap();
    let lines = input.lines();

    let map = lines.map(|l| l.as_bytes()).collect_vec();

    let grid = Grid::from_map(&map);

    let mut row_objects = vec![BTreeMap::new(); grid.rows()];
    let mut col_objects = vec![BTreeMap::new(); grid.cols()];

    use CardinalDirectionName::*;
    use ItemType::*;

    for r in grid.row_range() {
        for c in grid.col_range() {
            let cell = map[r][c];
            match cell {
                b'|' => {
                    row_objects[r].insert(c, Splitter);
                }
                b'-' => {
                    col_objects[c].insert(r, Splitter);
                }
                b'\\' => {
                    row_objects[r].insert(c, Mirror([N, S]));
                    col_objects[c].insert(r, Mirror([W, E]));
                }

                b'/' => {
                    row_objects[r].insert(c, Mirror([S, N]));
                    col_objects[c].insert(r, Mirror([E, W]));
                }

                b'.' => {}

                _ => panic!("Unexpected cell at ({r},{c}) -> {}", cell as char),
            }
        }
    }

    let objects = [col_objects, row_objects];

    let timer = Instant::now();
    let visited_count = solve(&[0, 0], E, &map, &grid, &objects, debug);

    let elapsed = timer.elapsed();
    dbg!(elapsed);
    dbg!(visited_count);

    let timer = Instant::now();
    let mut max_energized = 0;
    let mut best_dir = None;
    for d in [E,S,W,N] {
        let (dimension, increasing) = map_dir(d);
        let mut start_loc = if increasing { [0, 0] } else { [grid.rows() - 1, grid.cols() - 1] };

        for i in 0..grid.dimension(dimension) {
            start_loc[1 - dimension] = i;
            let energized = solve(&start_loc, d, &map, &grid, &objects, debug);
            //println!("start_loc: {start_loc:?} {d:?}: {energized}");
            if energized > max_energized {
                max_energized = energized;
                best_dir = Some((d, i));
            }
        }
    }

    let elapsed2 = timer.elapsed();
    dbg!(elapsed2);

    dbg!(max_energized);
    dbg!(best_dir.unwrap());

    let correct_count = solve(&[0, 76], S, &map, &grid, &objects, false);
    dbg!(correct_count);
}

fn solve(start_loc: &Location, start_dir: CardinalDirectionName, map: &Vec<&[u8]>, grid: &Grid, objects: &[Vec<BTreeMap<usize, ItemType>>;2], debug: bool) -> usize {

    use ItemType::*;
    use CardinalDirectionName::*;

    #[derive(Debug, Clone, Default)]
    struct VisitedCell {
        visited_from: [bool; 4],
        first_visit: Option<CardinalDirectionName>,
    }
    
    let mut visited_set = grid.new_map(VisitedCell::default());
    
    let mut rays = Vec::new();
    
    let mark_visited = |loc: &Location, dir, visited_set: &mut Vec<Vec<VisitedCell>>| -> bool {
        let cell = &mut index2d_array!(visited_set, loc);
        cell.first_visit.get_or_insert(dir);
        mem::replace(&mut cell.visited_from[dir as usize], true)
    };
    
    mark_visited(&start_loc, start_dir, &mut visited_set);

    let object_rays = |loc: &Location, object, dimension:usize, increasing, rays: &mut Vec<_>| {
        const SPLITTER_OUT_DIRS: [[CardinalDirectionName; 2]; 2] = [[E, W], [N, S]];    
        match object {
            Splitter => {
                SPLITTER_OUT_DIRS[dimension].iter().for_each(|out_dir| {
                    rays.push((*loc, *out_dir));
                });
            }
            Mirror(trans) => {
                rays.push((*loc, trans[increasing as usize]));
            }
        }
    };
    
    {
        let (dimension, increasing) = map_dir(start_dir);
        if let Some(object) = objects[dimension][start_loc[1-dimension]].get(&start_loc[dimension]) {
            object_rays(start_loc, *object, dimension, increasing, &mut rays);
        } else {
            rays.push((*start_loc, start_dir));
        }
    }

    
    while let Some(ray) = rays.pop() {
        let (location, direction) = ray;
        let (dimension, increasing) = map_dir(direction);
    
        let const_idx = location[1 - dimension];
        let changing_idx = location[dimension];
        let objects = &objects[dimension][const_idx];
        let pp = if !increasing {
            objects.range(..changing_idx).last().ok_or(0)
        } else {
            objects
                .range((changing_idx + 1)..)
                .next()
                .ok_or(grid.dimension(dimension))
        };
    
        if debug {
            println!("moving {direction:?} from {location:?} object_loc = {pp:?}.")
        }
    
        let mut index = changing_idx;
        let new_idx = pp.map_or_else(|p| p, |p| *p.0);
    
        let mut new_loc = location;
        while index != new_idx {
            index = if increasing { index + 1 } else { index - 1 };
            if index == new_idx {
                break;
            }
    
            new_loc[dimension] = index;
            mark_visited(&new_loc, direction, &mut visited_set);
        }
    
        new_loc[dimension] = new_idx;
        if let Ok(pp) = pp {
            let visited_loc = &index2d_array!(visited_set, new_loc);
            if debug {
                println!("\tfound object at {new_loc:?}: {pp:?}");
            }
    
            if !visited_loc.visited_from[direction as usize] {
                object_rays(&new_loc, *pp.1, dimension, increasing, &mut rays);
            } else {
                if debug {
                    println!("\talready visited");
                }
            }
        }
    
        if grid.in_bounds(&new_loc) {
            mark_visited(&new_loc, direction, &mut visited_set);
        }
    }
    
    fn draw_visited(visited_set: &Vec<Vec<VisitedCell>>, map: &Vec<&[u8]>) {
        for (r, row) in visited_set.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                if map[r][c] != b'.' {
                    print!("{}", map[r][c] as char)
                } else {
                    if let Some(dir) = cell.first_visit {
                        print!("{}", "<>^v".chars().nth(dir as usize).unwrap());
                    } else {
                        print!(".")
                    }
                }
            }
            println!();
        }
    
        println!();
    }
    
    if debug {
        /*
        for r in map.iter() {
            println!("{}", from_utf8(r).unwrap());
        }
        println!();
        */
        println!("start_loc: {start_loc:?}, start_dir: {start_dir:?}");
        draw_visited(&visited_set, &map);
    }
    
    let visited_count = visited_set
        .iter()
        .positions2d(|c| c.first_visit.is_some())
        .count();
    visited_count
}

fn map_dir(direction: CardinalDirectionName) -> (usize, bool) {
    use CardinalDirectionName::*;
    match direction {
        N => (0, false),
        S => (0, true),
        W => (1, false),
        E => (1, true),
    }
}
