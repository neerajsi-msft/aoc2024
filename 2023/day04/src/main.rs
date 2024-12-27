use neerajsi::*;
use itertools::Itertools;

fn main() {
    let input_raw = read_stdin_input();
    let input = std::str::from_utf8(&input_raw).unwrap();

    let match_counts = input.lines()
        .map(|l| {
            let items: [&str; 2] = l.split_once('|').unwrap().into();
            let [a, b] = items.map(str::split_ascii_whitespace);
            a.cartesian_product(b).filter(|(a, b)| a.eq(b)).count()
        })
        .collect_vec();

    let part1: usize = match_counts.iter().map(|&matches| if matches != 0 { 1usize << (matches - 1) } else { 0 }).sum();
    dbg!(part1);

    let cards_len = match_counts.len();
    let mut copy_counts = vec![1usize; cards_len];
    for i in 0..cards_len {
        let match_count = match_counts[i];
        let copy_count = copy_counts[i];
        for j in 0..match_count {
            copy_counts[i + j + 1] += copy_count;
        }
    }

    let part2: usize = copy_counts.iter().sum();

    dbg!(part2);
}
