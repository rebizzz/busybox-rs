use std::ffi::OsString;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use crate::core::fs::{open_or_stdin, read_bytes_or_stdin};
use crate::core::{Applet, Result};

pub struct CatApplet;
impl Applet for CatApplet {
    fn name(&self) -> &'static str { "cat" }
    fn description(&self) -> &'static str { "Concatenate FILE(s) and print on standard output" }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut show_ends = false;
        let mut number_all = false;
        let mut number_nonblank = false;
        let mut show_nonprinting = false;
        let mut files = Vec::new();

        for arg in args {
            let bytes = arg.as_bytes();
            if bytes.starts_with(b"-") && bytes.len() > 1 && bytes != b"-" {
                for &b in &bytes[1..] {
                    match b {
                        b'e' => { show_ends = true; show_nonprinting = true; }
                        b'E' => { show_ends = true; }
                        b'n' => { number_all = true; }
                        b'b' => { number_nonblank = true; }
                        b'v' => { show_nonprinting = true; }
                        _ => {}
                    }
                }
            } else {
                files.push(Path::new(arg));
            }
        }

        if files.is_empty() {
            files.push(Path::new("-"));
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();
        let mut line_num = 1;

        for file in files {
            let reader = open_or_stdin(file)?;
            let mut buf_reader = BufReader::new(reader);

            if !show_ends && !number_all && !number_nonblank && !show_nonprinting {
                io::copy(&mut buf_reader, &mut handle)?;
                continue;
            }

            let mut line_buf = Vec::new();
            while let Ok(n) = buf_reader.read_until(b'\n', &mut line_buf) {
                if n == 0 { break; }
                let is_blank = line_buf.len() == 1 && line_buf[0] == b'\n';
                if number_nonblank {
                    if !is_blank {
                        write!(handle, "{:6}\t", line_num)?;
                        line_num += 1;
                    }
                } else if number_all {
                    write!(handle, "{:6}\t", line_num)?;
                    line_num += 1;
                }

                for &b in &line_buf {
                    if b == b'\n' {
                        if show_ends { handle.write_all(b"$")?; }
                        handle.write_all(b"\n")?;
                    } else if show_nonprinting {
                        if b < 32 && b != b'\t' {
                            handle.write_all(&[b'^', b + 64])?;
                        } else if b == 127 {
                            handle.write_all(b"^?")?;
                        } else if b >= 128 && b < 160 {
                            handle.write_all(&[b'M', b'-', b'^', b - 128 + 64])?;
                        } else if b >= 160 && b < 255 {
                            handle.write_all(&[b'M', b'-', b - 128])?;
                        } else if b == 255 {
                            handle.write_all(b"M-^?")?;
                        } else {
                            handle.write_all(&[b])?;
                        }
                    } else {
                        handle.write_all(&[b])?;
                    }
                }
                line_buf.clear();
            }
        }
        Ok(0)
    }
}

pub struct HeadApplet;
impl Applet for HeadApplet {
    fn name(&self) -> &'static str { "head" }
    fn description(&self) -> &'static str { "Output the first part of files" }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut lines: i64 = 10;
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();
            if bytes == b"-n" {
                if i + 1 < args.len() {
                    lines = args[i + 1].to_string_lossy().parse().unwrap_or(10);
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-n") {
                lines = arg.to_string_lossy()[2..].parse().unwrap_or(10);
            } else if bytes.starts_with(b"-") && bytes.len() > 1 && bytes[1..].iter().all(|b| b.is_ascii_digit()) {
                lines = arg.to_string_lossy()[1..].parse().unwrap_or(10);
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
            let reader = open_or_stdin(file)?;
            let buf_reader = BufReader::new(reader);

            if lines >= 0 {
                let mut count = 0;
                for l in buf_reader.lines() {
                    if count >= lines { break; }
                    if let Ok(line) = l {
                        writeln!(handle, "{}", line)?;
                        count += 1;
                    }
                }
            } else {
                let all_lines: Vec<String> = buf_reader.lines().filter_map(std::result::Result::ok).collect();
                let keep = all_lines.len().saturating_sub((-lines) as usize);
                for l in &all_lines[..keep] {
                    writeln!(handle, "{}", l)?;
                }
            }
        }
        Ok(0)
    }
}

pub struct WcApplet;
impl Applet for WcApplet {
    fn name(&self) -> &'static str { "wc" }
    fn description(&self) -> &'static str { "Print newline, word, and byte counts" }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut count_lines = false;
        let mut count_words = false;
        let mut count_chars = false;
        let mut max_line = false;
        let mut files = Vec::new();

        for arg in args {
            let bytes = arg.as_bytes();
            if bytes.starts_with(b"-") && bytes.len() > 1 && bytes != b"-" {
                for &b in &bytes[1..] {
                    match b {
                        b'l' => count_lines = true,
                        b'w' => count_words = true,
                        b'c' | b'm' => count_chars = true,
                        b'L' => max_line = true,
                        _ => {}
                    }
                }
            } else {
                files.push(Path::new(arg));
            }
        }

        if !count_lines && !count_words && !count_chars && !max_line {
            count_lines = true;
            count_words = true;
            count_chars = true;
        }

