use std::{
    cmp::Ordering,
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

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    if args.len() != 2 {
        return Err(CommandLineError::new("Unexpected arg count.").into());
    }

    let file_name = args.nth(1).unwrap();

    println!("Opening file {}", file_name);

    let mut reports: Vec<Vec<i32>> = Vec::new();

    let mut line_number = 0u32;
    let reader = BufReader::new(File::open(file_name)?);
    for line in reader.lines() {
        line_number += 1;
        let line = line?;

        let vals = line.split_ascii_whitespace();

        let report: Vec<i32> = vals
            .map(|val_str| val_str.parse::<i32>())
            .collect::<Result<_, _>>()?;
        reports.push(report);
    }

    safe_reports(&reports)?;

    Ok(())
}

fn safe_report(report: &Vec<i32>) -> bool {
    let mut direction: Option<bool> = None;

    for i in 0..(report.len() - 1) {
        let mut delta = report[i + 1] - report[i];

        let new_direction = (delta >= 0);

        if direction == None {
            direction = Some(new_direction);
        } else if (direction.unwrap() != new_direction) {
            return false;
        }

        if !direction.unwrap() {
            delta = -delta
        }

        if delta < 1 || delta > 3 {
            println!("\tUnsafe delta: {}", delta);
            return false;
        }
    }

    true
}

fn safe_report_dampener(report : &Vec<i32>) -> bool
{
    if report.len() < 3 {
        return false;
    }

    let deltas : Vec<i32> = report.as_slice().windows(2).map(|vals|{vals[1] - vals[0]}).collect();

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
    for i in 0..deltas.len() {
        let delta = deltas[i];

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

        if (delta < -3 || delta > 3) {
            oob_pos = if oob_pos.is_none() { Some(i) } else {oob_pos};
        }
    }

    println!("\tzero:{zero_count} increase:{increase_count} decrease:{decrease_count} oob:{:?}", oob_pos);

    if zero_count != 0 {
        if zero_count > 1 {
            return false;
        }

        if increase_count != 0 && decrease_count != 0 {
            return false;
        }

        let zero_pos = zero_pos.unwrap();

        assert_eq!(report[zero_pos], report[zero_pos + 1]);

        let mut new_report = report.clone();
        new_report.remove(zero_pos);

        return safe_report(&new_report);
    } else {
        let try_remove = |pos : usize| {
            let mut new_report = report.clone();
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

fn safe_reports(reports: &Vec<Vec<i32>>) -> Result<(), Box<dyn Error>> {
    let mut report_index = 0u32;
    let mut safe_reports: u32 = 0u32;
    let mut safe_reports_dampener : u32 = 0u32;

    for report in reports {
        if report.len() < 2 {
            return Err(
                ProgramError::DataError(report_index, String::from("Report too short.")).into(),
            );
        }

        println!("Report {}: {:?}", report_index, report);
    
        let is_safe = safe_report(report);
        if is_safe {
            println!("\tSafe!");
        }

        safe_reports += is_safe as u32;
        if !is_safe {
            let is_safe = safe_report_dampener(report);
            if (is_safe) {
                println!("\tSafe (dampened)!");
            }
            safe_reports_dampener += is_safe as u32;
        }
        report_index += 1;
    }

    safe_reports_dampener += safe_reports;

    println!("Safe reports: {}", safe_reports);
    println!("Safe reports(dampened): {safe_reports_dampener}");
    Ok(())
}
