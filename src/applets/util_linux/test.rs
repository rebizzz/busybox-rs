use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self};
use std::io::Read;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::FileTypeExt;
use std::path::Path;

struct Tok<'a> {
    v: &'a [&'a [u8]],
    i: usize,
}

impl<'a> Tok<'a> {
    fn peek(&self) -> Option<&'a [u8]> {
        self.v.get(self.i).copied()
    }
    fn next(&mut self) -> Option<&'a [u8]> {
        let t = self.v.get(self.i).copied();
        if t.is_some() {
            self.i += 1;
        }
        t
    }
}

fn eval_or(t: &mut Tok) -> std::result::Result<bool, Vec<u8>> {
    let mut v = eval_and(t)?;
    while t.peek() == Some(b"-o") {
        t.next();
        let rhs = eval_and(t)?;
        v = v || rhs;
    }
    Ok(v)
}

fn eval_and(t: &mut Tok) -> std::result::Result<bool, Vec<u8>> {
    let mut v = eval_not(t)?;
    while t.peek() == Some(b"-a") {
        t.next();
        let rhs = eval_not(t)?;
        v = v && rhs;
    }
    Ok(v)
}

fn eval_not(t: &mut Tok) -> std::result::Result<bool, Vec<u8>> {
    if t.peek() == Some(b"!") {
        t.next();
        Ok(!eval_not(t)?)
    } else {
        eval_primary(t)
    }
}

fn parse_int(s: &[u8]) -> Option<i64> {
    let s = std::str::from_utf8(s).ok()?;
    if s.is_empty() {
        return None;
    }
    let (neg, digs) = match s.as_bytes()[0] {
        b'-' => (true, &s[1..]),
        b'+' => (false, &s[1..]),
        _ => (false, s),
    };
    if digs.is_empty() || !digs.bytes().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let mut v: i64 = 0;
    for c in digs.bytes() {
        v = v.checked_mul(10)?.checked_add((c - b'0') as i64)?;
    }
    Some(if neg { -v } else { v })
}

fn file_unary(op: &[u8], path: &[u8]) -> std::result::Result<bool, Vec<u8>> {
    let p = Path::new(std::ffi::OsStr::from_bytes(path));
    let meta_follow = || fs::metadata(p);
    let meta_link = || fs::symlink_metadata(p);
    match op {
        b"-e" => Ok(meta_follow().is_ok()),
        b"-f" => Ok(meta_follow().map(|m| m.is_file()).unwrap_or(false)),
        b"-d" => Ok(meta_follow().map(|m| m.is_dir()).unwrap_or(false)),
        b"-s" => Ok(meta_follow().map(|m| m.len() > 0).unwrap_or(false)),
        b"-L" | b"-h" => Ok(meta_link()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)),
        b"-c" => Ok(meta_follow()
            .map(|m| m.file_type().is_char_device())
            .unwrap_or(false)),
        b"-b" => Ok(meta_follow()
            .map(|m| m.file_type().is_block_device())
            .unwrap_or(false)),
        b"-p" => Ok(meta_follow()
            .map(|m| m.file_type().is_fifo())
            .unwrap_or(false)),
        b"-S" => Ok(meta_follow()
            .map(|m| m.file_type().is_socket())
            .unwrap_or(false)),
        b"-r" | b"-w" | b"-x" => {
            let c = match CString::new(path) {
                Ok(c) => c,
                Err(_) => return Ok(false),
            };
            let mode = if op == b"-r" {
                libc::R_OK
            } else if op == b"-w" {
                libc::W_OK
            } else {
                libc::X_OK
            };

            Ok(unsafe { libc::access(c.as_ptr(), mode) } == 0)
        }
        b"-t" => {
            let fd: libc::c_int = std::str::from_utf8(path)
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(-1);
            if fd < 0 {
                return Ok(false);
            }

            Ok(unsafe { libc::isatty(fd) } == 1)
        }
        _ => Err(b"unknown unary operator".to_vec()),
    }
}

fn is_unary(op: &[u8]) -> bool {
    matches!(
        op,
        b"-e"
            | b"-f"
            | b"-d"
            | b"-r"
            | b"-w"
            | b"-x"
            | b"-s"
            | b"-L"
            | b"-h"
            | b"-c"
            | b"-b"
            | b"-p"
            | b"-S"
            | b"-n"
            | b"-z"
            | b"-t"
    )
}

