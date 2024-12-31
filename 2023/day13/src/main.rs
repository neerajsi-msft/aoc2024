use std::{cmp::min, str::from_utf8};

use itertools::Itertools;
use neerajsi::*;

fn main() {
    let debug = std::env::args().nth(1).is_some();

    let input_raw = read_stdin_input();
    let input = std::str::from_utf8(&input_raw).unwrap();
    let mut lines = input.lines();

    let mut scores = 0;
    let mut smudge_scores = 0;
    loop {
        let pattern = lines.by_ref().take_while(|l| !l.is_empty()).map(|l| l.as_bytes()).collect_vec();
        if pattern.is_empty() {
            break;
        }

        let rows = pattern.len();
        let cols = pattern[0].len();
        assert!(rows != 0);
        assert!(cols != 0);
        assert!(pattern.iter().all(|r| r.len() == cols));

        if debug {
            for l in pattern.iter() {
                println!("{}", from_utf8(l).unwrap());
            }    
        }

        let mut row_defect_counts = vec![0; rows];
        let mut col_defect_counts = vec![0; cols];

        let mut mirror_row = None;
        let mut mirror_row_fixed = None;
        for r in 1..rows {
            let n_matches = min(r, rows - r);
            let rr = (r-n_matches)..r;
            let rr_rev = r..(r + n_matches);

            let defect_count = pattern[rr].iter().zip_eq(pattern[rr_rev].iter().rev()).map(
                |(a, b)| {
                    a.iter().zip_eq(b.iter()).filter(|(a, b)| **a != **b).count()
                }
            )
            .sum::<usize>();

            if debug {
                println!("\trow defects {r}: {defect_count}");
            }

            if defect_count == 0 {
                mirror_row = Some(r);
            }

            if defect_count == 1 {
                mirror_row_fixed = Some(r);
            }
        }

        let mut mirror_col = None;
        let mut mirror_col_fixed = None;
        for c in 1..cols {
            let n_matches = min(c, cols - c);

            // [1, 2, 3]
            //    

            let r = (c-n_matches)..c;
            let r_rev = c..(c + n_matches);
            let defect_count = pattern.iter().map(|row| {
                row[r.clone()].iter().zip_eq(row[r_rev.clone()].iter().rev()).filter(|(&a, &b)| a != b).count()
            })
            .sum::<usize>();

            if defect_count == 0 {
                mirror_col = Some(c);
            }

            if defect_count == 1 {
                mirror_col_fixed = Some(c);
            }

            if debug {
                println!("\tcol defects {c}: {defect_count}");
            }

        }

        fn calc_score(mirror_row :Option<usize>, mirror_col: Option<usize>) -> usize {
            if let Some(r) = mirror_row { 100 * r } else if let Some(c) = mirror_col {  c } else { 0 }
        }

        let cur_score = calc_score(mirror_row, mirror_col);
        let smudge_score = calc_score(mirror_row_fixed, mirror_col_fixed);

        if debug {
            println!("mirror: {mirror_row:?} {mirror_col:?} score: {cur_score}");
            println!("smudges: {mirror_row_fixed:?} {mirror_col_fixed:?} score: {smudge_score}");
            println!();
        }

        scores += cur_score;
        smudge_scores += smudge_score;
    }

    dbg!(scores);
    dbg!(smudge_scores);

}