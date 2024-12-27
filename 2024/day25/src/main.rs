use clap::Parser;
use itertools::Itertools;

#[derive(Parser, Debug)]
struct Args {
    #[arg(default_value = "input_sample.txt")]
    input_file: String,

    #[arg(short, long, default_value_t = false)]
    debug: bool,
}

fn check_no_overlap(lock: &Vec<&[u8]>, key: &Vec<&[u8]>, args: &Args) -> bool
{
    let res = lock.iter().zip(key).all(
        |(&l, &k)| 
            l.iter().zip(k).all(|(&l, &k)| l != b'#' || k != b'#'));

    if args.debug {
        lock.iter().zip(key).for_each(|(l, k)| {
            println!("{} {}", std::str::from_utf8(l).unwrap(), std::str::from_utf8(k).unwrap());
        });

        println!("RESULT: {res}");
        println!();
    }

    res
}

fn main() {

    let args = Args::parse();

    let input = std::fs::read_to_string(&args.input_file).expect("opened input file");

    let mut keys = Vec::new();
    let mut locks = Vec::new();

    let mut lines = input.lines();
    let mut rows = None;
    let mut cols = None;
    let mut fitting_pairs = 0usize;
    loop {
        let new_schematic = lines.by_ref()
            .take_while(|l| !l.is_empty())
            .map(|l| l.as_bytes())
            .collect_vec();

        if new_schematic.is_empty() {
            break;
        }

        let rc = *rows.get_or_insert(new_schematic.len());
        assert_ne!(rc, 0);
        assert_eq!(new_schematic.len(), rc);
        let cc = *cols.get_or_insert(new_schematic[0].len());
        assert_ne!(cc, 0);
        assert!(new_schematic.iter().all(|r| r.len() == cc));

        let (to_check, to_push) =
            if new_schematic[0][0] == b'#' {
                (&keys, &mut locks)
            } else {
                (&locks, &mut keys)
            };
        
        fitting_pairs += to_check.iter().filter(|&c| check_no_overlap(c, &new_schematic, &args)).count();
        to_push.push(new_schematic);
    }

    dbg!(fitting_pairs);
}
