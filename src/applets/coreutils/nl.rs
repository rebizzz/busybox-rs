use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct NlApplet;
impl Applet for NlApplet {
    fn name(&self) -> &'static str {
        "nl"
    }
    fn description(&self) -> &'static str {
        "Number lines of files"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut style = b't';
        let (mut incr, mut num, mut width): (i64, i64, usize) = (1, 1, 6);
        let mut sep = b"\t".to_vec();
        let mut files: Vec<&Path> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"--" {
                for a in &args[i + 1..] {
                    files.push(Path::new(a));
                }
                break;
            }
            if b.len() > 1
                && b[0] == b'-'
                && b != b"-"
                && matches!(b[1], b'b' | b'i' | b'v' | b's' | b'w')
            {
                let v = take_val(b, 1, &mut i, args);
                let s = String::from_utf8_lossy(&v);
                match b[1] {
                    b'b' => {
                        style = *v.first().unwrap_or(&b't');
                        if !matches!(style, b'a' | b't' | b'n') {
                            eprintln!("nl: invalid style");
                            return Ok(1);
                        }
                    }
                    b'i' => {
                        incr = match s.trim().parse() {
                            Ok(v) => v,
                            Err(_) => {
                                eprintln!("nl: bad -i");
                                return Ok(1);
                            }
                        }
                    }
                    b'v' => {
                        num = match s.trim().parse() {
                            Ok(v) => v,
                            Err(_) => {
                                eprintln!("nl: bad -v");
                                return Ok(1);
                            }
                        }
                    }
                    b's' => sep = v,
                    b'w' => {
                        width = match s.trim().parse() {
                            Ok(v) => v,
                            Err(_) => {
                                eprintln!("nl: bad -w");
                                return Ok(1);
                            }
                        }
                    }
                    _ => {}
                }
            } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            } else {
                files.push(Path::new(&args[i]));
            }
            i += 1;
        }
        if files.is_empty() {
            files.push(Path::new("-"));
        }
        let out = io::stdout();
        let mut w = out.lock();
        let mut buf = Vec::with_capacity(4096);
        let mut nbuf = Vec::with_capacity(32);
        for f in &files {
            let mut r = open_input(f)?;
            while next_line(&mut r, &mut buf)? {
                if style == b'a' || (style == b't' && !buf.is_empty()) {
                    nbuf.clear();
                    let mut v = num.unsigned_abs();
                    let mut digs = [0u8; 20];
                    let mut nd = 0;
                    if v == 0 {
                        digs[0] = b'0';
                        nd = 1;
                    }
                    while v > 0 {
                        digs[nd] = (v % 10) as u8 + b'0';
                        v /= 10;
                        nd += 1;
                    }
                    if num < 0 {
                        nbuf.push(b'-');
                    }
                    nbuf.extend(std::iter::repeat_n(
                        b' ',
                        width.saturating_sub(nd + (num < 0) as usize),
                    ));
                    for k in (0..nd).rev() {
                        nbuf.push(digs[k]);
                    }
                    w.write_all(&nbuf)?;
                    w.write_all(&sep)?;
                    num += incr;
                } else {
                    nbuf.clear();
                    nbuf.extend(std::iter::repeat_n(b' ', width + 1));
                    w.write_all(&nbuf)?;
                }
                w.write_all(&buf)?;
                w.write_all(b"\n")?;
            }
        }
        Ok(0)
    }
}

