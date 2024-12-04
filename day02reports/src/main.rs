use std::{
    env,
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("Command line error {0}")]
struct CommandLineError(String);

impl CommandLineError {
    fn new(msg: &str) -> Self {
        CommandLineError {
            0: String::from(msg),
        }
    }
}

#[derive(Debug, Clone, Error)]
enum ProgramError {
    #[error("Parsing error on line {0}")]
    ParseError(u32),
    #[error("Error in data on line {0}: {1}")]
    DataError(u32, String),
}

struct OneReport {
    report: Vec<i32>,
    deltas: Vec<i32>,
}

fn report_to_deltas(report: &[i32]) -> Vec<i32>
{
    report
        .windows(2)
        .map(|vals| vals[1] - vals[0])
        .collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    if args.len() != 2 {
        return Err(CommandLineError::new("Unexpected arg count.").into());
    }

    let file_name = args.nth(1).unwrap();

    println!("Opening file {}", file_name);

    let mut reports: Vec<OneReport> = Vec::new();

    let mut line_number = 0u32;
    let reader = BufReader::new(File::open(file_name)?);
    for line in reader.lines() {
        line_number += 1;
        let line = line?;

        let vals = line.split_ascii_whitespace();

        let report: Vec<i32> = vals
            .map(|val_str| val_str.parse::<i32>())
            .collect::<Result<_, _>>()?;

        let deltas  = report_to_deltas(report.as_slice());
        
        reports.push(OneReport{report, deltas});
    }

    safe_reports(&reports)?;

    Ok(())
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    #[default]
    None,
    Increasing,
    Decreasing
}

fn is_good_delta(delta: i32, direction: Direction) -> bool
{
    if direction == Direction::None { return false; }
    let delta = if direction == Direction::Decreasing { -delta } else { delta };
    (1..=3).contains(&delta)
}

fn new_direction_from_delta(delta: i32, direction: Direction) -> Direction
{
    match (direction, delta >= 0) {
        (Direction::None, true) => Direction::Increasing,
        (Direction::None, false) => Direction::Decreasing,
        (Direction::Increasing, true) => Direction::Increasing,
        (Direction::Decreasing, false) => Direction::Decreasing,
        (_, _) => Direction::None,
    }
}

fn safe_report_deltas_internal<'a, I>(deltas: I, direction: Direction) -> (Direction, Option<usize>)
    where I: Iterator<Item = &'a i32>
{
    let mut direction = direction;

    for (index, &delta) in deltas.enumerate() {
        let new_direction = new_direction_from_delta(delta, direction);

        if !is_good_delta(delta, new_direction) {
            return (direction, Some(index));
        }

        direction = new_direction;
    }

    (Direction::None, None)
}

fn safe_report_deltas(deltas: &[i32]) -> bool
{
    safe_report_deltas_internal(deltas.iter(), Default::default()).1.is_none()
}

fn safe_report_deltas_dampener(deltas: &[i32]) -> bool
{
    let (direction, bad_index) = safe_report_deltas_internal(deltas.iter(), Default::default());
    if bad_index.is_none() {
        return true;
    }

    let bad_index = bad_index.unwrap();
    let bad_delta = deltas[bad_index];

    //println!{"\tbad_index:{bad_index} bad_delta:{bad_delta}"}

    if bad_index + 1usize < deltas.len() {
        let delta = bad_delta + deltas[bad_index + 1];
        let new_direction = new_direction_from_delta(delta, direction);
        if is_good_delta(delta, new_direction) {
            if safe_report_deltas_internal(deltas.iter().skip(bad_index + 2), new_direction).1.is_none() {
                return true;
            }
        }
    }

    // If the first index is bad and we didn't use it to fix the subsequent delta, we can just drop it.
    if (bad_index == 0) {
        return safe_report_deltas_internal(deltas.iter().skip(1), Default::default()).1.is_none();
    }
    
    // see if we can fold this into the previous delta.
    let delta = bad_delta + deltas[bad_index - 1];
    let prev_direction = if bad_index == 1 { Direction::None } else { direction };
    let new_direction = new_direction_from_delta(delta, prev_direction);
    if is_good_delta(delta, new_direction) {
        return safe_report_deltas_internal(deltas.iter().skip(bad_index + 1), new_direction).1.is_none();
    }

    // If this is the last index and it wasn't folded backward, just drop it.
    if bad_index + 1usize == deltas.len() {
        return true;
    }

    false
}

