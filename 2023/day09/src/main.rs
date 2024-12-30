use itertools::Itertools;
use neerajsi::*;

fn main() {
    let input_raw = read_stdin_input();
    let input = std::str::from_utf8(&input_raw).unwrap();
    let lines = input.lines();

    let values = lines.map(|l| {
        l.split_ascii_whitespace().map(|n| n.parse::<i64>().unwrap()).collect_vec()
    })
    .collect_vec();

    // polynomial
    // a + bn + cn^2 + dn^3 +...
    //
    // delta 1:
    // 
    //

    let mut sum = 0;
    let mut prev_sum = 0;
    for (line, v) in values.iter().enumerate() {
        let mut deltas = v.clone();
        let mut terms = Vec::new();
        
        loop {
            if deltas.iter().all(|d| *d == 0) {
               break;
            }
            
            terms.push(deltas[0]);
            deltas = deltas.iter().tuple_windows().map(|(a, b)| *b - *a).collect_vec();
        }

        // formula
        // base + an^d

        // let's get the next one the naive way.
        let mut deltas = vec![0;v.len() + 1];
        for &i in terms.iter().rev() {
            let mut acc = i;
            for j in 0..deltas.len() {
                let dj = deltas[j];
                deltas[j] = acc;
                acc += dj;
            }
        }

        let last_term = deltas.last().unwrap();

        let mut prev_term = 0;
        for &i in terms.iter().rev() {
            prev_term = i - prev_term;
        }

        println!("{line}: pred:{last_term} prev: {prev_term} polynomial: {terms:?}");

        sum += last_term;
        prev_sum += prev_term;
    }

    dbg!(sum);
    dbg!(prev_sum);

}
