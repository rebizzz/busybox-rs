use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, Write};

pub struct PrintfApplet;

impl Applet for PrintfApplet {
    fn name(&self) -> &'static str {
        "printf"
    }

    fn description(&self) -> &'static str {
        "Format and print data"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("printf: missing operand");
            return Ok(1);
        }

        let format_str = args[0].to_string_lossy().to_string();
        let arg_strings: Vec<String> = args[1..]
            .iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect();

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        let mut arg_idx = 0;
        let mut exit_code = 0;
        let num_args = arg_strings.len();

        loop {
            let start_arg_idx = arg_idx;
            let mut stop = false;

            if let Err(e) = format_once(
                &format_str,
                &arg_strings,
                &mut arg_idx,
                &mut handle,
                &mut exit_code,
                &mut stop,
            ) {
                eprintln!("printf: {}", e);
                return Ok(1);
            }

            if stop {
                break;
            }

            if arg_idx >= num_args {
                break;
            }

            if arg_idx == start_arg_idx {
                break;
            }
        }

        Ok(exit_code)
    }
}

fn parse_numeric_arg(arg: Option<&str>, err_out: &mut i32) -> (f64, bool) {
    let s = match arg {
        Some(s) => s.trim(),
        None => return (0.0, true),
    };

    if s.is_empty() {
        eprintln!("printf: invalid number ''");
        *err_out = 1;
        return (0.0, false);
    }

    if (s.starts_with('\'') || s.starts_with('"')) && s.len() >= 2 {
        let ch = s.chars().nth(1).unwrap();
        return (ch as u32 as f64, true);
    }

    if s.starts_with("0x") || s.starts_with("0X") || s.starts_with("+0x") || s.starts_with("+0X") {
        let hex_slice = if s.starts_with('+') { &s[3..] } else { &s[2..] };
        if let Ok(val) = i64::from_str_radix(hex_slice, 16) {
            return (val as f64, true);
        }
    }

    if (s.starts_with('0') || s.starts_with("+0")) && s.len() > 1 && !s.contains('.') {
        let oct_slice = if s.starts_with('+') { &s[2..] } else { &s[1..] };
        if let Ok(val) = i64::from_str_radix(oct_slice, 8) {
            return (val as f64, true);
        }
    }

    match s.parse::<f64>() {
        Ok(v) => (v, true),
        Err(_) => {
            eprintln!("printf: invalid number '{}'", s);
            *err_out = 1;
            (0.0, false)
        }
    }
}