fn safe_report(report: &Vec<i32>) -> bool {
    let mut direction: Option<bool> = None;

    for i in 0..(report.len() - 1) {
        let mut delta = report[i + 1] - report[i];

        if direction == None {
            direction = Some(delta >= 0);
        }

        if !direction.unwrap() {
            delta = -delta
        }

        if delta < 1 || delta > 3 {
            // println!("\tUnsafe delta: {}", delta);
            return false;
        }
    }

    true
}

fn safe_report_dampener(report: &[i32]) -> bool {
    if report.len() < 3 {
        return false;
    }

    let deltas = report
        .windows(2)
        .map(|vals| vals[1] - vals[0]);

    //   1       4      2      3
    //       3      -2      1
    //
    //   1       4      3      4
    //       3       -1     1
    //

    let mut zero_count = 0usize;
    let mut increase_count = 0usize;
    let mut decrease_count = 0usize;
    let mut zero_pos: Option<usize> = None;
    let mut increase_pos: Option<usize> = None;
    let mut decrease_pos: Option<usize> = None;
    let mut oob_pos: Option<usize> = None;
    for (i, delta) in deltas.enumerate() {

        if delta == 0 {
            zero_count += 1;
            zero_pos = Some(i);
        } else if delta > 0 {
            increase_count += 1;
            increase_pos = Some(i);
        } else {
            decrease_count += 1;
            decrease_pos = Some(i);
        }

        if delta < -3 || delta > 3 {
            oob_pos = if oob_pos.is_none() { Some(i) } else { oob_pos };
        }
    }

    /*
    println!(
        "\tzero:{zero_count} increase:{increase_count} decrease:{decrease_count} oob:{:?}",
        oob_pos
    );
    */

    if zero_count != 0 {
        if zero_count > 1 {
            return false;
        }

        if increase_count != 0 && decrease_count != 0 {
            return false;
        }

        let zero_pos = zero_pos.unwrap();

        assert_eq!(report[zero_pos], report[zero_pos + 1]);

        let mut new_report = Vec::from(report);
        new_report.remove(zero_pos);

        return safe_report(&new_report);
    } else {
        let try_remove = |pos: usize| {
            let mut new_report = Vec::from(report);
            new_report.remove(pos);
            return safe_report(&new_report);
        };

        let remove_loc;

        if (increase_count != 0) && (decrease_count != 0) {
            if increase_count > 1 && decrease_count > 1 {
                return false;
            }

            if increase_count == 1 {
                remove_loc = increase_pos.unwrap();
            } else {
                remove_loc = decrease_pos.unwrap();
            }
        } else {
            assert!(oob_pos.is_some());
            remove_loc = oob_pos.unwrap();
        }

        return try_remove(remove_loc) || try_remove(remove_loc + 1);
    }
}

fn safe_reports(reports: &Vec<OneReport>) -> Result<(), Box<dyn Error>> {
    let mut report_index = 0u32;
    let mut safe_reports: u32 = 0u32;
    let mut safe_reports_dampener: u32 = 0u32;

    for report in reports {
        if report.report.len() < 2 {
            return Err(
                ProgramError::DataError(report_index, String::from("Report too short.")).into(),
            );
        }

        println!("Report {}: {:?}", report_index, report.report);

        let is_safe = safe_report(&report.report);
        if is_safe {
            println!("\tSafe!");
        }

        safe_reports += is_safe as u32;
        if !is_safe {
            let is_safe = safe_report_dampener(&report.report);
            if is_safe {
                println!("\tSafe (dampened)!");
            }
            safe_reports_dampener += is_safe as u32;
        }
        report_index += 1;
    }

    safe_reports_dampener += safe_reports;

    println!("Safe reports: {}", safe_reports);
    println!("Safe reports(dampened): {safe_reports_dampener}");

    let safe_reports_deltas = reports.iter().filter(|&r| safe_report_deltas(&r.deltas)).count();
    println!("safe_reports(deltas):{safe_reports_deltas}");

    let safe_reports_deltas_dampener = reports.iter().filter(|&r| safe_report_deltas_dampener(&r.deltas)).count();
    println!("safe_reports_deltas_dampener: {safe_reports_deltas_dampener}");

    reports.iter().enumerate().for_each(|(index, r)| {
        if safe_report_deltas_dampener(&r.deltas) != (safe_report(&r.report) || safe_report_dampener(&r.report)) {
            println!("Mismatch report:({:?}) deltas:({:?})", r.report, r.deltas);
        }
    });

    Ok(())
}
