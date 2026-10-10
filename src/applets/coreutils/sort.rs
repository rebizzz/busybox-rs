#![allow(non_upper_case_globals)]

use crate::core::{Applet, BbError, Result};
use std::ffi::OsString;
use std::io::{self, BufRead, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

pub const FLAG_n: u32 = 1 << 0;
pub const FLAG_g: u32 = 1 << 1;
pub const FLAG_h: u32 = 1 << 2;
pub const FLAG_M: u32 = 1 << 3;
pub const FLAG_V: u32 = 1 << 4;
pub const FLAG_u: u32 = 1 << 5;
pub const FLAG_c: u32 = 1 << 6;
pub const FLAG_s: u32 = 1 << 7;
pub const FLAG_z: u32 = 1 << 8;
pub const FLAG_b: u32 = 1 << 9;
pub const FLAG_r: u32 = 1 << 10;
pub const FLAG_d: u32 = 1 << 11;
pub const FLAG_f: u32 = 1 << 12;
pub const FLAG_i: u32 = 1 << 13;
pub const FLAG_bb: u32 = 0x8000_0000;
pub const FLAG_no_tie_break: u32 = 0x4000_0000;

unsafe extern "C" {
    fn strverscmp(s1: *const libc::c_char, s2: *const libc::c_char) -> libc::c_int;
}

#[derive(Clone, Debug)]
pub struct SortKey {
    pub range: [usize; 4],
    pub flags: u32,
}

#[inline]
fn is_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')
}

fn scale_suffix(tail: &[u8]) -> i32 {
    let suffix = b"kmgtpezy";
    if tail.is_empty() {
        return -1;
    }
    let first = tail[0].to_ascii_lowercase();
    if let Some(pos) = suffix.iter().position(|&c| c == first) {
        let n = pos as i32;
        if n != 0 && tail[0].is_ascii_lowercase() {
            return -1;
        }
        n
    } else {
        -1
    }
}

fn parse_strtod(s: &[u8]) -> (Option<f64>, usize) {
    let mut buf = [0u8; 128];
    let (c_ptr, _keep_alive) = if s.len() < 128 {
        buf[..s.len()].copy_from_slice(s);
        buf[s.len()] = 0;
        (buf.as_ptr() as *const libc::c_char, None)
    } else {
        let cs = match std::ffi::CString::new(s) {
            Ok(cs) => cs,
            Err(_) => {
                let end = s.iter().position(|&b| b == 0).unwrap_or(s.len());
                std::ffi::CString::new(&s[..end]).unwrap()
            }
        };
        (cs.as_ptr(), Some(cs))
    };

    let mut endptr: *mut libc::c_char = std::ptr::null_mut();
    let val = unsafe { libc::strtod(c_ptr, &mut endptr) };
    let consumed = unsafe { endptr.offset_from(c_ptr) as usize };
    if consumed == 0 {
        (None, 0)
    } else {
        (Some(val), consumed)
    }
}

fn compare_g_or_h(x: &[u8], y: &[u8], is_h: bool) -> i32 {
    let (dx_opt, consumed_x) = parse_strtod(x);
    let (dy_opt, consumed_y) = parse_strtod(y);

    if consumed_x == 0 {
        return if consumed_y == 0 { 0 } else { -1 };
    } else if consumed_y == 0 {
        return 1;
    }

    let dx = dx_opt.unwrap_or(0.0);
    let dy = dy_opt.unwrap_or(0.0);

    if dx.is_nan() {
        return if dy.is_nan() { 0 } else { -1 };
    } else if dy.is_nan() {
        return 1;
    }

    if is_h {
        let xs = scale_suffix(&x[consumed_x..]);
        let ys = scale_suffix(&y[consumed_y..]);
        if xs != ys {
            return (xs - ys).signum();
        }
    }

    if dx.is_infinite() {
        if dx < 0.0 {
            if dy.is_infinite() && dy < 0.0 {
                0
            } else {
                -1
            }
        } else {
            if dy.is_infinite() && dy > 0.0 {
                0
            } else {
                1
            }
        }
    } else if dy.is_infinite() {
        if dy < 0.0 {
            1
        } else {
            -1
        }
    } else if dx > dy {
        1
    } else if dx < dy {
        -1
    } else {
        0
    }
}

