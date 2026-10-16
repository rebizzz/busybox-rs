use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, BufRead, Read};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::process::Command;

pub struct XargsApplet;
impl Applet for XargsApplet {
    fn name(&self) -> &'static str {
        "xargs"
    }
    fn description(&self) -> &'static str {
        "Build and execute command lines from standard input"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut null_delim = false;
        let mut max_args: Option<usize> = None;
        let mut eof_str: Option<Vec<u8>> = None;
        let mut replace_str: Option<Vec<u8>> = None;
        let mut max_chars: Option<usize> = None;
        let mut trace = false;
        let mut cmd_parts = Vec::new();
        let mut i = 0;

        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();
            if bytes == b"-0" || bytes == b"--null" {
                null_delim = true;
            } else if bytes == b"-t" || bytes == b"--verbose" {
                trace = true;
            } else if bytes.starts_with(b"-t") {
                trace = true;
                let rest = &bytes[2..];
                if rest.starts_with(b"s") {
                    max_chars = std::str::from_utf8(&rest[1..]).ok().and_then(|s| s.parse().ok());
                }
            } else if bytes == b"-n" {
                if i + 1 < args.len() {
                    max_args = args[i + 1].to_string_lossy().parse().ok();
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-n") {
                max_args = arg.to_string_lossy()[2..].parse().ok();
            } else if bytes == b"-E" || bytes == b"-e" {
                if i + 1 < args.len() {
                    eof_str = Some(args[i + 1].as_bytes().to_vec());
                    i += 2;
                    continue;
                } else {
                    eof_str = Some(Vec::new());
                }
            } else if bytes.starts_with(b"-E") || bytes.starts_with(b"-e") {
                eof_str = Some(bytes[2..].to_vec());
            } else if bytes == b"-s" {
                if i + 1 < args.len() {
                    max_chars = args[i + 1].to_string_lossy().parse().ok();
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-s") {
                max_chars = arg.to_string_lossy()[2..].parse().ok();
            } else if bytes == b"-I" || bytes == b"-i" {
                if i + 1 < args.len() {
                    replace_str = Some(args[i + 1].as_bytes().to_vec());
                    i += 2;
                    continue;
                } else {
                    replace_str = Some(b"{}".to_vec());
                }
            } else if bytes.starts_with(b"-I") || bytes.starts_with(b"-i") {
                replace_str = Some(bytes[2..].to_vec());
            } else if bytes == b"-r" || bytes == b"--no-run-if-empty" {
            } else {
                cmd_parts.push(arg.clone());
            }
            i += 1;
        }

        if cmd_parts.is_empty() {
            cmd_parts.push(OsString::from("echo"));
        }

        let stdin = io::stdin();
        let mut stdin_handle = stdin.lock();
        let mut input_items: Vec<Vec<u8>> = Vec::new();

        if null_delim {
            let mut buf = Vec::new();
            while let Ok(n) = stdin_handle.read_until(0, &mut buf) {
                if n == 0 {
                    break;
                }
                if buf.last() == Some(&0) {
                    buf.pop();
                }
                if let Some(ref eof) = eof_str {
                    if &buf == eof {
                        break;
                    }
                }
                if !buf.is_empty() {
                    input_items.push(std::mem::take(&mut buf));
                }
            }
        } else if let Some(ref _rep) = replace_str {
            let mut line = Vec::new();
            while let Ok(n) = stdin_handle.read_until(b'\n', &mut line) {
                if n == 0 {
                    break;
                }
                if line.ends_with(b"\n") {
                    line.pop();
                    if line.ends_with(b"\r") {
                        line.pop();
                    }
                }
                let trimmed_start = line.iter().position(|&b| !matches!(b, b' ' | b'\t' | b'\r' | b'\n' | 0x0b | 0x0c)).unwrap_or(line.len());
                let slice = &line[trimmed_start..];
                if let Some(ref eof) = eof_str {
                    if slice == eof.as_slice() {
                        break;
                    }
                }
                if !slice.is_empty() {
                    input_items.push(slice.to_vec());
                }
                line.clear();
            }
        } else {
            let mut text = Vec::new();
            let _ = stdin_handle.read_to_end(&mut text);
            let mut cur_word = Vec::new();
            let mut in_word = false;
            let mut stopped = false;
            let mut in_quotes = false;

            for &b in &text {
                if stopped {
                    break;
                }
                if b == b'"' {
                    in_quotes = !in_quotes;
                    continue;
                }
                if !in_quotes && b.is_ascii_whitespace() {
                    if in_word {
                        if let Some(ref eof) = eof_str {
                            if &cur_word == eof {
                                stopped = true;
                                break;
                            }
                        }
                        input_items.push(std::mem::take(&mut cur_word));
                        in_word = false;
                    }
                } else {
                    in_word = true;
                    cur_word.push(b);
                }
            }
            if in_word && !stopped {
                if let Some(ref eof) = eof_str {
                    if &cur_word != eof {
                        input_items.push(cur_word);
                    }
                } else {
                    input_items.push(cur_word);
                }
            }
        }

        if input_items.is_empty() {
            return Ok(0);
        }

        let mut overall_status = 0;

        if let Some(ref rep_pat) = replace_str {
            for item in input_items {
                let mut cmd_args: Vec<OsString> = Vec::new();
                for arg in &cmd_parts[1..] {
                    let b = arg.as_bytes();
                    if b.windows(rep_pat.len()).any(|w| w == rep_pat.as_slice()) {
                        let mut new_arg = Vec::new();
                        let mut j = 0;
                        while j < b.len() {
                            if j + rep_pat.len() <= b.len() && &b[j..j + rep_pat.len()] == rep_pat.as_slice() {
                                new_arg.extend_from_slice(&item);
                                j += rep_pat.len();
                            } else {
                                new_arg.push(b[j]);
                                j += 1;
                            }
                        }
                        cmd_args.push(OsString::from_vec(new_arg));
                    } else {
                        cmd_args.push(arg.clone());
                    }
                }

                if trace {
                    eprint!("{}", cmd_parts[0].to_string_lossy());
                    for a in &cmd_args {
                        eprint!(" {}", a.to_string_lossy());
                    }
                    eprintln!();
                }

                let mut full_cmd = Command::new(&cmd_parts[0]);
                full_cmd.args(&cmd_args);

                match full_cmd.status() {
                    Ok(status) => {
                        let code = status.code().unwrap_or(1);
                        if code != 0 {
                            overall_status = code;
                        }
                    }
                    Err(e) => {
                        eprintln!("xargs: {}: {}", cmd_parts[0].to_string_lossy(), e);
                        return Ok(127);
                    }
                }
            }
            return Ok(overall_status);
        }

        let mut idx = 0;
        let default_max_chars = 32 * 1024 - 2048;
        let limit_chars = max_chars.unwrap_or(default_max_chars);

        while idx < input_items.len() {
            let mut batch = Vec::new();
            let mut cur_len: usize = cmd_parts.iter().map(|a| a.as_bytes().len() + 1).sum();

            while idx < input_items.len() {
                let item_len = input_items[idx].len() + 1;
                if !batch.is_empty() && cur_len + item_len > limit_chars {
                    break;
                }
                batch.push(OsString::from_vec(input_items[idx].clone()));
                cur_len += item_len;
                idx += 1;
                if let Some(n) = max_args {
                    if batch.len() >= n {
                        break;
                    }
                }
            }

            if trace {
                eprint!("{}", cmd_parts[0].to_string_lossy());
                for a in &cmd_parts[1..] {
                    eprint!(" {}", a.to_string_lossy());
                }
                for a in &batch {
                    eprint!(" {}", a.to_string_lossy());
                }
                eprintln!();
            }

            let mut full_cmd = Command::new(&cmd_parts[0]);
            for arg in &cmd_parts[1..] {
                full_cmd.arg(arg);
            }
            for arg in &batch {
                full_cmd.arg(arg);
            }

            match full_cmd.status() {
                Ok(status) => {
                    let code = status.code().unwrap_or(1);
                    if code != 0 {
                        overall_status = code;
                    }
                }
                Err(e) => {
                    eprintln!("xargs: {}: {}", cmd_parts[0].to_string_lossy(), e);
                    return Ok(127);
                }
            }
        }

        Ok(overall_status)
    }
}
