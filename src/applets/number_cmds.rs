use crate::core::fs::read_bytes_or_stdin;
use crate::core::{Applet, Result};
use std::collections::{HashMap, HashSet, VecDeque};
use std::ffi::OsString;
use std::io::{self, BufRead, Write};
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

pub struct SeqApplet;
impl Applet for SeqApplet {
    fn name(&self) -> &'static str {
        "seq"
    }
    fn description(&self) -> &'static str {
        "Print numbers from FIRST to LAST"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut sep = "\n".to_string();
        let mut pad = false;
        let mut pos_args = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();
            if bytes == b"-w" {
                pad = true;
            } else if bytes == b"-s" {
                if i + 1 < args.len() {
                    sep = args[i + 1].to_string_lossy().to_string();
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-s") {
                sep = arg.to_string_lossy()[2..].to_string();
            } else {
                pos_args.push(arg.to_string_lossy().to_string());
            }
            i += 1;
        }

        if pos_args.is_empty() || pos_args.len() > 3 {
            eprintln!("seq: invalid arguments");
            return Ok(1);
        }

        let (first_str, step_str, last_str) = match pos_args.len() {
            1 => ("1".to_string(), "1".to_string(), pos_args[0].clone()),
            2 => (pos_args[0].clone(), "1".to_string(), pos_args[1].clone()),
            3 => (
                pos_args[0].clone(),
                pos_args[1].clone(),
                pos_args[2].clone(),
            ),
            _ => unreachable!(),
        };

        let first: f64 = match first_str.parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("seq: invalid number: {}", first_str);
                return Ok(1);
            }
        };
        let step: f64 = match step_str.parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("seq: invalid number: {}", step_str);
                return Ok(1);
            }
        };
        let last: f64 = match last_str.parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("seq: invalid number: {}", last_str);
                return Ok(1);
            }
        };

        if step == 0.0 {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            loop {
                if writeln!(handle, "{}", first_str).is_err() {
                    break;
                }
            }
            return Ok(0);
        }

        let argv_strs: Vec<&str> = match pos_args.len() {
            1 => vec![&pos_args[0]],
            2 => vec![&pos_args[0], &pos_args[1]],
            3 => vec![&pos_args[0], &pos_args[1], &pos_args[2]],
            _ => unreachable!(),
        };

        let mut width = 0usize;
        let mut frac_part = 0usize;
        for (idx, arg_str) in argv_strs.iter().enumerate() {
            let dot_pos = arg_str.find('.').unwrap_or(arg_str.len());
            let w = dot_pos;
            let f = arg_str.len() - dot_pos;
            if width < w {
                width = w;
            }
            if idx + 1 == argv_strs.len() {
                break;
            }
            if frac_part < f {
                frac_part = f;
            }
        }
        if frac_part > 0 {
            frac_part -= 1;
            if frac_part > 0 {
                width += frac_part + 1;
            }
        }
        if !pad {
            width = 0;
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();
        let mut v = first;
        let mut n = 0;
        let mut sep_cur = "";

        while if step >= 0.0 { v <= last } else { v >= last } {
            let formatted = if frac_part > 0 {
                format!("{:.*}", frac_part, v)
            } else {
                format!("{:.0}", v)
            };
            let (is_neg, num_digits) = if let Some(stripped) = formatted.strip_prefix('-') {
                (true, stripped)
            } else {
                (false, formatted.as_str())
            };
            handle.write_all(sep_cur.as_bytes())?;
            if width > 0 && formatted.len() < width {
                let pad_count = width - formatted.len();
                if is_neg {
                    handle.write_all(b"-")?;
                }
                for _ in 0..pad_count {
                    handle.write_all(b"0")?;
                }
                handle.write_all(num_digits.as_bytes())?;
            } else {
                handle.write_all(formatted.as_bytes())?;
            }
            sep_cur = &sep;
            n += 1;
            v = first + (n as f64) * step;
        }

        if n > 0 {
            handle.write_all(b"\n")?;
        }
        Ok(0)
    }
}

