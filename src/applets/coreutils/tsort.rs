use crate::core::{Applet, Result};
use crate::core::fs::read_bytes_or_stdin;
use std::collections::{HashMap, HashSet, VecDeque};
use std::ffi::OsString;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct TsortApplet;
impl Applet for TsortApplet {
    fn name(&self) -> &'static str {
        "tsort"
    }
    fn description(&self) -> &'static str {
        "Topological sort"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let file = if args.is_empty() || args[0].as_bytes() == b"-" {
            Path::new("-")
        } else {
            Path::new(&args[0])
        };

        let content = read_bytes_or_stdin(file)?;
        let s = String::from_utf8_lossy(&content);
        let tokens: Vec<&str> = s.split_whitespace().collect();

        if !tokens.len().is_multiple_of(2) {
            eprintln!("tsort: odd number of tokens");
            return Ok(1);
        }

        let mut in_degrees: HashMap<String, usize> = HashMap::new();
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        let mut all_nodes: Vec<String> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();

        let mut i = 0;
        while i < tokens.len() {
            let u = tokens[i].to_string();
            let v = tokens[i + 1].to_string();
            i += 2;

            if seen.insert(u.clone()) {
                all_nodes.push(u.clone());
                in_degrees.entry(u.clone()).or_insert(0);
            }
            if seen.insert(v.clone()) {
                all_nodes.push(v.clone());
                in_degrees.entry(v.clone()).or_insert(0);
            }

            if u != v {
                adj.entry(u.clone()).or_default().push(v.clone());
                *in_degrees.entry(v).or_insert(0) += 1;
            }
        }

        let mut queue = VecDeque::new();
        for node in &all_nodes {
            if in_degrees[node] == 0 {
                queue.push_back(node.clone());
            }
        }

        let mut order = Vec::new();
        while let Some(node) = queue.pop_front() {
            order.push(node.clone());
            if let Some(neighbors) = adj.get(&node) {
                for neighbor in neighbors {
                    let deg = in_degrees.get_mut(neighbor).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        if order.len() != all_nodes.len() {
            eprintln!("tsort: cycle detected");
            return Ok(1);
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();
        for node in order {
            writeln!(handle, "{}", node)?;
        }
        Ok(0)
    }
}
