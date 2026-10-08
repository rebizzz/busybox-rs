use crate::core::fs::open_or_stdin;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct CmpApplet;

fn parse_scaled_number(s: &str) -> std::result::Result<u64, ()> {
    if s.is_empty() {
        return Err(());
    }
    let s_bytes = s.as_bytes();
    let mut num_end = 0;
    while num_end < s_bytes.len() && s_bytes[num_end].is_ascii_digit() {
        num_end += 1;
    }
    if num_end == 0 {
        return Err(());
    }
    let num: u64 = s[..num_end].parse().map_err(|_| ())?;
    let sfx = &s[num_end..];
    let mult: u64 = match sfx {
        "" => 1,
        "k" | "K" | "kiB" | "KiB" => 1024,
        "kb" | "kB" | "KB" => 1000,
        "m" | "M" | "miB" | "MiB" => 1024 * 1024,
        "mb" | "mB" | "MB" => 1_000_000,
        "g" | "G" | "giB" | "GiB" => 1024 * 1024 * 1024,
        "gb" | "gB" | "GB" => 1_000_000_000,
        _ => return Err(()),
    };
    num.checked_mul(mult).ok_or(())
}

fn skip_bytes<R: Read>(reader: &mut R, count: u64) -> io::Result<()> {
    if count == 0 {
        return Ok(());
    }
    let mut remaining = count;
    let mut buf = [0u8; 8192];
    while remaining > 0 {
        let to_read = remaining.min(buf.len() as u64) as usize;
        let n = reader.read(&mut buf[..to_read])?;
        if n == 0 {
            break;
        }
        remaining -= n as u64;
    }
    Ok(())
}

