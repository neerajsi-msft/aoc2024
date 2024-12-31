use itertools::Itertools;
use neerajsi::*;

fn get_connected_dirs(cell: u8) -> Option<[CardinalDirectionName; 2]> {
    use CardinalDirectionName::*;

    let dirs = match cell {
        b'|' => [N,S],
        b'-' => [E,W],
        b'L' => [N,E],
        b'J' => [N,W],
        b'7' => [W,S],
        b'F' => [S,E],
        b'.' => return None,
        _ => panic!("unknown tile")
    };

    Some(dirs)
}

fn main() {
    let input_raw = read_stdin_input();
    let input = std::str::from_utf8(&input_raw).unwrap();
    let lines = input.lines();

    let map = lines.map(|l| {
        l.as_bytes()
    })
    .collect_vec();

    let mut start_pos = None;
    for (r, row) in map.iter().enumerate() {
        for (c, cell) in row.iter().enumerate() {
            if *cell == b'S' {
                start_pos = Some([r,c]);
                break;
            }
        }
    }

    let start_pos = start_pos.unwrap();

    let mut cells = Vec::new();

    use CardinalDirectionName::*;

    let mut pos = start_pos;
    let grid = Grid::new(map.len(), map[0].len());
    let mut from_dir = None;
    for dir in [N,S,E,W] {
        let opposite = opposite_dir_cardinal(dir);
        let next_pos = grid.add_cardinal(pos, dir).unwrap();

        if let Some(conns) = get_connected_dirs(index2d_array!(map, next_pos)) {
            if conns.contains(&opposite) {
                pos = next_pos;
                from_dir = Some(opposite);
                break;
            }
        }        
    }


    assert!(pos != start_pos);

    
    let mut from_dir = from_dir.unwrap();
    cells.push(start_pos);
    while pos != start_pos {        
        cells.push(pos);
        let connected_dirs = get_connected_dirs(index2d_array!(map, pos)).unwrap();

        assert!(connected_dirs.contains(&from_dir));
        let next_dir = if connected_dirs[0] == from_dir { connected_dirs[1] } else { connected_dirs[0] };

        pos = grid.add_cardinal(pos, next_dir).unwrap();
        from_dir = opposite_dir_cardinal(next_dir);
    }

    let farthest_point = cells.len() / 2;
    dbg!(farthest_point);
    dbg!(cells.len());

    let mut path_map = vec2d!(grid.rows(), grid.cols(), b'.');
    for cell in cells {
        let v = index2d_array!(map, cell);
        let v = if v == b'S' { b'-' } else { v };
        index2d_array!(path_map, cell) = v;
    }

    fn draw_path_map(path_map: &[Vec<u8>]) {
        for l in path_map {
            println!("{}", std::str::from_utf8(&l).unwrap());
        }    
        println!();
    }

    draw_path_map(&path_map);

    let mut inside_count = 0;
    for r in 0..grid.rows() {
        let mut inside = false;
        let mut seen_ns = [false, false];
        let mut in_wall = false;
        for c in 0..grid.cols() {
            let loc = [r,c];
            
            let cur_cell = index2d_array!(path_map, loc);
            match cur_cell as char {
                '.' => {
                    if inside {
                        index2d_array!(path_map, loc) = b'*';
                        inside_count += 1;
                    }
                }

                // ignore east-west connections
                '-' => {assert!(in_wall)}

                '|' => {assert!(!in_wall); inside = !inside}

                _ => {
                    let Some(connected) = get_connected_dirs(cur_cell as u8)
                    else {
                        panic!("Unexpected cell value: {cur_cell}");
                    };

                    seen_ns[0] |= connected.contains(&N);
                    seen_ns[1] |= connected.contains(&S);
                    in_wall = !in_wall;
                    if !in_wall {
                        if seen_ns == [true, true] {
                            inside = !inside;
                        }

                        seen_ns = [false, false];
                    }
                }
            }
        }
    }

    draw_path_map(&path_map);

    println!("inside_count = {inside_count}");

}