fn compare_n(x: &[u8], y: &[u8]) -> i32 {
    let (dx_opt, _) = parse_strtod(x);
    let (dy_opt, _) = parse_strtod(y);
    let dx = dx_opt.unwrap_or(0.0);
    let dy = dy_opt.unwrap_or(0.0);
    if dx > dy {
        1
    } else if dx < dy {
        -1
    } else {
        0
    }
}

fn parse_month(s: &[u8]) -> Option<i32> {
    let mut i = 0;
    while i < s.len() && is_space(s[i]) {
        i += 1;
    }
    let rest = &s[i..];
    if rest.len() < 3 {
        return None;
    }
    let m = [
        rest[0].to_ascii_lowercase(),
        rest[1].to_ascii_lowercase(),
        rest[2].to_ascii_lowercase(),
    ];
    match &m {
        b"jan" => Some(0),
        b"feb" => Some(1),
        b"mar" => Some(2),
        b"apr" => Some(3),
        b"may" => Some(4),
        b"jun" => Some(5),
        b"jul" => Some(6),
        b"aug" => Some(7),
        b"sep" => Some(8),
        b"oct" => Some(9),
        b"nov" => Some(10),
        b"dec" => Some(11),
        _ => None,
    }
}

fn compare_m(x: &[u8], y: &[u8]) -> i32 {
    let mx = parse_month(x);
    let my = parse_month(y);
    match (mx, my) {
        (None, None) => 0,
        (None, Some(_)) => -1,
        (Some(_), None) => 1,
        (Some(dx), Some(dy)) => (dx - dy).signum(),
    }
}

fn compare_v(x: &[u8], y: &[u8]) -> i32 {
    let cs_x = match std::ffi::CString::new(x) {
        Ok(cs) => cs,
        Err(_) => {
            let end = x.iter().position(|&b| b == 0).unwrap_or(x.len());
            std::ffi::CString::new(&x[..end]).unwrap()
        }
    };
    let cs_y = match std::ffi::CString::new(y) {
        Ok(cs) => cs,
        Err(_) => {
            let end = y.iter().position(|&b| b == 0).unwrap_or(y.len());
            std::ffi::CString::new(&y[..end]).unwrap()
        }
    };
    let res = unsafe { strverscmp(cs_x.as_ptr(), cs_y.as_ptr()) };
    if res > 0 {
        1
    } else if res < 0 {
        -1
    } else {
        0
    }
}

fn compare_key_chunks(x: &[u8], y: &[u8], flags: u32) -> i32 {
    match flags & (FLAG_n | FLAG_g | FLAG_h | FLAG_M | FLAG_V) {
        FLAG_V => compare_v(x, y),
        FLAG_g => compare_g_or_h(x, y, false),
        FLAG_h => compare_g_or_h(x, y, true),
        FLAG_M => compare_m(x, y),
        FLAG_n => compare_n(x, y),
        _ => match x.cmp(y) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        },
    }
}

fn get_key(s: &[u8], key: &SortKey, flags: u32, key_sep: Option<u8>) -> Vec<u8> {
    if key.range[0] == 1
        && key.range[1] == 0
        && key.range[2] == 0
        && key.range[3] == 0
        && (flags & (FLAG_b | FLAG_d | FLAG_f | FLAG_i | FLAG_bb)) == 0
    {
        return s.to_vec();
    }

    let len = s.len();
    let mut start = 0;
    let mut end = 0;

    for j in 0..2 {
        if key.range[2 * j] == 0 {
            end = len;
        } else {
            let mut ended_on_sep = false;
            let mut fields = key.range[2 * j] + j;
            if j != 0 && key.range[3] != 0 {
                fields -= 1;
            }

            end = 0;
            for _i in 1..fields {
                if let Some(sep) = key_sep {
                    ended_on_sep = false;
                    while end < len {
                        let b = s[end];
                        end += 1;
                        if b == sep {
                            ended_on_sep = true;
                            break;
                        }
                    }
                } else {
                    while end < len && is_space(s[end]) {
                        end += 1;
                    }
                    while end < len {
                        if is_space(s[end]) {
                            break;
                        }
                        end += 1;
                    }
                }
            }
            if j != 0 && ended_on_sep && key.range[3] == 0 && end > 0 {
                end -= 1;
            }
        }
        if j == 0 {
            start = end;
        }
    }

    if (flags & FLAG_b) != 0 {
        while start < len && is_space(s[start]) {
            start += 1;
        }
    }
    if (flags & FLAG_bb) != 0 {
        while end > start && is_space(s[end - 1]) {
            end -= 1;
        }
    }
    if key.range[3] != 0 {
        end += key.range[3];
        if end > len {
            end = len;
        }
    }
    if key.range[1] != 0 {
        start += key.range[1] - 1;
        if start > len {
            start = len;
        }
    }
    if end < start {
        end = start;
    }

    let mut chunk = s[start..end].to_vec();
    if (flags & FLAG_d) != 0 {
        chunk.retain(|&b| is_space(b) || b.is_ascii_alphanumeric());
    }
    if (flags & FLAG_i) != 0 {
        chunk.retain(|&b| (0x20..=0x7E).contains(&b));
    }
    if (flags & FLAG_f) != 0 {
        for b in chunk.iter_mut() {
            *b = b.to_ascii_uppercase();
        }
    }

    chunk
}

