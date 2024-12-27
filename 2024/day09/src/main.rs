use std::cmp::min;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::error::Error;
use std::env;
use std::fs;
use std::time::Instant;
use itertools::Itertools;

fn time_it<T>(name: &str, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();

    let ret = f();
    
    let elapsed = start.elapsed();
    println!("{name} took: {elapsed:?}");
    
    ret
}

fn compute_score(id: usize, offset: usize, count: u8) -> u64 {
    let id = id as u64;
    let offset = offset as u64;
    let count = count as u64;

    id * (offset..(offset + count)).sum::<u64>()
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

            while free_count != 0 {
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
                if back_count > count {
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

struct File {
    offset:usize,
    id:usize,
    file_size:usize
}

fn print_file_map(file_map: &[File]) {
    let mut offset = 0usize;

    file_map.iter().for_each(|f| {
        if (offset < f.offset) {
            print!("{}", ".".repeat(f.offset - offset).to_string());
        }

        print!("{}", f.id.to_string().repeat(f.file_size));
        offset = f.offset + f.file_size;
    });

    println!();
}

fn solve_part2(
    data: &[u8]
    ) -> u64
{
    let mut file_map = Vec::new();
    let mut free_space_map = vec![VecDeque::new();10];

    let mut id = 0usize;
    let mut offset = 0usize;
    for file_and_free in data.chunks(2) {
        let file_size = file_and_free[0] as usize;
        file_map.push(File{offset, id, file_size});

        // println!("{id}: {file_and_free:?}");

        offset += file_size as usize;

        id += 1;
        
        if let Some(&free_count) = file_and_free.get(1) {
            if free_count == 0 { continue };
            free_space_map[free_count as usize].push_back(offset);
            
            offset += free_count as usize;
        }
        
    }

    // print_file_map(&file_map);

    assert!(free_space_map[0].is_empty());
    free_space_map.iter().for_each(|m| assert!(m.iter().is_sorted()));

    let mut output_map = Vec::new();
    while let Some(mut f) = file_map.pop() {
        if let Some((free_count, free_list)) = free_space_map
            .iter_mut()
            .enumerate()
            .skip(f.file_size)
            .filter(|(free_count, free_list)| {
                free_list.front().map_or(false,|offset| *offset < f.offset )
            })
            .min_by_key(|(_, free_list)| *free_list.front().unwrap()) {

            let free_offset = free_list.pop_front().unwrap();

            // Move the file to the free location.
            assert!(free_offset < offset);
            assert!(f.file_size <= free_count);
            f.offset = free_offset;
            if f.file_size < free_count {
                // put the remaining element back on the free list
                let new_count = free_count - f.file_size;
                let new_offset = free_offset + f.file_size;
                let new_map = & mut free_space_map[new_count];
                let loc = new_map.binary_search(&new_offset).unwrap_err();
                new_map.insert(loc, new_offset);

                assert!(new_map.iter().is_sorted());
            }
        }

        output_map.push(f);
    }

    output_map.sort_by(|e1, e2| e1.offset.cmp(&e2.offset) );

    // print_file_map(&output_map);

    output_map.iter().map(|f| compute_score(f.id, f.offset, f.file_size as u8)).sum()

}   

enum DiskSlot {
    File(File),
    Free{offset: usize, count: usize}
}

fn solve_part2_bruteforce(data: &[u8]) -> u64 {

    let mut disk: Vec<DiskSlot> = Vec::new();

    let mut offset = 0usize;
    let mut id = 0usize;
    for file_and_free in data.chunks(2) {
        let file_size = file_and_free[0] as usize;
        assert!(file_size != 0);
        disk.push(DiskSlot::File(File{offset, id, file_size})); 

        // println!("{id}: {file_and_free:?}");

        offset += file_size as usize;
        id += 1;

        if (file_and_free.len() == 2 &&
            file_and_free[1] != 0) {

            disk.push(DiskSlot::Free{offset, count: file_and_free[1] as usize});
        }
    }

    0
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

    let part1 = time_it("part1", || solve_part1(files.clone(), frees.clone()));
    dbg!(part1);

    let part2 = time_it("part2", || solve_part2(&data));
    dbg!(part2);
    Ok(())
}