fn is_binop(op: &[u8]) -> bool {
    matches!(
        op,
        b"=" | b"=="
            | b"!="
            | b"-eq"
            | b"-ne"
            | b"-gt"
            | b"-ge"
            | b"-lt"
            | b"-le"
            | b"-nt"
            | b"-ot"
            | b"-ef"
    )
}

fn eval_binary(a: &[u8], op: &[u8], c: &[u8]) -> std::result::Result<bool, Vec<u8>> {
    match op {
        b"=" | b"==" => Ok(a == c),
        b"!=" => Ok(a != c),
        b"-eq" | b"-ne" | b"-gt" | b"-ge" | b"-lt" | b"-le" => {
            let x = parse_int(a).ok_or_else(|| b"invalid integer".to_vec())?;
            let y = parse_int(c).ok_or_else(|| b"invalid integer".to_vec())?;
            Ok(match op {
                b"-eq" => x == y,
                b"-ne" => x != y,
                b"-gt" => x > y,
                b"-ge" => x >= y,
                b"-lt" => x < y,
                _ => x <= y,
            })
        }
        b"-nt" | b"-ot" | b"-ef" => {
            let pa = Path::new(std::ffi::OsStr::from_bytes(a));
            let pc = Path::new(std::ffi::OsStr::from_bytes(c));
            let ma = fs::metadata(pa).ok();
            let mc = fs::metadata(pc).ok();
            Ok(match op {
                b"-ef" => match (ma, mc) {
                    (Some(x), Some(y)) => {
                        use std::os::unix::fs::MetadataExt;
                        x.dev() == y.dev() && x.ino() == y.ino()
                    }
                    _ => false,
                },
                _ => {
                    let ta = ma.and_then(|m| m.modified().ok());
                    let tc = mc.and_then(|m| m.modified().ok());
                    match (ta, tc) {
                        (Some(x), Some(y)) => {
                            if op == b"-nt" {
                                x > y
                            } else {
                                x < y
                            }
                        }
                        _ => false,
                    }
                }
            })
        }
        _ => Err(b"unknown binary operator".to_vec()),
    }
}

fn eval_primary(t: &mut Tok) -> std::result::Result<bool, Vec<u8>> {
    match t.next() {
        None => Err(b"missing argument".to_vec()),
        Some(b"(") => {
            let v = eval_or(t)?;
            match t.next() {
                Some(b")") => Ok(v),
                _ => Err(b"missing ')'".to_vec()),
            }
        }
        Some(a) => {
            if is_unary(a) {
                let arg = t.next().ok_or_else(|| b"argument expected".to_vec())?;
                if a == b"-n" {
                    return Ok(!arg.is_empty());
                }
                if a == b"-z" {
                    return Ok(arg.is_empty());
                }
                return file_unary(a, arg);
            }

            if let Some(op) = t.peek() {
                if is_binop(op) {
                    t.next();
                    let c = t.next().ok_or_else(|| b"argument expected".to_vec())?;
                    return eval_binary(a, op, c);
                }
            }

            if is_unary(a)
                || is_binop(a)
                || a == b"!"
                || a == b"("
                || a == b")"
                || a == b"-a"
                || a == b"-o"
            {
                return Err(b"argument expected".to_vec());
            }
            Ok(!a.is_empty())
        }
    }
}

pub(crate) fn run_test(name: &str, args: &[OsString]) -> Result<i32> {
    let raw: Vec<&[u8]> = args.iter().map(|a| a.as_bytes()).collect();
    let expr: &[&[u8]] = if name == "[" {
        if raw.is_empty() || raw[raw.len() - 1] != b"]" {
            eprintln!("[: missing ']'");
            return Ok(2);
        }
        &raw[..raw.len() - 1]
    } else {
        &raw
    };
    if expr.is_empty() {
        return Ok(1);
    }
    let mut t = Tok { v: expr, i: 0 };
    match eval_or(&mut t) {
        Ok(v) => {
            if t.i != expr.len() {
                eprintln!("{}: too many arguments", name);
                return Ok(2);
            }
            Ok(if v { 0 } else { 1 })
        }
        Err(msg) => {
            eprintln!("{}: {}", name, String::from_utf8_lossy(&msg));
            Ok(2)
        }
    }
}

pub struct TestApplet;
impl Applet for TestApplet {
    fn name(&self) -> &'static str {
        "test"
    }
    fn description(&self) -> &'static str {
        "Evaluate a conditional expression"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_test("test", args)
    }
}
