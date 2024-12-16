use core::fmt;
use std::env;
use std::error;
use std::fs;
use std::error::Error;
use std::io::Write;
use std::path::Display;
use std::result::Result;
use std::thread::sleep;
use std::time::Duration;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use nalgebra;
use ratatui::prelude::*;
use ratatui::crossterm::event;

use nalgebra::dimension;
use nalgebra::Vector2;
use ratatui::widgets::canvas::Canvas;
use ratatui::widgets::canvas::Points;
use ratatui::widgets::Block;
use scan_fmt::scan_fmt;
use clap::Parser;

#[derive(Debug, Clone)]
struct Robot {
    p: Vector2<i64>,
    v: Vector2<i64>,
}

impl fmt::Display for Robot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "p:({},{}) v:({},{})", self.p.x, self.p.y, self.v.x, self.v.y)
    }
}

const DIM_SAMPLE: (i64, i64) = (11, 7);
const DIM_PUZZLE: (i64, i64) = (101, 103);

fn solve_naive_step(robots: &mut [Robot], dimensions: &Vector2<i64>)
{
    for r in robots {
        r.p += r.v;
        r.p = r.p.zip_map(dimensions, |a, d| {
            if a < 0 {
                a + d
            } else if a >= d {
                a - d
            } else {
                a
            }
        });
    }   
}

fn simulate_simple(robots: &mut [Robot], dimensions: &Vector2<i64>, step_count: usize) {
    for _ in 0..step_count {
        solve_naive_step(robots, dimensions);
    }

    //draw_grid_at_step(robots, dimensions, step_count);
}

fn draw_grid_at_step(robots: &[Robot], dimensions: &Vector2<i64>, step_count: usize) {
    println!("Step: {step_count}");
    let (rows, cols) = (dimensions.y as usize, dimensions.x as usize);
    let mut grid = vec![b'.'; rows * cols];
    for (i, r) in robots.iter().enumerate() {
        println!("i: {i:?}, r: {:?}", r);

        let cell = &mut grid[r.p.y as usize * cols + r.p.x as usize];
        *cell = match *cell { b'.' => b'1', _val => _val + 1 };
    }

    for r in 0..rows {
        let to_write = &grid[(r * cols)..((r + 1) * cols)];
        std::io::stdout().write(to_write).expect("write to stdout failed");
        std::io::stdout().write(&[b'\n']).expect("write to stdout failed");
    }
}

fn fix_velocities(robots: &mut [Robot], dimensions: Vector2<i64>)
{
    for r in robots {
        r.v = r.v.zip_map(&dimensions, |a, b| a % b);
    }
}

fn score_part1(robots: &[Robot], dimension: Vector2<i64>) -> i64 {
    let half_range = |d| [0..(d/2), (d/2+1)..d];

    let x_ranges = half_range(dimension.x);
    let y_ranges = half_range(dimension.y);

    /*
    let quadrant_ranges = [(x_ranges[0].clone(), y_ranges[0].clone()),
                                                          (x_ranges[1].clone(), y_ranges[0].clone()),
                                                          (x_ranges[1].clone(), y_ranges[0].clone()),
                                                          (x_ranges[1].clone(), y_ranges[1].clone())];
    */

    let mut quadrant_count = [0i64;4];

    for (i, r) in robots.iter().enumerate() {
        let x_quad = x_ranges.iter().position(|q| q.contains(&r.p.x));
        let y_quad = y_ranges.iter().position(|q| q.contains(&r.p.y));
        
        // println!("{i}: {r}: {x_quad:?}, {y_quad:?}");

        match (x_quad, y_quad) {
            (Some(x), Some(y)) => quadrant_count[y * 2 + x] += 1,
            _ => ()
        }
    }

    // dbg!(quadrant_count);

    let safety_factor: i64 = quadrant_count.iter().product();
    // dbg!(safety_factor);
    safety_factor

}

#[cfg(test)]
mod tests{

