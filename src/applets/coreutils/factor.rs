use crate::core::{Applet, Result};
use crate::core::digest::{BsdSum, Digest, Md5, Sha1, Sha256, Sha512, SysVSum};
use crate::core::fs::{open_or_stdin, read_bytes_or_stdin};
use super::common::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::ffi::{CString, OsStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime};
use std::env;

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
