use std::{clone, collections::{btree_set, BTreeMap, HashMap, HashSet}, hash::Hash, path::Display, str::FromStr, time::Instant};

use clap::Parser;
use itertools::Itertools;
use petgraph::algo::bellman_ford;
use scan_fmt::scan_fmt;
use strum::EnumString;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, EnumString)]
enum OPERATOR {
    XOR,
    OR,
    AND
}

#[derive(Parser, Debug)]
#[command(about)]
struct Args {
    #[arg(default_value = "input_sample.txt")]
    input_file: String,

    #[arg(default_value = "")]
    swaps:String,

    #[arg(short, long, default_value_t = false)]
    debug: bool,
}

type ExpressionMap = HashMap<String, (OPERATOR, String, String)>;
type ValueMap = HashMap<String, bool>;


#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
enum Operand {
    InputX(u8),
    InputY(u8),
    Generate(u8),
    Propagate(u8),
    CarryPSum(u8),
    PSum(u8),
    CarryOut(u8),
    Original(String),
}

impl <'a> std::fmt::Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Operand::*;
        match self {
            Original(s) => write!(f, "{s}"),
            InputX(v) => write!(f, "x{v:02}"),
            InputY(v) => write!(f, "y{v:02}"),
            Generate(v) => write!(f, "G{v:02}"),
            Propagate(v) => write!(f, "P{v:02}"),
            CarryPSum(v) => write!(f, "PS{v:02}"),
            PSum(v) => write!(f, "S{v:02}"),
            CarryOut(v) => write!(f, "C{v:02}"),
        }
    }
}

