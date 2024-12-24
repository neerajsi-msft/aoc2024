use std::{collections::{BTreeSet, HashMap, HashSet}, time::Instant};
use arrayvec::ArrayVec;
use clap::Parser;
use itertools::Itertools;
use petgraph::{graph::UnGraph, prelude::UnGraphMap, visit::{IntoNodeIdentifiers, IntoNodeReferences}};
use scan_fmt::scan_fmt;


#[derive(Parser, Debug)]
#[command(about)]
struct Args {
    #[arg(default_value = "input_sample.txt")]
    input_file: String,

    #[arg(short, long, default_value_t = false)]
    debug: bool,
}

type NetworkGraph<'a> = UnGraphMap<&'a str, ()>;

fn main() {

    let args = Args::parse();

    let start = Instant::now();

    let str = std::fs::read_to_string(&args.input_file).expect("Could not read input");

    let mut inputs = str.lines().map(|l| {
        scan_fmt!(l, "{}-{}", String, String).expect("line format error")
    } ).collect_vec();

    inputs.sort();

    let input_strs = inputs.iter().map(|(a, b)| (a.as_str(), b.as_str()));

    let graph= NetworkGraph::from_edges(input_strs);

    let mut covered_set = BTreeSet::new();

    for n in graph.nodes() {
        if n.chars().nth(0) == Some('t') {
            for (n1, n2) in graph.neighbors(n).tuple_combinations() {
                if graph.contains_edge(n1, n2) {
                    let mut group = [n, n1, n2];
                    group.sort();

                    covered_set.insert(group);
                }
            }
        }
    }

    if args.debug {
        println!("Covered Set:");
        println!("{}", covered_set.iter().format_with("\n", |elt, f| {
            f(&elt.iter().format(","))
        }));
    }

    dbg!(covered_set.len());

    fn bron_kerbosch<'a>(graph: &UnGraphMap<&'a str, ()>, cur_clique: &mut Vec<&'a str>, mut candidates: HashSet<&'a str>, mut excluded: HashSet<&'a str>, biggest_clique: &mut Vec<&'a str>) {
        if candidates.is_empty() && excluded.is_empty() {
            if cur_clique.len() > biggest_clique.len() {
                *biggest_clique = cur_clique.clone();
            }
        }
        
        while let Some(&n) = candidates.iter().next() {
            candidates.remove(n);

            let neighbors = graph.neighbors(n);

            fn hash_filter<'a>(hash_set: &HashSet<&str>, iter: impl Iterator<Item = &'a str>) -> HashSet<&'a str> {
                iter.filter(|v| hash_set.contains(v)).collect()
            };

            
            let neighbors_candidates = hash_filter(&candidates, neighbors.clone());
            let neighbors_excluded = hash_filter(&excluded, neighbors);

            cur_clique.push(n);
            bron_kerbosch(graph, cur_clique, neighbors_candidates, neighbors_excluded, biggest_clique);
            cur_clique.pop();
        }

    }

    let mut biggest_clique = Vec::new();
    let mut cur_clique = Vec::new(); 
    bron_kerbosch(&graph, &mut cur_clique, graph.nodes().collect(), HashSet::new(), &mut biggest_clique);

    biggest_clique.sort();
    println!("biggest clique: {}", biggest_clique.iter().format(","));
}
