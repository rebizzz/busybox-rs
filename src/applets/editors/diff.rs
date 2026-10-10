use crate::core::fs::open_or_stdin;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct DiffApplet;

#[derive(Clone, Debug, PartialEq, Eq)]
struct FileContent {
    lines: Vec<Vec<u8>>,
    has_trailing_newline: bool,
}

impl Applet for DiffApplet {
    fn name(&self) -> &'static str {
        "diff"
    }
    fn description(&self) -> &'static str {
        "Compare files line by line"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut unified = false;
        let mut brief = false;
        let mut ignore_space_change = false;
        let mut ignore_blank_lines = false;
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b.starts_with(b"-") && b.len() > 1 && b != b"-" && !b.starts_with(b"--") {
                for &ch in &b[1..] {
                    match ch {
                        b'u' => unified = true,
                        b'q' => brief = true,
                        b'b' => ignore_space_change = true,
                        b'B' => ignore_blank_lines = true,
                        b'r' => {}
                        b'N' => {}
                        _ => {}
                    }
                }
            } else if b == b"--brief" {
                brief = true;
            } else {
                files.push(Path::new(&args[i]));
            }
            i += 1;
        }

        if files.len() != 2 {
            eprintln!("diff: missing 2 file operands");
            return Ok(2);
        }

        if files[0] == Path::new("-") && files[1] == Path::new("-") {
            return Ok(0);
        }

        fn read_file(p: &Path) -> Result<FileContent> {
            let mut buf = Vec::new();
            if p == Path::new("-") {
                io::stdin().read_to_end(&mut buf)?;
            } else {
                let mut f = open_or_stdin(p)?;
                f.read_to_end(&mut buf)?;
            }

            if buf.is_empty() {
                return Ok(FileContent {
                    lines: Vec::new(),
                    has_trailing_newline: true,
                });
            }

            let has_trailing_newline = buf.last() == Some(&b'\n');
            let mut lines = Vec::new();
            for slice in buf.split(|&c| c == b'\n') {
                lines.push(slice.to_vec());
            }
            if has_trailing_newline && lines.last().map_or(false, |l| l.is_empty()) {
                lines.pop();
            }

            Ok(FileContent {
                lines,
                has_trailing_newline,
            })
        }

        let fc1 = match read_file(files[0]) {
            Ok(fc) => fc,
            Err(e) => {
                eprintln!("diff: {}: {}", files[0].display(), e);
                return Ok(2);
            }
        };
        let fc2 = match read_file(files[1]) {
            Ok(fc) => fc,
            Err(e) => {
                eprintln!("diff: {}: {}", files[1].display(), e);
                return Ok(2);
            }
        };

        let l1 = &fc1.lines;
        let l2 = &fc2.lines;

        let norm_b = |line: &[u8]| -> Vec<u8> {
            let mut out = Vec::new();
            let mut in_space = false;
            for &c in line {
                if c == b' ' || c == b'\t' || c == b'\r' {
                    if !in_space {
                        out.push(b' ');
                        in_space = true;
                    }
                } else {
                    out.push(c);
                    in_space = false;
                }
            }
            while out.last() == Some(&b' ') {
                out.pop();
            }
            out
        };

        let is_blank = |line: &[u8]| -> bool {
            line.iter().all(|&c| c == b' ' || c == b'\t' || c == b'\r')
        };

        let line_eq = |a: &[u8], b: &[u8]| -> bool {
            if ignore_space_change {
                norm_b(a) == norm_b(b)
            } else {
                a == b
            }
        };

        // LCS computation
        let n = l1.len();
        let m = l2.len();
        let mut dp = vec![vec![0u32; m + 1]; n + 1];

        for i in 0..n {
            for j in 0..m {
                if line_eq(&l1[i], &l2[j]) {
                    dp[i + 1][j + 1] = dp[i][j] + 1;
                } else {
                    dp[i + 1][j + 1] = dp[i + 1][j].max(dp[i][j + 1]);
                }
            }
        }

        let mut ops = Vec::new();
        let mut curr_i = n;
        let mut curr_j = m;
        while curr_i > 0 || curr_j > 0 {
            if curr_i > 0 && curr_j > 0 && line_eq(&l1[curr_i - 1], &l2[curr_j - 1]) {
                ops.push((' ', curr_i - 1, curr_j - 1));
                curr_i -= 1;
                curr_j -= 1;
            } else if curr_j > 0 && (curr_i == 0 || dp[curr_i][curr_j - 1] >= dp[curr_i - 1][curr_j]) {
                ops.push(('+', curr_i, curr_j - 1));
                curr_j -= 1;
            } else if curr_i > 0 {
                ops.push(('-', curr_i - 1, curr_j));
                curr_i -= 1;
            }
        }
        ops.reverse();

        // Check if there are differences
        let has_real_diff = ops.iter().any(|(op, idx1, idx2)| {
            if *op == ' ' {
                return false;
            }
            if ignore_blank_lines {
                if *op == '-' && is_blank(&l1[*idx1]) {
                    return false;
                }
                if *op == '+' && is_blank(&l2[*idx2]) {
                    return false;
                }
            }
            true
        }) || (fc1.has_trailing_newline != fc2.has_trailing_newline && !l1.is_empty() && !l2.is_empty());

        if !has_real_diff {
            return Ok(0);
        }

        if brief {
            println!("Files {} and {} differ", files[0].display(), files[1].display());
            return Ok(1);
        }

        let out = io::stdout();
        let mut lock = out.lock();

        if unified {
            writeln!(lock, "--- {}", files[0].display())?;
            writeln!(lock, "+++ {}", files[1].display())?;

            let r1 = if l1.is_empty() {
                "0,0".to_string()
            } else if l1.len() == 1 {
                "1".to_string()
            } else {
                format!("1,{}", l1.len())
            };

            let r2 = if l2.is_empty() {
                "0,0".to_string()
            } else if l2.len() == 1 {
                "1".to_string()
            } else {
                format!("1,{}", l2.len())
            };

            writeln!(lock, "@@ -{} +{} @@", r1, r2)?;

            for &(op, idx1, idx2) in &ops {
                match op {
                    ' ' => {
                        write!(lock, " ")?;
                        lock.write_all(&l1[idx1])?;
                        writeln!(lock)?;
                    }
                    '-' => {
                        write!(lock, "-")?;
                        lock.write_all(&l1[idx1])?;
                        writeln!(lock)?;
                        if idx1 + 1 == l1.len() && !fc1.has_trailing_newline {
                            writeln!(lock, "\\ No newline at end of file")?;
                        }
                    }
                    '+' => {
                        write!(lock, "+")?;
                        lock.write_all(&l2[idx2])?;
                        writeln!(lock)?;
                        if idx2 + 1 == l2.len() && !fc2.has_trailing_newline {
                            writeln!(lock, "\\ No newline at end of file")?;
                        }
                    }
                    _ => {}
                }
            }
        } else {
            // Traditional diff format
            let mut k = 0;
            while k < ops.len() {
                if ops[k].0 == ' ' {
                    k += 1;
                    continue;
                }
                let mut del_start = None;
                let mut del_count = 0;
                let mut add_start = None;
                let mut add_count = 0;

                let mut del_lines = Vec::new();
                let mut add_lines = Vec::new();

                while k < ops.len() && ops[k].0 != ' ' {
                    if ops[k].0 == '-' {
                        let i = ops[k].1;
                        if del_start.is_none() {
                            del_start = Some(i + 1);
                        }
                        del_count += 1;
                        del_lines.push(&l1[i]);
                    } else if ops[k].0 == '+' {
                        let j = ops[k].2;
                        if add_start.is_none() {
                            add_start = Some(j + 1);
                        }
                        add_count += 1;
                        add_lines.push(&l2[j]);
                    }
                    k += 1;
                }

                if del_count > 0 && add_count > 0 {
                    let d_str = if del_count == 1 {
                        format!("{}", del_start.unwrap())
                    } else {
                        format!("{},{}", del_start.unwrap(), del_start.unwrap() + del_count - 1)
                    };
                    let a_str = if add_count == 1 {
                        format!("{}", add_start.unwrap())
                    } else {
                        format!("{},{}", add_start.unwrap(), add_start.unwrap() + add_count - 1)
                    };
                    writeln!(lock, "{}c{}", d_str, a_str)?;
                    for line in del_lines {
                        write!(lock, "< ")?;
                        lock.write_all(line)?;
                        writeln!(lock)?;
                    }
                    writeln!(lock, "---")?;
                    for line in add_lines {
                        write!(lock, "> ")?;
                        lock.write_all(line)?;
                        writeln!(lock)?;
                    }
                } else if del_count > 0 {
                    let a_pos = if k < ops.len() { ops[k].2 } else { m };
                    let d_str = if del_count == 1 {
                        format!("{}", del_start.unwrap())
                    } else {
                        format!("{},{}", del_start.unwrap(), del_start.unwrap() + del_count - 1)
                    };
                    writeln!(lock, "{}d{}", d_str, a_pos)?;
                    for line in del_lines {
                        write!(lock, "< ")?;
                        lock.write_all(line)?;
                        writeln!(lock)?;
                    }
                } else if add_count > 0 {
                    let d_pos = if k < ops.len() { ops[k].1 } else { n };
                    let a_str = if add_count == 1 {
                        format!("{}", add_start.unwrap())
                    } else {
                        format!("{},{}", add_start.unwrap(), add_start.unwrap() + add_count - 1)
                    };
                    writeln!(lock, "{}a{}", d_pos, a_str)?;
                    for line in add_lines {
                        write!(lock, "> ")?;
                        lock.write_all(line)?;
                        writeln!(lock)?;
                    }
                }
            }
        }

        Ok(1)
    }
}
