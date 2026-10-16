use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, BufRead};
use std::os::unix::ffi::OsStrExt;

pub struct DcApplet;
impl Applet for DcApplet {
    fn name(&self) -> &'static str {
        "dc"
    }
    fn description(&self) -> &'static str {
        "Reverse-polish arbitrary precision desk calculator"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut exprs = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-e" && i + 1 < args.len() {
                exprs.push(args[i + 1].to_string_lossy().to_string());
                i += 2;
                continue;
            }
            i += 1;
        }

        let mut stack: Vec<i64> = Vec::new();

        let mut execute_tok = |tok: &str| match tok {
            "+" => {
                if stack.len() >= 2 {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a + b);
                }
            }
            "-" => {
                if stack.len() >= 2 {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a - b);
                }
            }
            "*" => {
                if stack.len() >= 2 {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a * b);
                }
            }
            "/" => {
                if stack.len() >= 2 {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    if b != 0 {
                        stack.push(a / b);
                    } else {
                        eprintln!("dc: divide by zero");
                    }
                }
            }
            "%" => {
                if stack.len() >= 2 {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    if b != 0 {
                        stack.push(a % b);
                    }
                }
            }
            "p" => {
                if let Some(top) = stack.last() {
                    println!("{}", top);
                } else {
                    eprintln!("dc: stack empty");
                }
            }
            "f" => {
                for v in stack.iter().rev() {
                    println!("{}", v);
                }
            }
            num_str => {
                if let Ok(n) = num_str.parse::<i64>() {
                    stack.push(n);
                }
            }
        };

        if !exprs.is_empty() {
            for expr in exprs {
                for tok in expr.split_whitespace() {
                    execute_tok(tok);
                }
            }
        } else {
            let stdin = io::stdin();
            for line in stdin.lock().lines().map_while(std::result::Result::ok) {
                for tok in line.split_whitespace() {
                    execute_tok(tok);
                }
            }
        }

        Ok(0)
    }
}