        if files.is_empty() {
            files.push(Path::new("-"));
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        for file in &files {
            let content = read_bytes_or_stdin(file)?;
            let mut lines = 0usize;
            let mut words = 0usize;
            let chars = content.len();
            let mut longest = 0usize;
            let mut cur_line_len = 0usize;
            let mut in_word = false;

            for &b in &content {
                if b == b'\n' {
                    lines += 1;
                    if cur_line_len > longest { longest = cur_line_len; }
                    cur_line_len = 0;
                } else {
                    cur_line_len += 1;
                }

                if b.is_ascii_whitespace() {
                    in_word = false;
                } else if !in_word {
                    words += 1;
                    in_word = true;
                }
            }
            if cur_line_len > longest { longest = cur_line_len; }

            let mut parts = Vec::new();
            if count_lines { parts.push(format!("{}", lines)); }
            if count_words { parts.push(format!("{}", words)); }
            if count_chars { parts.push(format!("{}", chars)); }
            if max_line { parts.push(format!("{}", longest)); }

            if file.as_os_str() != "-" {
                parts.push(file.display().to_string());
            }
            writeln!(handle, "{}", parts.join(" "))?;
        }
        Ok(0)
    }
}

pub struct TailApplet;
impl Applet for TailApplet {
    fn name(&self) -> &'static str { "tail" }
    fn description(&self) -> &'static str { "Output the last part of files" }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut count: usize = 10;
        let mut from_beginning = false;
        let mut byte_mode = false;
        let mut quiet = false;
        let mut verbose = false;
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();
            if bytes.starts_with(b"-c") {
                byte_mode = true;
                let val_str = if bytes.len() > 2 {
                    &bytes[2..]
                } else if i + 1 < args.len() {
                    i += 1;
                    args[i].as_bytes()
                } else {
                    b""
                };
                if val_str.starts_with(b"+") {
                    from_beginning = true;
                    count = String::from_utf8_lossy(&val_str[1..]).parse().unwrap_or(1);
                } else {
                    from_beginning = false;
                    let s = if val_str.starts_with(b"-") { &val_str[1..] } else { val_str };
                    count = String::from_utf8_lossy(s).parse().unwrap_or(10);
                }
            } else if bytes.starts_with(b"-n") {
                byte_mode = false;
                let val_str = if bytes.len() > 2 {
                    &bytes[2..]
                } else if i + 1 < args.len() {
                    i += 1;
                    args[i].as_bytes()
                } else {
                    b""
                };
                if val_str.starts_with(b"+") {
                    from_beginning = true;
                    count = String::from_utf8_lossy(&val_str[1..]).parse().unwrap_or(1);
                } else {
                    from_beginning = false;
                    let s = if val_str.starts_with(b"-") { &val_str[1..] } else { val_str };
                    count = String::from_utf8_lossy(s).parse().unwrap_or(10);
                }
            } else if bytes.starts_with(b"-") && bytes.len() > 1 && bytes[1..].iter().all(|b| b.is_ascii_digit()) {
                byte_mode = false;
                from_beginning = false;
                count = String::from_utf8_lossy(&bytes[1..]).parse().unwrap_or(10);
            } else if bytes.starts_with(b"+") && bytes.len() > 1 && bytes[1..].iter().all(|b| b.is_ascii_digit()) {
                byte_mode = false;
                from_beginning = true;
                count = String::from_utf8_lossy(&bytes[1..]).parse().unwrap_or(1);
            } else if bytes == b"-q" {
                quiet = true;
            } else if bytes == b"-v" {
                verbose = true;
            } else if bytes == b"--" {
                i += 1;
                while i < args.len() {
                    files.push(Path::new(&args[i]));
                    i += 1;
                }
                break;
            } else if bytes.starts_with(b"-") && bytes != b"-" {
                // Ignore unknown flags or handle them
            } else {
                files.push(Path::new(arg));
            }
            i += 1;
        }

        if files.is_empty() {
            files.push(Path::new("-"));
        }

        let print_headers = (files.len() > 1 && !quiet) || verbose;
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        for (idx, file) in files.iter().enumerate() {
            let content = match read_bytes_or_stdin(file) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("tail: cannot open '{}': {}", file.display(), e);
                    continue;
                }
            };

            if print_headers {
                if idx > 0 {
                    handle.write_all(b"\n")?;
                }
                writeln!(handle, "==> {} <==", file.display())?;
            }

            if byte_mode {
                if from_beginning {
                    let start = count.saturating_sub(1);
                    if start < content.len() {
                        handle.write_all(&content[start..])?;
                    }
                } else {
                    let start = content.len().saturating_sub(count);
                    handle.write_all(&content[start..])?;
                }
            } else {
                if from_beginning {
                    let mut line_no = 1;
                    let mut offset = 0;
                    for (i, &b) in content.iter().enumerate() {
                        if line_no >= count {
                            offset = i;
                            break;
                        }
                        if b == b'\n' {
                            line_no += 1;
                            if line_no >= count {
                                offset = i + 1;
                                break;
                            }
                        }
                    }
                    if line_no >= count && offset < content.len() {
                        handle.write_all(&content[offset..])?;
                    }
                } else {
                    if count == 0 {
                        continue;
                    }
                    let mut newlines = 0;
                    let mut start = 0;
                    let mut found = false;
                    for (i, &b) in content.iter().enumerate().rev() {
                        if b == b'\n' {
                            if i + 1 == content.len() {
                                continue;
                            }
                            newlines += 1;
                            if newlines == count {
                                start = i + 1;
                                found = true;
                                break;
                            }
                        }
                    }
                    if !found {
                        start = 0;
                    }
                    handle.write_all(&content[start..])?;
                }
            }
        }
        Ok(0)
    }
}