impl Applet for CmpApplet {
    fn name(&self) -> &'static str {
        "cmp"
    }

    fn description(&self) -> &'static str {
        "Compare two files byte by byte"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut opt_l = false;
        let mut opt_s = false;
        let mut max_count: Option<u64> = None;
        let mut positional = Vec::new();

        let mut i = 0;
        let mut parsing_opts = true;
        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();

            if parsing_opts && bytes == b"--" {
                parsing_opts = false;
                i += 1;
                continue;
            }

            if parsing_opts && bytes.starts_with(b"-") && bytes.len() > 1 && bytes != b"-" {
                let mut j = 1;
                let mut need_next_arg = false;
                while j < bytes.len() {
                    let c = bytes[j];
                    match c {
                        b'l' => {
                            opt_l = true;
                            opt_s = false;
                            j += 1;
                        }
                        b's' => {
                            opt_s = true;
                            opt_l = false;
                            j += 1;
                        }
                        b'n' => {
                            let num_str = if j + 1 < bytes.len() {
                                let s = match std::str::from_utf8(&bytes[j + 1..]) {
                                    Ok(val) => val,
                                    Err(_) => {
                                        eprintln!("cmp: invalid number");
                                        return Ok(1);
                                    }
                                };
                                j = bytes.len();
                                s
                            } else if i + 1 < args.len() {
                                need_next_arg = true;
                                i += 1;
                                let s = match args[i].to_str() {
                                    Some(val) => val,
                                    None => {
                                        eprintln!("cmp: invalid number");
                                        return Ok(1);
                                    }
                                };
                                j = bytes.len();
                                s
                            } else {
                                eprintln!("cmp: option requires an argument -- 'n'");
                                return Ok(1);
                            };

                            match parse_scaled_number(num_str) {
                                Ok(n) => max_count = Some(n),
                                Err(_) => {
                                    eprintln!("cmp: invalid number '{}'", num_str);
                                    return Ok(1);
                                }
                            }
                        }
                        _ => {
                            eprintln!("cmp: invalid option -- '{}'", c as char);
                            eprintln!("Usage: cmp [-l|s] [-n NUM] FILE1 [FILE2 [SKIP1 [SKIP2]]]");
                            return Ok(1);
                        }
                    }
                    if need_next_arg {
                        break;
                    }
                }
                i += 1;
            } else {
                positional.push(arg);
                i += 1;
            }
        }

        if positional.is_empty() || positional.len() > 4 {
            eprintln!("Usage: cmp [-l|s] [-n NUM] FILE1 [FILE2 [SKIP1 [SKIP2]]]");
            return Ok(1);
        }

        let file1_arg = positional[0];
        let file2_arg = if positional.len() > 1 {
            positional[1]
        } else {
            &OsString::from("-")
        };

        let mut skip1: u64 = 0;
        let mut skip2: u64 = 0;

        if positional.len() > 2 {
            let s1 = positional[2].to_str().unwrap_or("");
            match parse_scaled_number(s1) {
                Ok(n) => skip1 = n,
                Err(_) => {
                    eprintln!("cmp: invalid number '{}'", positional[2].to_string_lossy());
                    return Ok(1);
                }
            }
        }

        if positional.len() > 3 {
            let s2 = positional[3].to_str().unwrap_or("");
            match parse_scaled_number(s2) {
                Ok(n) => skip2 = n,
                Err(_) => {
                    eprintln!("cmp: invalid number '{}'", positional[3].to_string_lossy());
                    return Ok(1);
                }
            }
        }

        let f1_is_stdin = file1_arg.as_bytes() == b"-";
        let f2_is_stdin = file2_arg.as_bytes() == b"-";

        if f1_is_stdin && f2_is_stdin {
            return Ok(0);
        }

        let mut reader1 = match open_or_stdin(Path::new(file1_arg)) {
            Ok(r) => r,
            Err(e) => {
                if !opt_s {
                    eprintln!("cmp: {}", e);
                }
                return Ok(2);
            }
        };

        let mut reader2 = match open_or_stdin(Path::new(file2_arg)) {
            Ok(r) => r,
            Err(e) => {
                if !opt_s {
                    eprintln!("cmp: {}", e);
                }
                return Ok(2);
            }
        };

        if let Err(e) = skip_bytes(&mut reader1, skip1) {
            if !opt_s {
                eprintln!("cmp: {}: {}", file1_arg.to_string_lossy(), e);
            }
            return Ok(2);
        }

        if let Err(e) = skip_bytes(&mut reader2, skip2) {
            if !opt_s {
                eprintln!("cmp: {}: {}", file2_arg.to_string_lossy(), e);
            }
            return Ok(2);
        }

        let f1_display = file1_arg.to_string_lossy();
        let f2_display = file2_arg.to_string_lossy();

        let stdout = io::stdout();
        let mut out = stdout.lock();

        let mut byte_pos: u64 = 0;
        let mut line_pos: u64 = 1;

        let mut buf1 = [0u8; 8192];
        let mut buf2 = [0u8; 8192];
        let mut len1 = 0;
        let mut len2 = 0;
        let mut idx1 = 0;
        let mut idx2 = 0;

        let mut diff_found = false;

        loop {
            if let Some(limit) = max_count {
                if byte_pos >= limit {
                    break;
                }
            }

            if idx1 >= len1 {
                match reader1.read(&mut buf1) {
                    Ok(n) => {
                        len1 = n;
                        idx1 = 0;
                    }
                    Err(e) => {
                        if !opt_s {
                            eprintln!("cmp: {}: {}", f1_display, e);
                        }
                        return Ok(2);
                    }
                }
            }

            if idx2 >= len2 {
                match reader2.read(&mut buf2) {
                    Ok(n) => {
                        len2 = n;
                        idx2 = 0;
                    }
                    Err(e) => {
                        if !opt_s {
                            eprintln!("cmp: {}: {}", f2_display, e);
                        }
                        return Ok(2);
                    }
                }
            }

            let eof1 = idx1 >= len1;
            let eof2 = idx2 >= len2;

            if eof1 && eof2 {
                break;
            }

            if eof1 || eof2 {
                if !opt_s {
                    let shorter_file = if eof1 { &f1_display } else { &f2_display };
                    let _ = out.flush();
                    eprintln!("cmp: EOF on {}", shorter_file);
                }
                return Ok(1);
            }

            let b1 = buf1[idx1];
            let b2 = buf2[idx2];
            idx1 += 1;
            idx2 += 1;
            byte_pos += 1;

            if b1 != b2 {
                diff_found = true;
                if opt_s {
                    return Ok(1);
                }
                if opt_l {
                    let _ = writeln!(out, "{} {:3o} {:3o}", byte_pos, b1, b2);
                } else {
                    let _ = writeln!(
                        out,
                        "{} {} differ: byte {}, line {}",
                        f1_display, f2_display, byte_pos, line_pos
                    );
                    return Ok(1);
                }
            }

            if b1 == b'\n' {
                line_pos += 1;
            }
        }

        let _ = out.flush();
        if diff_found {
            Ok(1)
        } else {
            Ok(0)
        }
    }
}