fn compare_lines(
    a_idx: usize,
    a_line: &[u8],
    b_idx: usize,
    b_line: &[u8],
    keys: &[SortKey],
    global_flags: u32,
    key_sep: Option<u8>,
) -> std::cmp::Ordering {
    let mut flags = global_flags;
    let mut retval = 0;

    for key in keys {
        flags = if key.flags != 0 {
            key.flags
        } else {
            global_flags
        };
        let x = get_key(a_line, key, flags, key_sep);
        let y = get_key(b_line, key, flags, key_sep);

        retval = compare_key_chunks(&x, &y, flags);
        if retval != 0 {
            break;
        }
    }

    if retval == 0 {
        if (global_flags & FLAG_s) != 0 {
            return a_idx.cmp(&b_idx);
        }
        if (global_flags & FLAG_no_tie_break) == 0 {
            flags = global_flags;
            retval = match a_line.cmp(b_line) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            };
        }
    }

    if (flags & FLAG_r) != 0 {
        retval = -retval;
    }

    retval.cmp(&0)
}

fn parse_key_spec(spec: &str) -> std::result::Result<SortKey, &'static str> {
    let bytes = spec.as_bytes();
    let mut idx = 0;
    let mut key = SortKey {
        range: [0; 4],
        flags: 0,
    };
    let mut part = 0;

    while idx < bytes.len() {
        let start_digit = idx;
        while idx < bytes.len() && bytes[idx].is_ascii_digit() {
            idx += 1;
        }
        if idx == start_digit {
            return Err("bad field specification");
        }
        let num_str =
            std::str::from_utf8(&bytes[start_digit..idx]).map_err(|_| "bad field specification")?;
        let field_num: usize = num_str.parse().map_err(|_| "bad field specification")?;
        if field_num == 0 {
            return Err("bad field specification");
        }
        key.range[2 * part] = field_num;

        if idx < bytes.len() && bytes[idx] == b'.' {
            idx += 1;
            let start_char_digit = idx;
            while idx < bytes.len() && bytes[idx].is_ascii_digit() {
                idx += 1;
            }
            if idx == start_char_digit {
                return Err("bad field specification");
            }
            let char_str = std::str::from_utf8(&bytes[start_char_digit..idx])
                .map_err(|_| "bad field specification")?;
            let char_num: usize = char_str.parse().map_err(|_| "bad field specification")?;
            if char_num == 0 {
                return Err("bad field specification");
            }
            key.range[2 * part + 1] = char_num;
        }

        while idx < bytes.len() {
            if bytes[idx] == b',' && part == 0 {
                part = 1;
                idx += 1;
                break;
            }
            let ch = bytes[idx];
            let mut flag = match ch {
                b'n' => FLAG_n,
                b'g' => FLAG_g,
                b'h' => FLAG_h,
                b'M' => FLAG_M,
                b'b' => FLAG_b,
                b'r' => FLAG_r,
                b'd' => FLAG_d,
                b'f' => FLAG_f,
                b'i' => FLAG_i,
                _ => return Err("unknown key option"),
            };
            if part != 0 && flag == FLAG_b {
                flag = FLAG_bb;
            }
            key.flags |= flag;
            idx += 1;
        }
    }

    Ok(key)
}

pub struct SortApplet;

