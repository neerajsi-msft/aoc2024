use std::{collections::HashMap, hash::Hash};

use itertools::Itertools;
use neerajsi::*;
use num::Integer;

fn main() {
    let input_raw = read_stdin_input();
    let input = std::str::from_utf8(&input_raw).unwrap();
    let mut lines = input.lines();

    let first = lines.next().unwrap();

    assert!(lines.next().unwrap().is_empty());

    let sets = lines
        .map(|l| scan_fmt::scan_fmt!(l, "{} = ({}, {})", String, String, String).unwrap())
        .collect_vec();

    let map: HashMap<&str, (&str, &str)> = HashMap::from_iter(
        sets.iter()
            .map(|s| (s.0.as_str(), (s.1.as_str(), s.2.as_str()))),
    );

    println!("instrs len: {}", first.len());

    let do_step = |pos, c| {
        let cur = map[&pos];
        match c {
            'L' => cur.0,
            'R' => cur.1,
            _ => panic!("Unexpected step {c}"),
        }
    };

    {
        let mut steps = 0;
        let mut pos = "AAA";

        for c in first.chars().cycle() {
            pos = do_step(pos, c);

            steps += 1;
            if pos == "ZZZ" {
                println!("found end");
                break;
            }
        }

        dbg!(steps);
    }

    {
        let targets = map.keys().filter(|&l| l.ends_with('Z')).collect_vec();

        dbg!(&targets);

        #[derive(Debug, Default, Clone)]
        struct GhostInfo<'a> {
            start_pos: &'a str,
            target_list: Vec<(&'a str, usize, usize, usize)>,
        }

        let ghosts = map
            .keys()
            .filter(|&l| l.ends_with('A'))
            .copied()
            .collect_vec();

        dbg!(&ghosts);

        let mut ghost_infos = Vec::new();

        for g in ghosts {
            #[derive(Debug, Default, Clone, Copy)]
            struct TargetInfo {
                initial_step_count: usize,
                cycle_step_count: usize,
                round_count: u8,
            };

            let mut target_map = HashMap::new();
            let mut pos = g;

            for (i, (phase, c)) in first.chars().enumerate().cycle().enumerate() {
                pos = do_step(pos, c);

                let steps = i + 1;

                if pos.ends_with("Z") {
                    let target_info = target_map.entry((pos, phase)).or_insert(TargetInfo {
                        initial_step_count: steps,
                        ..TargetInfo::default()
                    });

                    target_info.round_count += 1;
                    let round_count = target_info.round_count;
                    if round_count == 2 {
                        target_info.cycle_step_count = steps - target_info.initial_step_count;
                    }

                    // If there's one target, we have to hit it twice to prove that
                    // there's a cycle.  With more than one target, we have to hit
                    // the target 3 times so that we can make sure we hit all cyclical
                    // targets twice.
                    if round_count == 2 && target_map.len() == 1 || round_count == 3 {
                        break;
                    }
                }
            }

            let target_list = target_map
                .iter()
                .filter_map(|((&(pos, phase), target_info))| {
                    if target_info.round_count >= 2 {
                        Some((
                            pos,
                            phase,
                            target_info.initial_step_count,
                            target_info.cycle_step_count,
                        ))
                    } else {
                        None
                    }
                })
                .collect_vec();

            ghost_infos.push(GhostInfo {
                start_pos: g,
                target_list,
            });
        }

        dbg!(&ghost_infos);

        // Only one target
        assert!(ghost_infos.iter().all(|g| g.target_list.len() == 1));

        // All the same phase.
        assert!(ghost_infos.iter().map(|g| g.target_list[0].1).all_equal());

        // All cycles go through the beginning of the direction sequence, so no biasing
        // is necessary to deal with a non-cyclical prefix
        assert!(ghost_infos.iter().all(|g| g.target_list.iter().all(|t| t.2 == t.3)));

        let part2 = ghost_infos.iter()
            .map(|g| g.target_list[0].2)
            .fold(1, |acc, v| v.lcm(&acc));

        dbg!(part2);
    }
}
