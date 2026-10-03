use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use crate::core::fs::read_bytes_or_stdin;
use crate::core::{Applet, Result};

pub struct CutApplet;
impl Applet for CutApplet {
    fn name(&self) -> &'static str { "cut" }
    fn description(&self) -> &'static str { "Print selected fields from FILEs to stdout" }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut delim_opt: Option<Vec<u8>> = None;
        let mut odelim_opt: Option<Vec<u8>> = None;
        let mut suppress = false;
        let mut no_sort = false;
        let mut mode: Option<char> = None;
        let mut list_opt: Option<Vec<u8>> = None;
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();

            if bytes.starts_with(b"--output-delimiter=") {
                odelim_opt = Some(bytes[b"--output-delimiter=".len()..].to_vec());
                i += 1;
                continue;
            } else if bytes == b"--output-delimiter" {
                if i + 1 < args.len() {
                    odelim_opt = Some(args[i + 1].as_bytes().to_vec());
                    i += 2;
                    continue;
                }
            }

            if bytes.starts_with(b"-") && bytes.len() > 1 && bytes != b"-" {
                let mut arg_idx = 1;
                while arg_idx < bytes.len() {
                    let opt_char = bytes[arg_idx] as char;
                    match opt_char {
                        's' => { suppress = true; arg_idx += 1; }
                        'D' => { no_sort = true; arg_idx += 1; }
                        'n' => { arg_idx += 1; }
                        'd' => {
                            let val = if arg_idx + 1 < bytes.len() {
                                bytes[arg_idx + 1..].to_vec()
                            } else if i + 1 < args.len() {
                                i += 1;
                                args[i].as_bytes().to_vec()
                            } else {
                                Vec::new()
                            };
                            delim_opt = Some(val);
                            break;
                        }
                        'O' => {
                            let val = if arg_idx + 1 < bytes.len() {
                                bytes[arg_idx + 1..].to_vec()
                            } else if i + 1 < args.len() {
                                i += 1;
                                args[i].as_bytes().to_vec()
                            } else {
                                Vec::new()
                            };
                            odelim_opt = Some(val);
                            break;
                        }
                        'b' | 'c' | 'f' | 'F' => {
                            mode = Some(opt_char);
                            let val = if arg_idx + 1 < bytes.len() {
                                bytes[arg_idx + 1..].to_vec()
                            } else if i + 1 < args.len() {
                                i += 1;
                                args[i].as_bytes().to_vec()
                            } else {
                                Vec::new()
                            };
                            list_opt = Some(val);
                            break;
                        }
                        _ => { arg_idx += 1; }
                    }
                }
            } else {
                files.push(Path::new(arg));
            }
            i += 1;
        }

        let m = match mode {
            Some(m) => m,
            None => {
                eprintln!("cut: expected a list of bytes, characters, or fields");
                return Ok(1);
            }
        };

        let list_bytes = list_opt.unwrap_or_default();
        let list_str = String::from_utf8_lossy(&list_bytes);

        let mut ranges: Vec<(usize, usize)> = Vec::new();
        for part in list_str.split(',') {
            if part.is_empty() { continue; }
            if let Some(idx) = part.find('-') {
                let start_str = &part[..idx];
                let end_str = &part[idx + 1..];
                let start: usize = if start_str.is_empty() { 1 } else {
                    match start_str.parse() {
                        Ok(v) if v > 0 => v,
                        _ => { eprintln!("cut: invalid range"); return Ok(1); }
                    }
                };
                let end: usize = if end_str.is_empty() { usize::MAX } else {
                    match end_str.parse() {
                        Ok(v) if v > 0 => v,
                        _ => { eprintln!("cut: invalid range"); return Ok(1); }
                    }
                };
                if start > end {
                    eprintln!("cut: invalid decreasing range");
                    return Ok(1);
                }
                ranges.push((start, end));
            } else {
                match part.parse::<usize>() {
                    Ok(v) if v > 0 => ranges.push((v, v)),
                    _ => { eprintln!("cut: invalid range"); return Ok(1); }
                }
            }
        }

        if !no_sort {
            ranges.sort_by_key(|r| r.0);
        }

        if files.is_empty() {
            files.push(Path::new("-"));
        }

        let is_regex = m == 'F';
        let default_delim = if is_regex { b" ".to_vec() } else { b"\t".to_vec() };
        let delim = delim_opt.unwrap_or(default_delim);

        let odelim = if let Some(od) = odelim_opt {
            od
        } else if m == 'b' || m == 'c' {
            Vec::new()
        } else if is_regex {
            b" ".to_vec()
        } else {
            delim.clone()
        };

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        for file in files {
            let content = read_bytes_or_stdin(file)?;

            if delim == b"\n" && !is_regex {
                // Cut by lines
                let s = String::from_utf8_lossy(&content);
                let lines: Vec<&str> = s.lines().collect();
                let mut first_print = true;
                for &(start, end) in &ranges {
                    for line_idx in start..=end.min(lines.len()) {
                        if line_idx >= 1 && line_idx <= lines.len() {
                            if !first_print {
                                handle.write_all(&odelim)?;
                            }
                            first_print = false;
                            handle.write_all(lines[line_idx - 1].as_bytes())?;
                        }
                    }
                }
                if !first_print {
                    handle.write_all(b"\n")?;
                }
                continue;
            }

            let mut lines = Vec::new();
            let mut cur = Vec::new();
            for &b in &content {
                if b == b'\n' {
                    lines.push(std::mem::take(&mut cur));
                } else {
                    cur.push(b);
                }
            }
            if !cur.is_empty() {
                lines.push(cur);
            }

            for line in lines {
                if m == 'b' || m == 'c' {
                    let linelen = line.len();
                    let mut printed = vec![false; linelen];
                    let mut need_odelim = false;

                    for &(start, end) in &ranges {
                        let mut spos = start.saturating_sub(1);
                        let endpos = if end == usize::MAX { usize::MAX } else { end - 1 };
                        while spos < linelen {
                            if !printed[spos] {
                                printed[spos] = true;
                                if need_odelim && spos != 0 && !printed[spos - 1] {
                                    need_odelim = false;
                                    handle.write_all(&odelim)?;
                                }
                                handle.write_all(&[line[spos]])?;
                            }
                            if spos >= endpos {
                                if !odelim.is_empty() {
                                    need_odelim = true;
                                }
                                break;
                            }
                            spos += 1;
                        }
                    }
                    handle.write_all(b"\n")?;
                } else {
                    // Field cutting (f or F)
                    if delim.is_empty() {
                        if suppress { continue; }
                        handle.write_all(&line)?;
                        handle.write_all(b"\n")?;
                        continue;
                    }

                    if is_regex {
                        let s = String::from_utf8_lossy(&line);
                        let has_delim = if delim == b" " {
                            s.chars().any(|c| c.is_whitespace())
                        } else {
                            line.windows(delim.len()).any(|w| w == delim.as_slice())
                        };

                        if !has_delim {
                            if suppress { continue; }
                            if no_sort {
                                let has1 = ranges.iter().any(|&(st, en)| st <= 1 && 1 <= en);
                                if has1 {
                                    writeln!(handle, "{}", s)?;
                                } else {
                                    writeln!(handle)?;
                                }
                                continue;
                            }
                            writeln!(handle, "{}", s)?;
                            continue;
                        }

                        // Split by whitespace or regex
                        let fields: Vec<&str> = if delim == b" " {
                            s.split_whitespace().collect()
                        } else {
                            let d_str = String::from_utf8_lossy(&delim);
                            s.split(&*d_str).collect()
                        };

                        let mut matched_fields = Vec::new();
                        for &(start, end) in &ranges {
                            if start == end {
                                if start >= 1 && start <= fields.len() {
                                    matched_fields.push(fields[start - 1]);
                                }
                            } else {
                                // Preserves intermediate delimiters
                                let mut start_match = None;
                                let mut end_match = None;
                                let mut field_idx = 0;
                                let mut in_word = false;

                                for (ci, ch) in s.char_indices() {
                                    if ch.is_whitespace() {
                                        if in_word {
                                            in_word = false;
                                            if field_idx == end {
                                                end_match = Some(ci);
                                                break;
                                            }
                                        }
                                    } else {
                                        if !in_word {
                                            in_word = true;
                                            field_idx += 1;
                                            if field_idx == start {
                                                start_match = Some(ci);
                                            }
                                        }
                                    }
                                }
                                if in_word && end_match.is_none() && field_idx <= end {
                                    end_match = Some(s.len());
                                }

                                if let (Some(sm), Some(em)) = (start_match, end_match) {
                                    matched_fields.push(&s[sm..em]);
                                }
                            }
                        }
                        let od_str = String::from_utf8_lossy(&odelim);
                        writeln!(handle, "{}", matched_fields.join(&od_str))?;
                    } else {
                        let delim_byte = delim[0];
                        let has_delim = line.contains(&delim_byte);
                        if !has_delim {
                            if suppress { continue; }
                            handle.write_all(&line)?;
                            handle.write_all(b"\n")?;
                            continue;
                        }

                        let fields: Vec<&[u8]> = line.split(|&b| b == delim_byte).collect();
                        let num_fields = fields.len();

                        let mut in_fields = vec![false; num_fields];
                        for &(start, end) in &ranges {
                            let s = start.saturating_sub(1);
                            let e = end.min(num_fields);
                            for idx in s..e {
                                if idx < num_fields { in_fields[idx] = true; }
                            }
                        }

                        let mut first = true;
                        for (idx, &is_sel) in in_fields.iter().enumerate() {
                            if is_sel {
                                if !first { handle.write_all(&odelim)?; }
                                first = false;
                                handle.write_all(fields[idx])?;
                            }
                        }
                        handle.write_all(b"\n")?;
                    }
                }
            }
        }
        Ok(0)
    }
}
