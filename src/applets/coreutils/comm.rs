use crate::core::{Applet, Result};
use crate::core::fs::read_bytes_or_stdin;
use std::ffi::OsString;
use std::io::{self, BufRead, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct CommApplet;
impl Applet for CommApplet {
    fn name(&self) -> &'static str {
        "comm"
    }
    fn description(&self) -> &'static str {
        "Compare two sorted files line by line"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut sup1 = false;
        let mut sup2 = false;
        let mut sup3 = false;
        let mut files = Vec::new();

        for arg in args {
            let bytes = arg.as_bytes();
            if bytes == b"-1" {
                sup1 = true;
            } else if bytes == b"-2" {
                sup2 = true;
            } else if bytes == b"-3" {
                sup3 = true;
            } else if bytes == b"-12" {
                sup1 = true;
                sup2 = true;
            } else if bytes == b"-13" {
                sup1 = true;
                sup3 = true;
            } else if bytes == b"-23" {
                sup2 = true;
                sup3 = true;
            } else if bytes == b"-123" {
                sup1 = true;
                sup2 = true;
                sup3 = true;
            } else {
                files.push(Path::new(arg));
            }
        }

        if files.len() != 2 {
            eprintln!("comm: need 2 files");
            return Ok(1);
        }

        let f1_bytes = read_bytes_or_stdin(files[0])?;
        let f2_bytes = read_bytes_or_stdin(files[1])?;

        let split_lines = |bytes: &[u8]| -> Vec<String> {
            let s = String::from_utf8_lossy(bytes);
            let mut lines = Vec::new();
            for l in s.split('\n') {
                lines.push(l.to_string());
            }
            if lines.last().map(|s| s.is_empty()).unwrap_or(false) {
                lines.pop();
            }
            lines
        };

        let l1 = split_lines(&f1_bytes);
        let l2 = split_lines(&f2_bytes);

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        let mut i1 = 0;
        let mut i2 = 0;

        let p1 = |h: &mut io::StdoutLock, s: &str| {
            if !sup1 {
                let _ = writeln!(h, "{}", s);
            }
        };
        let p2 = |h: &mut io::StdoutLock, s: &str| {
            if !sup2 {
                let prefix = if !sup1 { "\t" } else { "" };
                let _ = writeln!(h, "{}{}", prefix, s);
            }
        };
        let p3 = |h: &mut io::StdoutLock, s: &str| {
            if !sup3 {
                let prefix = match (!sup1, !sup2) {
                    (true, true) => "\t\t",
                    (true, false) | (false, true) => "\t",
                    (false, false) => "",
                };
                let _ = writeln!(h, "{}{}", prefix, s);
            }
        };

        while i1 < l1.len() && i2 < l2.len() {
            if l1[i1] == l2[i2] {
                p3(&mut handle, &l1[i1]);
                i1 += 1;
                i2 += 1;
            } else if l1[i1] < l2[i2] {
                p1(&mut handle, &l1[i1]);
                i1 += 1;
            } else {
                p2(&mut handle, &l2[i2]);
                i2 += 1;
            }
        }
        while i1 < l1.len() {
            p1(&mut handle, &l1[i1]);
            i1 += 1;
        }
        while i2 < l2.len() {
            p2(&mut handle, &l2[i2]);
            i2 += 1;
        }

        Ok(0)
    }
}
