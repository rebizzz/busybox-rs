use std::ffi::OsString;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use crate::core::fs::{open_or_stdin, read_bytes_or_stdin};
use crate::core::{Applet, Result};


pub struct TeeApplet;
impl Applet for TeeApplet {
    fn name(&self) -> &'static str { "tee" }
    fn description(&self) -> &'static str { "Copy standard input to each FILE and standard output" }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut append = false;
        let mut files = Vec::new();

        for arg in args {
            let bytes = arg.as_bytes();
            if bytes == b"-a" { append = true; }
            else if bytes == b"-i" { }
            else if !bytes.starts_with(b"-") {
                files.push(Path::new(arg));
            }
        }

        let mut handles = Vec::new();
        for f in &files {
            let file = if append {
                std::fs::OpenOptions::new().create(true).append(true).open(f)?
            } else {
                std::fs::OpenOptions::new().create(true).write(true).truncate(true).open(f)?
            };
            handles.push(file);
        }

        let stdin = io::stdin();
        let mut stdin_handle = stdin.lock();
        let stdout = io::stdout();
        let mut stdout_handle = stdout.lock();

        let mut buf = [0u8; 8192];
        loop {
            let n = match stdin_handle.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => return Err(e.into()),
            };
            stdout_handle.write_all(&buf[..n])?;
            for h in &mut handles {
                h.write_all(&buf[..n])?;
            }
        }
        Ok(0)
    }
}

pub struct StringsApplet;
impl Applet for StringsApplet {
    fn name(&self) -> &'static str { "strings" }
    fn description(&self) -> &'static str { "Find and print letter sequences in binary files" }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut min_len = 4;
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();
            if bytes == b"-n" {
                if i + 1 < args.len() {
                    min_len = args[i + 1].to_string_lossy().parse().unwrap_or(4);
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-n") {
                min_len = arg.to_string_lossy()[2..].parse().unwrap_or(4);
            } else if bytes.starts_with(b"-") && bytes.len() > 1 && bytes[1..].iter().all(|b| b.is_ascii_digit()) {
                min_len = arg.to_string_lossy()[1..].parse().unwrap_or(4);
            } else {
                files.push(Path::new(arg));
            }
            i += 1;
        }

        if files.is_empty() {
            files.push(Path::new("-"));
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        for file in files {
            let content = read_bytes_or_stdin(file)?;
            let mut cur = Vec::new();
            for &b in &content {
                if (b >= 32 && b < 127) || b == b'\t' {
                    cur.push(b);
                } else {
                    if cur.len() >= min_len {
                        handle.write_all(&cur)?;
                        handle.write_all(b"\n")?;
                    }
                    cur.clear();
                }
            }
            if cur.len() >= min_len {
                handle.write_all(&cur)?;
                handle.write_all(b"\n")?;
            }
        }
        Ok(0)
    }
}

pub struct CommApplet;
impl Applet for CommApplet {
    fn name(&self) -> &'static str { "comm" }
    fn description(&self) -> &'static str { "Compare two sorted files line by line" }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut sup1 = false;
        let mut sup2 = false;
        let mut sup3 = false;
        let mut files = Vec::new();

        for arg in args {
            let bytes = arg.as_bytes();
            if bytes == b"-1" { sup1 = true; }
            else if bytes == b"-2" { sup2 = true; }
            else if bytes == b"-3" { sup3 = true; }
            else if bytes == b"-12" { sup1 = true; sup2 = true; }
            else if bytes == b"-13" { sup1 = true; sup3 = true; }
            else if bytes == b"-23" { sup2 = true; sup3 = true; }
            else if bytes == b"-123" { sup1 = true; sup2 = true; sup3 = true; }
            else { files.push(Path::new(arg)); }
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
            for l in s.split('\n') { lines.push(l.to_string()); }
            if lines.last().map(|s| s.is_empty()).unwrap_or(false) { lines.pop(); }
            lines
        };

        let l1 = split_lines(&f1_bytes);
        let l2 = split_lines(&f2_bytes);

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        let mut i1 = 0;
        let mut i2 = 0;