fn format_once<W: Write>(
    format: &str,
    args: &[String],
    arg_idx: &mut usize,
    out: &mut W,
    exit_code: &mut i32,
    stop: &mut bool,
) -> std::result::Result<(), String> {
    let bytes = format.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 1;
            if i >= bytes.len() {
                out.write_all(b"\\").map_err(|e| e.to_string())?;
                break;
            }
            match bytes[i] {
                b'a' => out.write_all(b"\x07").map_err(|e| e.to_string())?,
                b'b' => out.write_all(b"\x08").map_err(|e| e.to_string())?,
                b'f' => out.write_all(b"\x0c").map_err(|e| e.to_string())?,
                b'n' => out.write_all(b"\n").map_err(|e| e.to_string())?,
                b'r' => out.write_all(b"\r").map_err(|e| e.to_string())?,
                b't' => out.write_all(b"\t").map_err(|e| e.to_string())?,
                b'v' => out.write_all(b"\x0b").map_err(|e| e.to_string())?,
                b'\\' => out.write_all(b"\\").map_err(|e| e.to_string())?,
                b'c' => {
                    *stop = true;
                    return Ok(());
                }
                b'0'..=b'7' => {
                    let mut val = bytes[i] - b'0';
                    let mut count = 1;
                    while count < 3 && i + 1 < bytes.len() && (b'0'..=b'7').contains(&bytes[i + 1])
                    {
                        i += 1;
                        val = val * 8 + (bytes[i] - b'0');
                        count += 1;
                    }
                    out.write_all(&[val]).map_err(|e| e.to_string())?;
                }
                other => {
                    out.write_all(&[b'\\', other]).map_err(|e| e.to_string())?;
                }
            }
            i += 1;
            continue;
        }

        if bytes[i] != b'%' {
            out.write_all(&[bytes[i]]).map_err(|e| e.to_string())?;
            i += 1;
            continue;
        }

        let spec_start = i;
        i += 1;
        if i >= bytes.len() {
            return Err("%: invalid format".to_string());
        }

        if bytes[i] == b'%' {
            out.write_all(b"%").map_err(|e| e.to_string())?;
            i += 1;
            continue;
        }

        let mut flag_minus = false;
        let mut flag_plus = false;
        let mut flag_space = false;
        let mut flag_hash = false;
        let mut flag_zero = false;

        loop {
            if i >= bytes.len() {
                return Err(format!("{}: invalid format", &format[spec_start..]));
            }
            match bytes[i] {
                b'-' => flag_minus = true,
                b'+' => flag_plus = true,
                b' ' => flag_space = true,
                b'#' => flag_hash = true,
                b'0' => flag_zero = true,
                _ => break,
            }
            i += 1;
        }

        let mut width: Option<i32> = None;
        if i < bytes.len() && bytes[i] == b'*' {
            i += 1;
            let w_val = get_int_arg(args, arg_idx, exit_code);
            if w_val < 0 {
                flag_minus = true;
                width = Some(-w_val);
            } else {
                width = Some(w_val);
            }
        } else if i < bytes.len() && bytes[i].is_ascii_digit() {
            let mut w = 0i32;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                w = w * 10 + (bytes[i] - b'0') as i32;
                i += 1;
            }
            width = Some(w);
        }

        let mut precision: Option<i32> = None;
        if i < bytes.len() && bytes[i] == b'.' {
            i += 1;
            if i < bytes.len() && bytes[i] == b'*' {
                i += 1;
                let p_val = get_int_arg(args, arg_idx, exit_code);
                if p_val >= 0 {
                    precision = Some(p_val);
                }
            } else {
                let mut p = 0i32;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    p = p * 10 + (bytes[i] - b'0') as i32;
                    i += 1;
                }
                precision = Some(p);
            }
        }

        while i < bytes.len() && matches!(bytes[i], b'h' | b'l' | b'L' | b'z' | b'j' | b't') {
            i += 1;
        }

        if i >= bytes.len() {
            return Err(format!("{}: invalid format", &format[spec_start..]));
        }

        let conv = bytes[i];
        i += 1;

        match conv {
            b's' => {
                let s_arg = get_str_arg(args, arg_idx);
                let text = if let Some(p) = precision {
                    if (p as usize) < s_arg.len() {
                        &s_arg[..p as usize]
                    } else {
                        s_arg
                    }
                } else {
                    s_arg
                };
                let w = width.unwrap_or(0) as usize;
                if text.len() < w {
                    let pad = w - text.len();
                    if flag_minus {
                        out.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
                        out.write_all(&vec![b' '; pad]).map_err(|e| e.to_string())?;
                    } else {
                        out.write_all(&vec![b' '; pad]).map_err(|e| e.to_string())?;
                        out.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
                    }
                } else {
                    out.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
                }
            }
            b'b' => {
                let s_arg = get_str_arg(args, arg_idx);
                let b_bytes = s_arg.as_bytes();
                let mut bi = 0;
                while bi < b_bytes.len() {
                    if b_bytes[bi] == b'\\' {
                        bi += 1;
                        if bi >= b_bytes.len() {
                            out.write_all(b"\\").map_err(|e| e.to_string())?;
                            break;
                        }
                        match b_bytes[bi] {
                            b'a' => out.write_all(b"\x07").map_err(|e| e.to_string())?,
                            b'b' => out.write_all(b"\x08").map_err(|e| e.to_string())?,
                            b'f' => out.write_all(b"\x0c").map_err(|e| e.to_string())?,
                            b'n' => out.write_all(b"\n").map_err(|e| e.to_string())?,
                            b'r' => out.write_all(b"\r").map_err(|e| e.to_string())?,
                            b't' => out.write_all(b"\t").map_err(|e| e.to_string())?,
                            b'v' => out.write_all(b"\x0b").map_err(|e| e.to_string())?,
                            b'\\' => out.write_all(b"\\").map_err(|e| e.to_string())?,
                            b'c' => {
                                *stop = true;
                                return Ok(());
                            }
                            b'0'..=b'7' => {
                                let mut val = b_bytes[bi] - b'0';
                                let mut count = 1;
                                while count < 3
                                    && bi + 1 < b_bytes.len()
                                    && (b'0'..=b'7').contains(&b_bytes[bi + 1])
                                {
                                    bi += 1;
                                    val = val * 8 + (b_bytes[bi] - b'0');
                                    count += 1;
                                }
                                out.write_all(&[val]).map_err(|e| e.to_string())?;
                            }
                            other => {
                                out.write_all(&[b'\\', other]).map_err(|e| e.to_string())?;
                            }
                        }
                    } else {
                        out.write_all(&[b_bytes[bi]]).map_err(|e| e.to_string())?;
                    }
                    bi += 1;
                }
            }
            b'c' => {
                let s_arg = get_str_arg(args, arg_idx);
                let first_byte = s_arg.as_bytes().first().copied().unwrap_or(0);
                out.write_all(&[first_byte]).map_err(|e| e.to_string())?;
            }
            b'd' | b'i' => {
                let (val, _) = parse_numeric_arg(args.get(*arg_idx).map(|s| s.as_str()), exit_code);
                *arg_idx += 1;
                let int_val = val as i64;
                let mut formatted = format!("{}", int_val);
                if int_val >= 0 {
                    if flag_plus {
                        formatted = format!("+{}", formatted);
                    } else if flag_space {
                        formatted = format!(" {}", formatted);
                    }
                }
                pad_and_print(
                    &formatted,
                    width,
                    flag_minus,
                    flag_zero && !flag_minus && int_val >= 0 && !flag_space,
                    out,
                )?;
            }
            b'u' => {
                let (val, _) = parse_numeric_arg(args.get(*arg_idx).map(|s| s.as_str()), exit_code);
                *arg_idx += 1;
                let u_val = val as u64;
                let formatted = format!("{}", u_val);
                pad_and_print(&formatted, width, flag_minus, flag_zero && !flag_minus, out)?;
            }
            b'o' => {
                let (val, _) = parse_numeric_arg(args.get(*arg_idx).map(|s| s.as_str()), exit_code);
                *arg_idx += 1;
                let u_val = val as u64;
                let formatted = if flag_hash && u_val != 0 {
                    format!("0{:o}", u_val)
                } else {
                    format!("{:o}", u_val)
                };
                pad_and_print(&formatted, width, flag_minus, flag_zero && !flag_minus, out)?;
            }
            b'x' => {
                let (val, _) = parse_numeric_arg(args.get(*arg_idx).map(|s| s.as_str()), exit_code);
                *arg_idx += 1;
                let u_val = val as u64;
                let formatted = if flag_hash && u_val != 0 {
                    format!("0x{:x}", u_val)
                } else {
                    format!("{:x}", u_val)
                };
                pad_and_print(&formatted, width, flag_minus, flag_zero && !flag_minus, out)?;
            }
            b'X' => {
                let (val, _) = parse_numeric_arg(args.get(*arg_idx).map(|s| s.as_str()), exit_code);
                *arg_idx += 1;
                let u_val = val as u64;
                let formatted = if flag_hash && u_val != 0 {
                    format!("0X{:X}", u_val)
                } else {
                    format!("{:X}", u_val)
                };
                pad_and_print(&formatted, width, flag_minus, flag_zero && !flag_minus, out)?;
            }
            b'f' => {
                let (val, _) = parse_numeric_arg(args.get(*arg_idx).map(|s| s.as_str()), exit_code);
                *arg_idx += 1;
                let mut formatted = format_float_libc('f', precision.or(Some(6)), val);
                if val >= 0.0 && flag_plus && !formatted.starts_with('+') {
                    formatted = format!("+{}", formatted);
                } else if val >= 0.0 && flag_space && !formatted.starts_with(' ') {
                    formatted = format!(" {}", formatted);
                }
                pad_and_print(&formatted, width, flag_minus, flag_zero && !flag_minus, out)?;
            }
            b'e' => {
                let (val, _) = parse_numeric_arg(args.get(*arg_idx).map(|s| s.as_str()), exit_code);
                *arg_idx += 1;
                let formatted = format_float_libc('e', precision.or(Some(6)), val);
                pad_and_print(&formatted, width, flag_minus, flag_zero && !flag_minus, out)?;
            }
            b'E' => {
                let (val, _) = parse_numeric_arg(args.get(*arg_idx).map(|s| s.as_str()), exit_code);
                *arg_idx += 1;
                let formatted = format_float_libc('E', precision.or(Some(6)), val);
                pad_and_print(&formatted, width, flag_minus, flag_zero && !flag_minus, out)?;
            }
            b'g' => {
                let (val, _) = parse_numeric_arg(args.get(*arg_idx).map(|s| s.as_str()), exit_code);
                *arg_idx += 1;
                let formatted = format_float_libc('g', precision, val);
                pad_and_print(&formatted, width, flag_minus, false, out)?;
            }
            b'G' => {
                let (val, _) = parse_numeric_arg(args.get(*arg_idx).map(|s| s.as_str()), exit_code);
                *arg_idx += 1;
                let formatted = format_float_libc('G', precision, val);
                pad_and_print(&formatted, width, flag_minus, false, out)?;
            }
            _ => {
                return Err(format!("{}: invalid format", &format[spec_start..i]));
            }
        }
    }

    Ok(())
}

