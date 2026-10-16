use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;

pub struct ExprApplet;

impl Applet for ExprApplet {
    fn name(&self) -> &'static str {
        "expr"
    }
    fn description(&self) -> &'static str {
        "Evaluate expressions"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("expr: missing operand");
            return Ok(2);
        }

        let tokens: Vec<&[u8]> = args.iter().map(|s| s.as_bytes()).collect();

        fn parse_int(s: &[u8]) -> Option<i64> {
            std::str::from_utf8(s).ok()?.parse::<i64>().ok()
        }

        fn eval(tokens: &[&[u8]]) -> std::result::Result<Vec<u8>, i32> {
            if tokens.is_empty() {
                return Err(2);
            }
            if tokens.len() == 1 {
                return Ok(tokens[0].to_vec());
            }

            let ops_order: &[&[&[u8]]] = &[
                &[b"|"],
                &[b"&"],
                &[b"<", b"<=", b"=", b"!=", b">=", b">"],
                &[b"+", b"-"],
                &[b"*", b"/", b"%"],
            ];

            for ops in ops_order {
                for idx in (1..tokens.len() - 1).rev() {
                    let op = tokens[idx];
                    if ops.contains(&op) {
                        let left = eval(&tokens[..idx])?;
                        let right = eval(&tokens[idx + 1..])?;

                        let left_str = std::str::from_utf8(&left).unwrap_or("");
                        let right_str = std::str::from_utf8(&right).unwrap_or("");
                        let left_num = parse_int(&left);
                        let right_num = parse_int(&right);

                        match op {
                            b"|" => {
                                let is_zero_or_null = left.is_empty() || left == b"0";
                                return if !is_zero_or_null {
                                    Ok(left)
                                } else {
                                    let right_zero_or_null = right.is_empty() || right == b"0";
                                    if !right_zero_or_null {
                                        Ok(right)
                                    } else {
                                        Ok(b"0".to_vec())
                                    }
                                };
                            }
                            b"&" => {
                                let l_null = left.is_empty() || left == b"0";
                                let r_null = right.is_empty() || right == b"0";
                                return if !l_null && !r_null {
                                    Ok(left)
                                } else {
                                    Ok(b"0".to_vec())
                                };
                            }
                            b"=" | b"!=" | b"<" | b"<=" | b">" | b">=" => {
                                let cmp = if let (Some(a), Some(b)) = (left_num, right_num) {
                                    a.cmp(&b)
                                } else {
                                    left_str.cmp(right_str)
                                };
                                let res = match op {
                                    b"=" => cmp.is_eq(),
                                    b"!=" => cmp.is_ne(),
                                    b"<" => cmp.is_lt(),
                                    b"<=" => cmp.is_le(),
                                    b">" => cmp.is_gt(),
                                    b">=" => cmp.is_ge(),
                                    _ => false,
                                };
                                return Ok(if res { b"1".to_vec() } else { b"0".to_vec() });
                            }
                            b"+" | b"-" | b"*" | b"/" | b"%" => {
                                let (a, b) = match (left_num, right_num) {
                                    (Some(a), Some(b)) => (a, b),
                                    _ => return Err(2),
                                };
                                let res = match op {
                                    b"+" => a + b,
                                    b"-" => a - b,
                                    b"*" => a * b,
                                    b"/" => {
                                        if b == 0 {
                                            eprintln!("expr: division by zero");
                                            return Err(2);
                                        }
                                        a / b
                                    }
                                    b"%" => {
                                        if b == 0 {
                                            eprintln!("expr: division by zero");
                                            return Err(2);
                                        }
                                        a % b
                                    }
                                    _ => 0,
                                };
                                return Ok(res.to_string().into_bytes());
                            }
                            _ => {}
                        }
                    }
                }
            }

            Ok(tokens[0].to_vec())
        }

        match eval(&tokens) {
            Ok(res) => {
                let out = io::stdout();
                let mut lock = out.lock();
                lock.write_all(&res)?;
                lock.write_all(b"\n")?;
                let is_null_or_zero = res.is_empty() || res == b"0";
                Ok(if is_null_or_zero { 1 } else { 0 })
            }
            Err(code) => Ok(code),
        }
    }
}

