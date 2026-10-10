use crate::core::fs::open_or_stdin;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct DiffApplet;

impl Applet for DiffApplet {
    fn name(&self) -> &'static str {
        "diff"
    }
    fn description(&self) -> &'static str {
        "Compare files line by line"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut unified = false;
        let mut files = Vec::new();

        for arg in args {
            let b = arg.as_bytes();
            if b == b"-u" || b.starts_with(b"-U") {
                unified = true;
            } else if b.starts_with(b"-") && b != b"-" {
            } else {
                files.push(Path::new(arg));
            }
        }

        if files.len() != 2 {
            eprintln!("diff: missing 2 file operands");
            return Ok(2);
        }

        fn read_lines(p: &Path) -> Result<Vec<Vec<u8>>> {
            let r: Box<dyn BufRead> = if p == Path::new("-") {
                Box::new(BufReader::new(io::stdin()))
            } else {
                Box::new(BufReader::new(open_or_stdin(p)?))
            };
            let mut lines = Vec::new();
            for l in r.lines() {
                lines.push(l?.into_bytes());
            }
            Ok(lines)
        }

        let l1 = match read_lines(files[0]) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("diff: {}: {}", files[0].display(), e);
                return Ok(2);
            }
        };
        let l2 = match read_lines(files[1]) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("diff: {}: {}", files[1].display(), e);
                return Ok(2);
            }
        };

        if l1 == l2 {
            return Ok(0);
        }

        let out = io::stdout();
        let mut lock = out.lock();

        if unified {
            writeln!(lock, "--- {}", files[0].display())?;
            writeln!(lock, "+++ {}", files[1].display())?;
            let r1 = if l1.len() == 1 {
                "1".to_string()
            } else {
                format!("1,{}", l1.len())
            };
            let r2 = if l2.len() == 1 {
                "1".to_string()
            } else {
                format!("1,{}", l2.len())
            };
            writeln!(lock, "@@ -{} +{} @@", r1, r2)?;

            let mut i = 0;
            let mut j = 0;
            while i < l1.len() || j < l2.len() {
                if i < l1.len() && j < l2.len() && l1[i] == l2[j] {
                    write!(lock, " ")?;
                    lock.write_all(&l1[i])?;
                    writeln!(lock)?;
                    i += 1;
                    j += 1;
                } else if i < l1.len() && (j >= l2.len() || !l2.contains(&l1[i])) {
                    write!(lock, "-")?;
                    lock.write_all(&l1[i])?;
                    writeln!(lock)?;
                    i += 1;
                } else if j < l2.len() {
                    write!(lock, "+")?;
                    lock.write_all(&l2[j])?;
                    writeln!(lock)?;
                    j += 1;
                } else {
                    i += 1;
                }
            }
        } else {
            let mut i = 0;
            let mut j = 0;
            while i < l1.len() || j < l2.len() {
                if i < l1.len() && j < l2.len() && l1[i] == l2[j] {
                    i += 1;
                    j += 1;
                } else if i < l1.len() && (j >= l2.len() || !l2.contains(&l1[i])) {
                    writeln!(lock, "{}d{}", i + 1, j)?;
                    write!(lock, "< ")?;
                    lock.write_all(&l1[i])?;
                    writeln!(lock)?;
                    i += 1;
                } else if j < l2.len() {
                    writeln!(lock, "{}a{}", i, j + 1)?;
                    write!(lock, "> ")?;
                    lock.write_all(&l2[j])?;
                    writeln!(lock)?;
                    j += 1;
                } else {
                    i += 1;
                }
            }
        }

        Ok(1)
    }
}