pub struct FactorApplet;
impl Applet for FactorApplet {
    fn name(&self) -> &'static str {
        "factor"
    }
    fn description(&self) -> &'static str {
        "Print prime factors"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let inputs: Vec<String> = if args.is_empty() {
            let stdin = io::stdin();
            let mut words = Vec::new();
            for l in stdin.lock().lines().map_while(std::result::Result::ok) {
                for w in l.split_whitespace() {
                    words.push(w.to_string());
                }
            }
            words
        } else {
            args.iter()
                .map(|s| s.to_string_lossy().to_string())
                .collect()
        };

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        for arg in &inputs {
            let trimmed = arg.trim();
            let cleaned = trimmed.strip_prefix('+').unwrap_or(trimmed);
            match cleaned.parse::<u128>() {
                Ok(0) => {
                    writeln!(handle, "0:")?;
                }
                Ok(1) => {
                    writeln!(handle, "1:")?;
                }
                Ok(n) => {
                    let factors = factorize(n);
                    let fact_strs: Vec<String> = factors.iter().map(|f| f.to_string()).collect();
                    writeln!(handle, "{}: {}", n, fact_strs.join(" "))?;
                }
                Err(_) => {
                    eprintln!("factor: '{}': is not a valid positive integer", arg);
                }
            }
        }
        Ok(0)
    }
}

fn mul_mod(mut a: u128, mut b: u128, m: u128) -> u128 {
    let mut res = 0;
    a %= m;
    while b > 0 {
        if b % 2 == 1 {
            res = (res + a) % m;
        }
        a = (a * 2) % m;
        b /= 2;
    }
    res
}

fn pow_mod(mut base: u128, mut exp: u128, m: u128) -> u128 {
    let mut res = 1;
    base %= m;
    while exp > 0 {
        if exp % 2 == 1 {
            res = mul_mod(res, base, m);
        }
        base = mul_mod(base, base, m);
        exp /= 2;
    }
    res
}

fn is_prime_mr(n: u128) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 || n == 3 || n == 5 || n == 7 {
        return true;
    }
    if n.is_multiple_of(2) || n.is_multiple_of(3) || n.is_multiple_of(5) || n.is_multiple_of(7) {
        return false;
    }

    let mut d = n - 1;
    let mut s = 0;
    while d.is_multiple_of(2) {
        d /= 2;
        s += 1;
    }

    let bases = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
    'outer: for &a in &bases {
        if n <= a {
            break;
        }
        let mut x = pow_mod(a, d, n);
        if x == 1 || x == n - 1 {
            continue;
        }
        for _ in 0..s - 1 {
            x = mul_mod(x, x, n);
            if x == n - 1 {
                continue 'outer;
            }
        }
        return false;
    }
    true
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

fn pollard_rho(n: u128) -> u128 {
    if n.is_multiple_of(2) {
        return 2;
    }
    if is_prime_mr(n) {
        return n;
    }

    let mut c = 1u128;
    loop {
        let f = |x: u128| (mul_mod(x, x, n) + c) % n;
        let mut x = 2;
        let mut y = 2;
        let mut d = 1;
        while d == 1 {
            x = f(x);
            y = f(f(y));
            let diff = x.abs_diff(y);
            d = gcd(diff, n);
        }
        if d != n {
            return d;
        }
        c += 1;
    }
}

fn factor_all(n: u128, factors: &mut Vec<u128>) {
    if n == 1 {
        return;
    }
    if is_prime_mr(n) {
        factors.push(n);
        return;
    }
    let d = pollard_rho(n);
    factor_all(d, factors);
    factor_all(n / d, factors);
}

pub fn factorize(mut n: u128) -> Vec<u128> {
    let mut factors = Vec::new();
    while n.is_multiple_of(2) {
        factors.push(2);
        n /= 2;
    }
    while n.is_multiple_of(3) {
        factors.push(3);
        n /= 3;
    }
    while n.is_multiple_of(5) {
        factors.push(5);
        n /= 5;
    }
    while n.is_multiple_of(7) {
        factors.push(7);
        n /= 7;
    }
    if n > 1 {
        factor_all(n, &mut factors);
    }
    factors.sort();
    factors
}
