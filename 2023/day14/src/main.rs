use core::hash;
use std::{collections::HashMap, hash::{DefaultHasher, Hash, Hasher}};

use itertools::Itertools;
use neerajsi::*;

fn main() {
    let debug = std::env::args().nth(1).is_some();

    let input_raw = read_stdin_input();
    let input = std::str::from_utf8(&input_raw).unwrap();
    let lines = input.lines();

    let map = lines.map(|l| l.as_bytes()).collect_vec();

    let grid = Grid::new(map.len(), map[0].len());

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    enum ItemType{
        Rock,
        Barrier
    }

    let mut rocks_and_barriers = vec![Vec::new(); grid.cols()];
    for r in 0..grid.rows() {
        for c in 0..grid.cols() {
            let cell = map[r][c];
            match cell {
                b'.' => {}
                b'#' => {rocks_and_barriers[c].push((ItemType::Barrier, r));}
                b'O' => {rocks_and_barriers[c].push((ItemType::Rock, r));}
                _ => panic!("Unknown type {cell}")
            }
        }
    }

    fn do_tilt_up(rocks_and_barriers: &mut Vec<Vec<(ItemType, usize)>>) {
        for column in rocks_and_barriers {
            let mut cur_row = 0;
            for (ty, row) in column {
                let new_row =
                match ty {
                    ItemType::Barrier => {
                        *row
                    }

                    ItemType::Rock => {
                        cur_row
                    }
                };

                cur_row = new_row + 1;
                *row = new_row;
            }
        }
    }

    fn do_tilt_down(rocks_and_barriers: &mut Vec<Vec<(ItemType, usize)>>, row_count: usize) {
        for column in rocks_and_barriers {
            let mut cur_row = row_count - 1;
            for (ty, row) in column.iter_mut().rev() {
                let new_row =
                match ty {
                    ItemType::Barrier => {
                        *row
                    }

                    ItemType::Rock => {
                        cur_row
                    }
                };

                cur_row = new_row - 1;
                *row = new_row;
            }

            assert!(column.is_sorted_by_key(|(_, r)| *r));
            assert!(column.iter().tuple_windows().all(|(a, b)| a.1 != b.1));
        }
    }

    fn do_score(rocks_and_barriers: &Vec<Vec<(ItemType, usize)>>, grid: &Grid) -> usize {
        rocks_and_barriers.iter().map(|c| {
            c.iter().filter_map(|&(ty, i)| {
                match ty {
                    ItemType::Rock => {
                        Some(grid.rows() - i)
                    },
    
                    ItemType::Barrier => { 
                        None
                    }
                }
            })
            .sum::<usize>()
        })
        .sum()
    } 

    do_tilt_up(&mut rocks_and_barriers);


    let part1: usize = do_score(&rocks_and_barriers, &grid);
    dbg!(part1);

    let spin_cycle_count = 1_000_000_000;

    fn do_transpose(rocks_and_barriers: &Vec<Vec<(ItemType, usize)>>, mut new_vec: Vec<Vec<(ItemType, usize)>>) -> Vec<Vec<(ItemType, usize)>> {
        for r in new_vec.iter_mut() {
            r.clear();
        }

        for r in 0..rocks_and_barriers.len() {
            for &(ty, c) in rocks_and_barriers[r].iter() {
                new_vec[c].push((ty, r));
            }
        }

        new_vec
    }

    fn print_rocks_and_barriers(rocks_and_barriers: &Vec<Vec<(ItemType, usize)>>, grid: &Grid) {
        assert!(rocks_and_barriers.iter().all(|c| c.is_sorted_by_key(|(_ty, i)| i)));

        for r in grid.row_range() {
            for c in grid.col_range() {
                if let Ok(i) = rocks_and_barriers[c].binary_search_by_key(&r, |(_ty, i)| *i) {
                    match rocks_and_barriers[c][i].0 {
                        ItemType::Barrier => print!("#"),
                        ItemType::Rock => print!("O"),
                    }
                } else {
                    print!(".");
                }
            }
            println!();
        }

        println!();
    }

    let mut transposed = vec![Vec::new(); grid.rows()];
    let mut hash_map = HashMap::new();
    let mut hash_exists_count = 0;
    for i in 1..=spin_cycle_count {
        do_tilt_up(&mut rocks_and_barriers);
        if debug {
            if i == 0 {
                print_rocks_and_barriers(&rocks_and_barriers, &grid);
            }
        }
        transposed = do_transpose(&rocks_and_barriers, transposed);
        do_tilt_up(&mut transposed);
        rocks_and_barriers = do_transpose(&transposed, rocks_and_barriers);

        if debug {
            if i == 0 {
                print_rocks_and_barriers(&rocks_and_barriers, &grid);
            }
        }

        do_tilt_down(&mut rocks_and_barriers, grid.rows());

        if debug {
            if i == 0 {
                print_rocks_and_barriers(&rocks_and_barriers, &grid);
            }
        }

        transposed = do_transpose(&rocks_and_barriers, transposed);
        do_tilt_down(&mut transposed, grid.cols());
        rocks_and_barriers = do_transpose(&transposed, rocks_and_barriers);

        if debug {
            if i == 0 {
                print_rocks_and_barriers(&rocks_and_barriers, &grid);
            }
        }

        let score = do_score(&rocks_and_barriers, &grid);
        let entry = if let Some(entry) = hash_map.get_mut(&rocks_and_barriers) {
            hash_exists_count += 1;
            entry
        } else {
            hash_exists_count = 0;
            hash_map.entry(rocks_and_barriers.clone()).or_insert((score, Vec::new()))
        };

        assert_eq!(score, entry.0);
        entry.1.push(i);

        if hash_exists_count == 1000 {
            break;
        }
    }

    let value_counts = hash_map.values().map(|v| (v.1[0], v.0, &v.1)).sorted().collect_vec();
    value_counts.iter().for_each(|e| println!("{} {} {:?}", e.0, e.1, e.2));

    let part2 = do_score(&rocks_and_barriers, &grid);
    dbg!(part2);

}