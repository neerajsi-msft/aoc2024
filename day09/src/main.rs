use std::cmp::min;
use std::error::Error;
use std::{env, io::read_to_string};
use std::{fs, io};

fn compute_score(id: usize, offset: usize, count: u8) -> u64 {
    let id = id as u64;
    let offset = offset as u64;
    let count = count as u64;
    (offset..(offset + count))
        .map(|o| {
            //print!("{}", id);
            o * id
        })
        .sum()
}

fn solve_part1<'a>(
    mut files: impl DoubleEndedIterator<Item = (usize, &'a u8)>,
    mut frees: impl Iterator<Item = &'a u8>,
) -> u64 {
    let mut back_remaining = None;
    let mut sum = 0u64;
    let mut offset = 0usize;

    'b: {
        loop {
            let f = files.next();
            let Some((id, &count)) = f else { break };
            sum += compute_score(id, offset, count);

            offset += count as usize;

            let Some(&free_count) = frees.next() else {
                break;
            };

            let mut free_count = free_count;

            while (free_count != 0) {
                if back_remaining.is_none() {
                    let Some((last_file_id, &last_file_count)) = files.next_back() else {
                        break 'b;
                    };

                    back_remaining = Some((last_file_id, last_file_count));
                }

                let (id, back_count) = back_remaining.unwrap();
                let count = min(free_count, back_count);
                sum += compute_score(id, offset, count);

                free_count -= count;
                offset += count as usize;
                if (back_count > count) {
                    back_remaining = Some((id, back_count - count));
                } else {
                    back_remaining = None;
                }
            }
        }
    }

    if let Some((id, count)) = back_remaining {
        sum += compute_score(id, offset, count);
    }

    sum
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_name = env::args().nth(1).unwrap_or("input_sample.txt".into());

    let str = fs::read_to_string(file_name)?;
    let data: Vec<u8> = str
        .trim_ascii()
        .as_bytes()
        .iter()
        .map(|c| *c - b'0')
        .collect();

    let files = data.iter().step_by(2).enumerate();
    let frees = data.iter().skip(1).step_by(2);

    let part1 = solve_part1(files.clone(), frees.clone());
    dbg!(part1);

    Ok(())
}
