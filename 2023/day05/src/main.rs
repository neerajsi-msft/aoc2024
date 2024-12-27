use std::{cmp::{max, min}, collections::BTreeMap, ops::Range};

use neerajsi::*;
use itertools::Itertools;
use rangemap::{RangeMap, RangeSet};
use scan_fmt::scan_fmt;

fn intersect_range(l: &Range<usize>, r: &Range<usize>) -> Range<usize> {
    let start = max(l.start, r.start);
    let end = max(min(l.end, r.end), start);

    start..end
}

fn union_range(l: &Range<usize>, r: &Range<usize>) -> Range<usize>
{
    let start = min(l.start, r.start);
    let end = max(max(l.end, r.end), start);
    start..end
}

fn shift_range(r: &Range<usize>, delta: isize) -> Range<usize>
{
    r.start.checked_add_signed(delta).unwrap()..r.end.checked_add_signed(delta).unwrap()
}

fn map_range_to_range(r: &Range<usize>, from: &Range<usize>, to: usize) -> Range<usize>
{
    let delta = to as isize - from.start as isize;
    shift_range(r, delta)
}

/*
fn advance_range(r: Range<usize>, new_start: usize) -> Range<usize> {
    let new_start = min(r.end, new_start);

    new_start..r.end
}
*/

fn map_seed_ranges(seed_ranges: Vec<Range<usize>>,
                   maps: &Vec<(&str, BTreeMap<usize, (usize, usize)>)>,
                   debug: bool) -> usize
{
    maps.iter().fold(
        seed_ranges,
        |in_v, m| {
            if debug {
                println!("processing map: {}: {:?}", m.0, m.1);
            }

            let mut out_v = Vec::new();
            for cur_range in in_v {
                if debug {
                    println!("\tmapping seeds {cur_range:?}");
                }

                let mut prev_end = cur_range.start;

                for (&s_end, &(d_start, count)) in m.1.range(cur_range.start..) {
                    let s_start = s_end - count;
                    if s_start < cur_range.end {
                        if prev_end < s_start {
                            let unmapped = prev_end..s_start;
                            if debug {
                                println!("\t\tleading unmapped: {unmapped:?}");
                            }

                            out_v.push(unmapped);
                        }

                        let m_range = (s_end - count)..s_end;
                        let intersection = intersect_range(&m_range, &cur_range);
                        
                        prev_end = intersection.end;

                        let dest_range = map_range_to_range(&intersection, &m_range, d_start);
                        if debug {
                            println!("\t\tmaps to: {dest_range:?}");
                        }
                        out_v.push(dest_range);
                    } else if s_start >= cur_range.end {
                        break;

                    } else {
                        let m_range = prev_end..s_end;
                        let intersection = intersect_range(&m_range, &cur_range);
                        
                        prev_end = intersection.end;
                        if debug {
                            println!("\t\tunmapped: {intersection:?}");
                        }

                        out_v.push(intersection);
                    }
                }

                let last_unmapped = prev_end..cur_range.end;
                let last_unmapped = intersect_range(&cur_range, &last_unmapped);
                if last_unmapped.start < last_unmapped.end {
                    if debug {
                        println!("\t\tlast_unmapped: {last_unmapped:?}");
                    }

                    out_v.push(last_unmapped);
                }
            }

            out_v.sort_by_key(|r| r.start);
            let mut out_index = 0;
            for i in 0..out_v.len() {
                assert!(out_index <= i);
                if intersect_range(&out_v[out_index], &out_v[i]).is_empty() {
                    out_index += 1;
                    out_v[out_index] = out_v[i].clone();
                } else {
                    out_v[out_index] = union_range(&out_v[out_index], &out_v[i]);
                }
            }

            out_v.truncate(out_index + 1);

            if debug {
                println!("coalesced: {out_v:?}");
            }

            out_v
        }
    )
    .iter()
    .map(|r| r.start)
    .min()
    .unwrap()
}

fn map_seed_ranges_rangemaps(
    seed_ranges: Vec<Range<usize>>,
    maps: &Vec<(&str, RangeMap<usize, usize>)>
    ) -> usize
{
    let seed_set = RangeSet::from_iter(seed_ranges);
    maps.iter().fold(
        seed_set,
        |in_set, m| {
            in_set.iter().flat_map(
                |r| {
                let overlaps =
                    m.1.overlapping(r).map(|(or, &dest)| {
                        let intersection = intersect_range(or, r);
                        map_range_to_range(&intersection, or, dest)
                    });
                
                let gaps = m.1.gaps(r)
                    .map(|g|{
                        intersect_range(&g, r)
                    });
                
                overlaps.chain(gaps)
            })
            .collect::<RangeSet<usize>>()
        }
    )
    .iter()
    .map(|r| r.start)
    .min()
    .unwrap()
}

fn main() {
    let input_raw = read_stdin_input();
    let input = std::str::from_utf8(&input_raw).unwrap();

    let debug = std::env::args().nth(1).is_some();

    let mut lines = input.lines();

    let seeds = lines.next().unwrap()
        .strip_prefix("seeds: ")
        .unwrap()
        .split_ascii_whitespace()
        .map(|s| s.parse::<usize>().unwrap())
        .collect_vec();

    assert!(lines.next().unwrap().is_empty());
    
    let mut maps = Vec::new();
    loop {
        let mut cur_map = BTreeMap::new();

        let Some(map_name) = lines.next() else { break };

        for map_line in lines.by_ref().take_while(|l| !l.is_empty()) {
            let (d_start, s_start, c) = scan_fmt!(map_line, "{} {} {}", usize, usize, usize).unwrap();

            cur_map.insert(s_start + c, (d_start, c));
        }

        maps.push((map_name, cur_map));
    }

    let min_location = seeds.iter()
        .map(
            |&s|
                maps.iter().fold(s, |s, m| {
                    if let Some((&s_end, &(d_start, count))) = m.1.range(s..).nth(0) {
                        let s_start = s_end - count;
                        if s >= s_start {
                            return d_start + (s - s_start);
                        } 
                    }
                    s
                })
        )
        .min().unwrap();

    dbg!(min_location);

    let seed_ranges1 = seeds.iter().map(|&s| s..(s+1)).collect_vec();
    let part1_r = map_seed_ranges(seed_ranges1.clone(), &maps, debug);
    dbg!(part1_r);

    let seed_ranges = seeds.iter().tuples().map(|(&start, &count)| start..(start+count)).collect_vec();

    let part2 = time_it("p2", || map_seed_ranges(seed_ranges.clone(), &maps, debug));

    dbg!(part2);

    let range_maps = maps.iter().map(|m| {
        (m.0,
            RangeMap::from_iter(m.1.iter().map(|(&s_end, &(d_start, count))| {
                ((s_end-count)..s_end, d_start)
            }))
        )
    }).collect_vec();


    let part1_rm = map_seed_ranges_rangemaps(seed_ranges1, &range_maps);
    let part2_rm = time_it("p2rm", || map_seed_ranges_rangemaps(seed_ranges, &range_maps));

    dbg!(part1_rm);
    dbg!(part2_rm);

}