fn get_str_arg<'a>(args: &'a [String], arg_idx: &mut usize) -> &'a str {
    if *arg_idx < args.len() {
        let s = &args[*arg_idx];
        *arg_idx += 1;
        s
    } else {
        ""
    }
}

fn get_int_arg(args: &[String], arg_idx: &mut usize, exit_code: &mut i32) -> i32 {
    let (v, _) = parse_numeric_arg(args.get(*arg_idx).map(|s| s.as_str()), exit_code);
    *arg_idx += 1;
    v as i32
}

fn pad_and_print<W: Write>(
    text: &str,
    width: Option<i32>,
    left_align: bool,
    zero_pad: bool,
    out: &mut W,
) -> std::result::Result<(), String> {
    let w = width.unwrap_or(0) as usize;
    if text.len() < w {
        let pad_len = w - text.len();
        let pad_char = if zero_pad { b'0' } else { b' ' };
        if left_align {
            out.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
            out.write_all(&vec![b' '; pad_len])
                .map_err(|e| e.to_string())?;
        } else {
            out.write_all(&vec![pad_char; pad_len])
                .map_err(|e| e.to_string())?;
            out.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
        }
    } else {
        out.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn format_float_libc(spec: char, prec: Option<i32>, val: f64) -> String {
    use std::ffi::CString;
    let fmt_str = match prec {
        Some(p) => format!("%.{}{}", p, spec),
        None => format!("%{}", spec),
    };
    if let Ok(c_fmt) = CString::new(fmt_str) {
        let mut buf = vec![0u8; 128];
        unsafe {
            let n = libc::snprintf(
                buf.as_mut_ptr() as *mut libc::c_char,
                buf.len(),
                c_fmt.as_ptr(),
                val,
            );
            if n > 0 && (n as usize) < buf.len() {
                buf.truncate(n as usize);
                if let Ok(s) = String::from_utf8(buf) {
                    return s;
                }
            }
        }
    }
    format!("{}", val)
}