impl Applet for SortApplet {
    fn name(&self) -> &'static str {
        "sort"
    }

    fn description(&self) -> &'static str {
        "Sort lines of text"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut flag_n = false;
        let mut flag_g = false;
        let mut flag_h = false;
        let mut flag_m_month = false;
        let mut flag_v = false;
        let mut flag_u = false;
        let mut flag_c = false;
        let mut flag_s = false;
        let mut flag_z = false;
        let mut flag_b = false;
        let mut flag_r = false;
        let mut flag_d = false;
        let mut flag_f = false;
        let mut flag_i = false;

        let mut output_file: Option<OsString> = None;
        let mut key_separator: Option<u8> = None;
        let mut key_specs: Vec<String> = Vec::new();
        let mut input_files: Vec<OsString> = Vec::new();

        let mut i = 0;
        let mut end_of_options = false;

        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();

            if end_of_options {
                input_files.push(arg.clone());
                i += 1;
                continue;
            }

            if bytes == b"--" {
                end_of_options = true;
                i += 1;
                continue;
            }

            if bytes == b"-" {
                input_files.push(arg.clone());
                i += 1;
                continue;
            }

            if bytes.starts_with(b"-") && bytes.len() > 1 {
                let mut j = 1;
                while j < bytes.len() {
                    let c = bytes[j];
                    match c {
                        b'o' => {
                            if j + 1 < bytes.len() {
                                output_file = Some(OsString::from(
                                    std::str::from_utf8(&bytes[j + 1..]).unwrap_or(""),
                                ));
                            } else {
                                i += 1;
                                if i >= args.len() {
                                    eprintln!("sort: option requires an argument -- 'o'");
                                    return Ok(2);
                                }
                                output_file = Some(args[i].clone());
                            }
                            break;
                        }
                        b't' => {
                            let val = if j + 1 < bytes.len() {
                                &bytes[j + 1..]
                            } else {
                                i += 1;
                                if i >= args.len() {
                                    eprintln!("sort: option requires an argument -- 't'");
                                    return Ok(2);
                                }
                                args[i].as_bytes()
                            };
                            if val.len() != 1 {
                                eprintln!("sort: bad -t parameter");
                                return Ok(2);
                            }
                            key_separator = Some(val[0]);
                            break;
                        }
                        b'k' => {
                            let val = if j + 1 < bytes.len() {
                                String::from_utf8_lossy(&bytes[j + 1..]).into_owned()
                            } else {
                                i += 1;
                                if i >= args.len() {
                                    eprintln!("sort: option requires an argument -- 'k'");
                                    return Ok(2);
                                }
                                args[i].to_string_lossy().into_owned()
                            };
                            key_specs.push(val);
                            break;
                        }
                        b'S' | b'T' => {
                            if j + 1 < bytes.len() {
                            } else {
                                i += 1;
                            }
                            break;
                        }
                        b'n' => flag_n = true,
                        b'g' => flag_g = true,
                        b'h' => flag_h = true,
                        b'M' => flag_m_month = true,
                        b'V' => flag_v = true,
                        b'u' => flag_u = true,
                        b'c' => flag_c = true,
                        b's' => flag_s = true,
                        b'z' => flag_z = true,
                        b'b' => flag_b = true,
                        b'r' => flag_r = true,
                        b'd' => flag_d = true,
                        b'f' => flag_f = true,
                        b'i' => flag_i = true,
                        b'm' => {}
                        _ => {
                            eprintln!("sort: invalid option -- '{}'", c as char);
                            return Ok(2);
                        }
                    }
                    j += 1;
                }
            } else {
                input_files.push(arg.clone());
            }
            i += 1;
        }

        let mut global_flags: u32 = 0;
        if flag_n {
            global_flags |= FLAG_n;
        }
        if flag_g {
            global_flags |= FLAG_g;
        }
        if flag_h {
            global_flags |= FLAG_h;
        }
        if flag_m_month {
            global_flags |= FLAG_M;
        }
        if flag_v {
            global_flags |= FLAG_V;
        }
        if flag_u {
            global_flags |= FLAG_u;
        }
        if flag_c {
            global_flags |= FLAG_c;
        }
        if flag_s {
            global_flags |= FLAG_s;
        }
        if flag_z {
            global_flags |= FLAG_z;
        }
        if flag_r {
            global_flags |= FLAG_r;
        }
        if flag_d {
            global_flags |= FLAG_d;
        }
        if flag_f {
            global_flags |= FLAG_f;
        }
        if flag_i {
            global_flags |= FLAG_i;
        }
        if flag_b {
            global_flags |= FLAG_b | FLAG_bb;
        }

        let mut keys: Vec<SortKey> = Vec::new();
        for spec in &key_specs {
            match parse_key_spec(spec) {
                Ok(k) => keys.push(k),
                Err(msg) => {
                    eprintln!("sort: {}", msg);
                    return Ok(2);
                }
            }
        }
        if keys.is_empty() {
            keys.push(SortKey {
                range: [1, 0, 0, 0],
                flags: 0,
            });
        }

        if input_files.is_empty() {
            input_files.push(OsString::from("-"));
        }

        let delim = if flag_z { b'\0' } else { b'\n' };
        let mut lines: Vec<Vec<u8>> = Vec::new();

        for file_arg in &input_files {
            if file_arg == "-" {
                let stdin = io::stdin();
                let mut reader = stdin.lock();
                let mut buf = Vec::new();
                loop {
                    buf.clear();
                    let n = reader.read_until(delim, &mut buf).map_err(BbError::from)?;
                    if n == 0 {
                        break;
                    }
                    if buf.ends_with(&[delim]) {
                        buf.pop();
                    }
                    lines.push(buf.clone());
                }
            } else {
                let p = Path::new(file_arg);
                let f = match std::fs::File::open(p) {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("sort: {}: {}", p.display(), e);
                        return Ok(1);
                    }
                };
                let mut reader = io::BufReader::new(f);
                let mut buf = Vec::new();
                loop {
                    buf.clear();
                    let n = reader.read_until(delim, &mut buf).map_err(BbError::from)?;
                    if n == 0 {
                        break;
                    }
                    if buf.ends_with(&[delim]) {
                        buf.pop();
                    }
                    lines.push(buf.clone());
                }
            }
        }

        if flag_c {
            let limit = if flag_u { -1 } else { 0 };
            for idx in 1..lines.len() {
                let cmp = compare_lines(
                    idx - 1,
                    &lines[idx - 1],
                    idx,
                    &lines[idx],
                    &keys,
                    global_flags,
                    key_separator,
                );
                let val = match cmp {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                };
                if val > limit {
                    eprintln!("Check line {}", idx);
                    return Ok(1);
                }
            }
            return Ok(0);
        }

        let mut indexed_lines: Vec<(usize, Vec<u8>)> = lines.into_iter().enumerate().collect();

        indexed_lines.sort_by(|(idx_a, line_a), (idx_b, line_b)| {
            compare_lines(
                *idx_a,
                line_a,
                *idx_b,
                line_b,
                &keys,
                global_flags,
                key_separator,
            )
        });

        let sorted_lines: Vec<Vec<u8>> = if flag_u {
            if indexed_lines.is_empty() {
                Vec::new()
            } else {
                let u_flags = (global_flags | FLAG_no_tie_break) & (!FLAG_s);
                let mut result: Vec<Vec<u8>> = Vec::with_capacity(indexed_lines.len());
                let mut last_idx = indexed_lines[0].0;
                let mut last_line = std::mem::take(&mut indexed_lines[0].1);

                for (curr_idx, curr_line) in indexed_lines.into_iter().skip(1) {
                    let cmp = compare_lines(
                        last_idx,
                        &last_line,
                        curr_idx,
                        &curr_line,
                        &keys,
                        u_flags,
                        key_separator,
                    );
                    if cmp != std::cmp::Ordering::Equal {
                        result.push(last_line);
                        last_idx = curr_idx;
                        last_line = curr_line;
                    }
                }
                result.push(last_line);
                result
            }
        } else {
            indexed_lines.into_iter().map(|(_, l)| l).collect()
        };

        let out_writer: Box<dyn Write> = if let Some(ref path) = output_file {
            let f = std::fs::File::create(path).map_err(|e| BbError::Io {
                path: Some(PathBuf::from(path)),
                source: e,
            })?;
            Box::new(f)
        } else {
            Box::new(io::stdout().lock())
        };

        let mut buf_out = io::BufWriter::new(out_writer);
        for line in &sorted_lines {
            buf_out.write_all(line).map_err(BbError::from)?;
            buf_out.write_all(&[delim]).map_err(BbError::from)?;
        }
        buf_out.flush().map_err(BbError::from)?;

        Ok(0)
    }
}