fn main() {
    let args = Args::parse();

    let start = Instant::now();

    let mut swaps: HashMap<&str, &str> = HashMap::new();

    if !args.swaps.is_empty() {
        for (a, b) in args.swaps.split(',').tuples() {
            swaps.insert(a, b);
            swaps.insert(b, a);
        }
    }

    let str = std::fs::read_to_string(&args.input_file).expect("Could not read input");

    let mut lines = str.lines();

    let values: ValueMap = lines.by_ref()
        .take_while(|l| !l.is_empty())
        .map(|l| scan_fmt!(l, "{}: {d}", String, u8).expect("Parse error"))
        .map(|(s, v)| (s, v != 0))
        .collect();

    let expressions_original = lines
        .map(|l|  {
            let res = scan_fmt!(l, "{} {} {} -> {}", String, String, String, String);
            if res.is_err() {
                panic!("Could not parse {l}: {res:?}");
            }

            let (a, op, b, out) = res.unwrap();
            
            let result = if let Some(s) = swaps.get(out.as_str()) {
                s.to_string()
            } else {
                out
            };

            let (a, b) = if a < b { (a, b) } else { (b, a) };
            
            (result, (OPERATOR::from_str(&op).unwrap(), a, b))
        }).collect_vec();

    let expressions: ExpressionMap = expressions_original.iter()
        .map(|v| v.clone() )
        .collect();
        
    let mut solved = values.clone();

    fn solve_recurse(search: &str, expressions: &ExpressionMap, values: &ValueMap) -> Option<bool> {
        if let Some(&value) = values.get(search) {
            return Some(value);
        }

        let (op, left, right) = expressions.get(search)?;
        let left = solve_recurse(left, expressions, values);
        let right = solve_recurse(right, expressions, values);

        let (a, b) = if left.is_some() { (left, right) } else { (right, left) };

        match op {
            OPERATOR::AND => {
                if let Some(a) = a {
                    if !a { Some(false) } else { b }
                } else {
                    assert_eq!(b, None);
                    None
                }
            }
            OPERATOR::OR => {
                if let Some(a) = a {
                    if a { Some(true) } else { b }
                } else {
                    assert_eq!(b, None);
                    None
                }
            }
            OPERATOR::XOR => {
                if let (Some(a), Some(b)) = (a, b) {
                    Some(a ^ b)
                } else {
                    None
                }
            }
        }
    }

    let mut z = 0;
    for i in 0..64 {
        let z_wire = format!("z{i:02}");

        let val = solve_recurse(&z_wire, &expressions, &mut solved);
        
        if args.debug {
            println!("{z_wire} = {val:?}");
        }
        
        let Some(val) = val else { break };

        z |= (val as usize ) << (i as usize);
    }

    println!("z = {z}");

    fn print_recurse(search: &str, expressions: &ExpressionMap, rename_map: &HashMap<String, Operand>) {
        if let Some(ren) = rename_map.get(search) {
            print!("{ren}({search})");
            return;
        }
        
        let expr = expressions.get(search);
        let Some(expr) = expr else {
            print!("{search}");
            return;
        };

        let (op, a, b) = expr;

        print!("({search}: {:?} ", op);
        print_recurse(&a, expressions, rename_map);
        print!(" ");
        print_recurse(&b, expressions, rename_map);
        print!(")");
    }

    use Operand::*;

    let mut normalized_exprs = expressions.iter().map(
        |(res, (op, a, b))| {
            (Original(res.clone()), Original(a.clone()), *op, Original(b.clone()))
        })
        .collect_vec();

    let mut rename_map: HashMap<String, Operand> = HashMap::new();
    let mut rename_map_reverse: BTreeMap<Operand, String> = BTreeMap::new();
    let mut name_wire = |orig: &mut Operand, name: Operand, rename_map: &mut HashMap<String, Operand>| {
        let Original(str) = orig else { todo!("renaming {orig} to {name}") };
        rename_map.insert(str.to_string(), name.clone());
        rename_map_reverse.insert(name.clone(), str.to_string());
        *orig = name;
    };

    for (res, a, op, b) in normalized_exprs.iter_mut() {
        let update_input = |inp: &mut Operand| {
            match inp {
                Original(s) => {
                    let (s_first, rest) = s.split_at(1);
                    if s_first == "x" {
                        *inp = InputX(rest.parse::<u8>().unwrap())
                    } else if s_first == "y" {
                        *inp = InputY(rest.parse::<u8>().unwrap())
                    
                    };
                }
                _ => {}
            }
        };

        update_input(a);
        update_input(b);

        if let (InputX(v_a), InputY(v_b)) = (a, b) {
            if v_a == v_b {

                let new_name = match op {
                    OPERATOR::AND => if *v_a == 0 { CarryOut(*v_a) } else {Generate(*v_a)},
                    OPERATOR::XOR => PSum(*v_a),
                    OPERATOR::OR  => Propagate(*v_a),
                };
    
                name_wire(res, new_name, &mut rename_map);
            }
        }
    }



    fn remap_operand(op: &mut Operand, rename_map: &HashMap<String, Operand>) {
        if let Original(n) = op {
            if let Some(new_name) = rename_map.get(n) {
                *op = new_name.clone();
            }
        }
    }

    use OPERATOR::*;

    loop {
        let mut changed = false;
        for (res, a, op, b) in normalized_exprs.iter_mut() {
            
            if !matches!(res, Original(_)) {
                continue;
            }

            remap_operand(a, &rename_map);
            remap_operand(b, &rename_map);
            if *a > *b {
                std::mem::swap(a, b);
            }

            match (a, op, b) {
                (PSum(v_a), AND, CarryOut(v_b)) => {
                    if *v_b + 1 == *v_a {
                        name_wire(res, CarryPSum(*v_a), &mut rename_map);
                        changed = true;
                    }
                },
                (Generate(v_a), OR, CarryPSum(v_b)) => {
                    if *v_a == *v_b {
                        name_wire(res, CarryOut(*v_a), &mut rename_map);
                        changed = true;
                    }
                }                
                _ => {}
            }
        }

        if !changed { break }
    }
    
    // Half-adder:
    //      z00 = x00 ^ y00
    //      c00 = x00 & y00
    //
    // Full-adder:
    //      Zn = Xn ^ Yn ^ C(n-1)
    //      Cn = (Xn & Yn) | (C(n-1) & (Xn | Yn)) 
    //              -OR-
    //      Cn = (Xn & Yn) | (C(n-1) & (Xn ^ Yn))
    
    // Generate: Gn = Xn & Yn
    // Sum:      Sn = Xn ^ Yn
    // Prop:     Pn = Xn | Yn

    // Cn = Gn-1 | (Cn-1 & (Pn | Sn))
    //
        
    normalized_exprs.sort_by(|a, b| (&a.1, a.2, &a.3).cmp(&(&b.1, b.2, &b.3)));

    for (res, a, op, b) in normalized_exprs.iter_mut() {
        
        
        if let Generate(v) = a {
            if *v == 1 {
                
            }
        }
    }

    normalized_exprs.iter().for_each(|(res, a, op, b)| {
        let a_str = a.to_string();
        let b_str = b.to_string();
        let r_str = res.to_string();
        let a_ren = rename_map_reverse.get(&a).unwrap_or(&a_str);
        let b_ren = rename_map_reverse.get(&b).unwrap_or(&b_str);
        let res_ren = rename_map_reverse.get(&res).unwrap_or(&r_str);
        println!("{a}({a_ren}) {op:?} {b}({b_ren}) -> {res}({res_ren})");
    });



    let mut values = ValueMap::new(); 


    for col in 0..44 {
        let x_wire: String = format!("x{col:02}");
        let y_wire: String = format!("y{col:02}");
        let z_wire: String = format!("z{col:02}");

        values.clear();

        //
        //  Xn   |   Yn   |  Cn-1  | Cn
        //-------------------------------
        //  0        0        X      0
        //  0        1       None    None 
        //  1        0       None    None
        //  0        1        1      1
        //  1        0        1      1
        //  
        //  1        1        X      1
        //

        print!("{z_wire}:");
        print_recurse(&z_wire, &expressions, &rename_map);
        println!();
    }

    println!("swaps: {}", swaps.keys().sorted().join(","));
}
