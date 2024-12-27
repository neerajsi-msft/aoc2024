use std::{cmp::{max, min}, collections::{hash_map, HashMap}};

use neerajsi::{read_stdin_input, SumMultiple};
use scan_fmt::scan_fmt;


fn main() {
    let input_raw = read_stdin_input();
    let input = std::str::from_utf8(&input_raw).unwrap();

    let p1max: HashMap<&str, usize> = HashMap::from([
        ("red", 12),
        ("green", 13),
        ("blue", 14)
    ]);

    let res = input.lines()
        .map(
            |l| {
                let (game, draws) = l.split_once(':').unwrap();
                let game_id = scan_fmt!(game, "Game {}", usize).unwrap();
                let mut possible = true;
                let mut min_req = HashMap::new();
                for draw in draws.split(';') {
                    let mut values: HashMap<String, usize> = HashMap::new();
                    for value in draw.split(',') {
                        let (count, color) = scan_fmt!(value, "{d} {}", usize, String).unwrap();

                        *values.entry(color).or_default() += count;
                    }

                    for (color, c) in values {
                        if p1max[color.as_str()] < c {
                            possible = false;
                        }

                        let mr = min_req.entry(color).or_default();

                        *mr = max(*mr, c);
                    }
                }

                let power = min_req.values().product();

                // dbg!(power);

                [if possible {
                    game_id
                 } else {
                    0
                 },
                 power]

            }
        ).sum_multiple();

    dbg!(res);

}