    #[test]
    fn negative_modulus() {
        let x = -10;
        let y = 7;
        println!("{x} % {y} = {}", x % y);
    }

}

fn simulate_ratatui(robots: &mut Vec<Robot>, dimensions: Vector2<i64>, start_step_no: usize, args: &Args) -> std::io::Result<()>
{
        // all coordinates are reversed for x and y, since the TUI coordinate system is different
    // from the puzzle coordinate system.
    let mut step_no = start_step_no;

    let mut terminal = ratatui::init();

    loop {
        let points = robots.iter().map(|r| (r.p.x as f64, (dimensions.y - r.p.y - 1) as f64)).collect::<Vec<_>>();

        let score = score_part1(&robots, dimensions);
        let mut event = None;

        if args.render_threshold.is_none_or(|rt| rt >= score) {
            terminal.draw(|frame| {
                let area = frame.area();
                let step_str = format!("step:{step_no} score:{score}");
                let canvas = Canvas::default()
                    .block(Block::bordered().title(step_str.as_str()))
                    .x_bounds([0.0, dimensions.x as f64])
                    .y_bounds([0.0, dimensions.y as f64])
                    .paint(| ctx | {
                        ctx.draw(&Points{coords: &points, color: Color::Green});
                    });
    
                frame.render_widget(canvas, area);
            })?;
    
            if event::poll(Duration::from_millis(1))? {
                event = Some(event::read()?);
            }
        }
        

        if args.num_steps == Some(step_no) {
            event = Some(event::read()?);
        }

        if args.score_threshold.is_none_or(|st| st >= score) {

            event = Some(event::read()?);
        }

        if let Some(Event::Key(key_event)) = event {
            if key_event.code == KeyCode::Char('q') {
                break;
            }            
        }

        //sleep(Duration::from_millis(200));
        solve_naive_step(robots.as_mut_slice(), &dimensions);
        step_no += 1;
    }

    //terminal.clear()?;

    ratatui::restore();

    Ok(())
}

#[derive(Parser, Debug)]
#[command(about)]
/// Simulate robots moving around a toroidal field.
struct Args {
    #[arg(default_value = "input_sample.txt")]
    input_file: String,

    #[arg(short, long, default_value_t = false)]
    is_sample: bool,

    #[arg(long, default_value_t = false)]
    tui: bool,

    #[arg(short, long, default_value_t = 0)]
    start_step: usize,

    #[arg(short, long)]
    num_steps: Option<usize>,

    #[arg(short='t', long)]
    score_threshold: Option<i64>,

    #[arg(short='r', long)]
    render_threshold: Option<i64>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let str = fs::read_to_string(&args.input_file)?;

    let robots_original = str.trim_ascii()
        .lines()
        .map(
            |s| {
                let r = scan_fmt!(s, "p={},{} v={},{}", i64, i64, i64, i64).unwrap();
                Robot{
                    p: Vector2::new(r.0, r.1),
                    v: Vector2::new(r.2, r.3)
                }
            }
        ).collect::<Vec<_>>();

    let dimensions = if (args.is_sample) { DIM_SAMPLE } else { DIM_PUZZLE };
    let dimensions = Vector2::new(dimensions.0, dimensions.1);
    
    let mut robots = robots_original.clone();
    fix_velocities(&mut robots, dimensions);

    simulate_simple(&mut robots, &dimensions, 100);
    let robots_part1 = robots.clone();

    let mut robots = robots_original.clone();


    if args.tui {
        simulate_simple(&mut robots, &dimensions, args.start_step);
        simulate_ratatui(&mut robots, dimensions, args.start_step, &args)?;
    } else {
        simulate_simple(&mut robots, &dimensions, args.start_step);
        draw_grid_at_step(&robots, &dimensions, args.start_step);
        //simulate_ratatui(&mut robots, dimensions, start_step_no)?;
    }

    let part1 = score_part1(&robots_part1, dimensions);
    dbg!(part1);
    Ok(())
}
