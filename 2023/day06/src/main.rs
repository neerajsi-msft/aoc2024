use neerajsi::*;
use itertools::Itertools;

fn main() {
    let input_raw = read_stdin_input();
    let input = std::str::from_utf8(&input_raw).unwrap();
    let mut lines = input.lines();
    
    let times = lines.next().unwrap();
    let dists = lines.next().unwrap();

    let times = times.strip_prefix("Time: ").unwrap();
    let dists = dists.strip_prefix("Distance: ").unwrap();

    let races = times.split_ascii_whitespace().zip(dists.split_ascii_whitespace())
        .map(|(t, d)| (t.parse::<u64>().unwrap(), d.parse::<u64>().unwrap()))
        .collect_vec();

    // race distance = (tt - ht) * ht = tt*ht - ht^2
    // winning tt*ht - ht^2 > td
    // equal = (tt^2 - sqrt(4td)) / 2

    fn is_win(ht: u64, total_time: u64, total_dist: u64) -> bool {
        (total_time - ht) * ht > total_dist
    }

    let part1: u64 =
        races.iter()
            .map(|&(total_time, total_dist)| {
                let mut win_ways = 0;
                for ht in 0..total_time {
                    if is_win(ht, total_time, total_dist) {
                        win_ways += 1
                    }
                }

                win_ways
            })
            .product();
    
    dbg!(part1);

    let [time, dist] = races.iter().fold(
        [String::new(), String::new()],
        |strs, &r| {
            let r: [u64;2] = r.into();
            std::array::from_fn(|i| format!("{}{}", strs[i], r[i]))
        }
    )
    .map(|s| s.parse::<u64>().unwrap());

    println!("time: {time} dist: {dist}");

    let timef = time as f64;
    let distf = dist as f64;

    // ht^2 - tt*ht + td = 0

    let quad = (timef * timef - 4.0*distf).sqrt() / 2.0;
    let vals = [timef / 2.0 + quad, timef / 2.0 - quad];

    dbg!(vals);

    let vals_int = vals.map(|v| v.floor() as u64);
    let wins = vals_int.iter().flat_map(|v| {
        ((v-5)..(v+5)).filter(|&v| {
            let is_win = is_win(v, time, dist);
            println!("{v}: {is_win}");
            is_win
        })
    }).minmax().into_option().unwrap();

    dbg!(wins);
    dbg!(wins.1 - wins.0 + 1);

}
