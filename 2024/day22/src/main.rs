use std::{collections::VecDeque, time::Instant};
use clap::Parser;
use itertools::Itertools;


#[derive(Parser, Debug)]
#[command(about)]
struct Args {
    #[arg(default_value = "input_sample.txt")]
    input_file: String,

    #[arg(short, long, default_value_t = false)]
    debug: bool,

    #[arg(short='s', long)]
    debug_sequence: Option<usize>
}

fn mix(a: u64, b: u64) -> u64 {
    a ^ b
}

fn prune(a: u64) -> u64 {
    a % 16777216
}

fn round(v: u64) -> u64 {
    let v = prune(mix(v, v * 64));
    let v = prune(mix(v, v / 32));
    let v = prune(mix(v, v * 2048));
    v
}


fn simulate_value(v: u64) -> u64 {
    let mut v = v;
    for _ in 0..2000 {
        v = round(v);
    }

    v
}

type SequenceMap = Vec<u64>;

const SEQUENCE_MAP_SIZE: usize = 20 * 20 * 20 * 20;

fn sequence_to_id<'a>(sequence: impl IntoIterator<Item = i8>) -> usize {
    sequence.into_iter().fold(0, |acc, v| 20 * acc + if v >= 0 { v as usize } else { (9 - v) as usize })
}

fn id_to_sequence(id: usize) -> [i8; 4] {
    let mut decoded_sequence = [0; 4];
    let mut id = id;

    for i in 0..4 {
        let n = (id % 20) as i8;
        id /= 20;
        let n = if n <= 9 { n } else { 9 - n };
        decoded_sequence[3 - i] = n;
    }

    decoded_sequence
}

#[test]
fn test_id_to_sequence() {
    let test_array = [-2,1,-1,3];
    let id = sequence_to_id(test_array);
    let decoded = id_to_sequence(id);

    assert_eq!(test_array, decoded);
}

fn get_sequence_counts(start_v: u64, sequence_map: &mut SequenceMap, debug_sequence: Option<usize>) {
    let mut local_sequence_map = vec!(false; SEQUENCE_MAP_SIZE);
    let mut sequence = VecDeque::new();
    let mut v = start_v;
    for i in 0..2000 {
        sequence.push_back((v % 10) as u8);
        if sequence.len() == 5 {
            let sequence_id = sequence_to_id(sequence.iter().tuple_windows().map(|(&a, &b)| b as i8 - a as i8));

            if !local_sequence_map[sequence_id] {
                sequence_map[sequence_id] += *sequence.back().unwrap() as u64;
                if Some(sequence_id) == debug_sequence {
                    println!("Sequence {start_v}-{i}: {sequence_id}, {sequence:?}");
                    dbg!(&sequence);
                    dbg!(id_to_sequence(sequence_id));
                    dbg!(i);
                }

                local_sequence_map[sequence_id] = true;
            }

            sequence.pop_front();
        }
        v = round(v);
    }
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let start = Instant::now();

    let str = std::fs::read_to_string(&args.input_file)?;

    let inputs = str.lines().map(|l| l.parse::<u64>().unwrap() ).collect_vec();

    let mut sequence_map = SequenceMap::new();
    sequence_map.resize(SEQUENCE_MAP_SIZE, Default::default());


    let mut sum = 0;
    for i in inputs {
        let res = simulate_value(i);
        if args.debug {
            println!("{i}: {res}");
        }

        get_sequence_counts(i, &mut sequence_map, args.debug_sequence);

        sum += res;
    }

    let (id, &best_sequence_price) = sequence_map.iter().enumerate().max_by_key(|(_, &c)| c).unwrap();

    println!("sum: {sum}");


    let decoded_sequence = id_to_sequence(id);
    println!("seq: {decoded_sequence:?} cost: {best_sequence_price}");

    println!("time: {:?}", start.elapsed());

    Ok(())
}