        let p1 = |h: &mut io::StdoutLock, s: &str| {
            if !sup1 { let _ = writeln!(h, "{}", s); }
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
        while i1 < l1.len() { p1(&mut handle, &l1[i1]); i1 += 1; }
        while i2 < l2.len() { p2(&mut handle, &l2[i2]); i2 += 1; }

        Ok(0)
    }
}

pub struct UniqApplet;
impl Applet for UniqApplet {
    fn name(&self) -> &'static str { "uniq" }
    fn description(&self) -> &'static str { "Report or omit repeated lines" }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut count_flag = false;
        let mut dups_only = false;
        let mut unique_only = false;
        let mut skip_fields = 0usize;
        let mut skip_chars = 0usize;
        let mut check_chars = usize::MAX;
        let mut pos_args = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();
            if bytes == b"-c" { count_flag = true; }
            else if bytes == b"-d" { dups_only = true; }
            else if bytes == b"-u" { unique_only = true; }
            else if bytes == b"-f" {
                if i + 1 < args.len() {
                    skip_fields = args[i + 1].to_string_lossy().parse().unwrap_or(0);
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-f") {
                skip_fields = arg.to_string_lossy()[2..].parse().unwrap_or(0);
            } else if bytes == b"-s" {
                if i + 1 < args.len() {
                    skip_chars = args[i + 1].to_string_lossy().parse().unwrap_or(0);
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-s") {
                skip_chars = arg.to_string_lossy()[2..].parse().unwrap_or(0);
            } else if bytes == b"-w" {
                if i + 1 < args.len() {
                    check_chars = args[i + 1].to_string_lossy().parse().unwrap_or(usize::MAX);
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-w") {
                check_chars = arg.to_string_lossy()[2..].parse().unwrap_or(usize::MAX);
            } else {
                pos_args.push(Path::new(arg));
            }
            i += 1;
        }

        let (infile, outfile) = match pos_args.len() {
            0 => (Path::new("-"), Path::new("-")),
            1 => (pos_args[0], Path::new("-")),
            2 => (pos_args[0], pos_args[1]),
            _ => {
                eprintln!("uniq: extra operand");
                return Ok(1);
            }
        };

        let content = read_bytes_or_stdin(infile)?;
        let mut lines = Vec::new();
        let mut cur = Vec::new();
        for &b in &content {
            if b == b'\n' {
                lines.push(String::from_utf8_lossy(&cur).to_string());
                cur.clear();
            } else {
                cur.push(b);
            }
        }
        if !cur.is_empty() {
            lines.push(String::from_utf8_lossy(&cur).to_string());
        }

        let extract_key = |s: &str| -> String {
            let mut cur = s;
            for _ in 0..skip_fields {
                let trimmed = cur.trim_start_matches(|c: char| c.is_ascii_whitespace());
                if let Some(pos) = trimmed.find(|c: char| c.is_ascii_whitespace()) {
                    cur = &trimmed[pos..];
                } else {
                    cur = "";
                    break;
                }
            }
            let chars: Vec<char> = cur.chars().collect();
            chars.into_iter().skip(skip_chars).take(check_chars).collect()
        };

        let mut groups: Vec<(usize, String)> = Vec::new();
        for line in lines {
            let key = extract_key(&line);
            if let Some((cnt, last_line)) = groups.last_mut() {
                if extract_key(last_line) == key {
                    *cnt += 1;
                    continue;
                }
            }
            groups.push((1, line));
        }

        let mut out: Box<dyn Write> = if outfile.as_os_str() == "-" {
            Box::new(io::stdout())
        } else {
            Box::new(std::fs::File::create(outfile)?)
        };

        for (cnt, line) in groups {
            if dups_only && cnt == 1 { continue; }
            if unique_only && cnt > 1 { continue; }
            if count_flag {
                writeln!(out, "{:7} {}", cnt, line)?;
            } else {
                writeln!(out, "{}", line)?;
            }
        }
        Ok(0)
    }
}
