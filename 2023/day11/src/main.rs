use itertools::Itertools;
use neerajsi::*;

fn main() {
    let input_raw = read_stdin_input();
    let input = std::str::from_utf8(&input_raw).unwrap();
    let lines = input.lines();

    let map = lines.map(|l| {
        l.as_bytes()
    })
    .collect_vec();

    let grid = Grid::new(map.len(), map[0].len());

    let mut galaxies = Vec::new();
    let mut full_rows = vec![false; grid.rows()];
    let mut full_cols = vec![false; grid.cols()];
    for r in 0..grid.rows() {
        for c in 0..grid.cols() {
            let cell = map[r][c];
            match cell {
                b'.' => {},
                b'#' => {
                    full_rows[r] = true;
                    full_cols[c] = true;
                    galaxies.push([r,c])
                },
                _ => {
                    panic!("Unknown cell {cell}");
                }
            }
        }
    }

    let parts = galaxies.iter().tuple_combinations()
        .map(|(a, b)| {
            let mut dist = taxicab_distance(*a, *b);

            let mut row_range = [a[0], b[0]];
            row_range.sort();;

            let mut col_range = [a[1], b[1]];
            col_range.sort();

            let empties = full_rows[row_range[0]..row_range[1]].iter().filter(|r| !*r).count() +
                                 full_cols[col_range[0]..col_range[1]].iter().filter(|c| !*c).count(); 

            let dist1 = dist + empties;
            let dist2 = dist + (1000000 - 1)*empties;

            [dist1, dist2]
            
        })
        .sum_multiple();

    dbg!(parts);

}
