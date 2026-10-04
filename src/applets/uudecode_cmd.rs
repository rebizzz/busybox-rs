use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use crate::core::fs::read_bytes_or_stdin;
use crate::core::{Applet, Result};

pub struct UudecodeApplet;
impl Applet for UudecodeApplet {
    fn name(&self) -> &'static str { "uudecode" }
    fn description(&self) -> &'static str { "Decode a uuencoded file" }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut out_file: Option<OsString> = None;
        let mut in_files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();
            if bytes == b"-o" {
                if i + 1 < args.len() {
                    out_file = Some(args[i + 1].clone());
                    i += 2;
                    continue;
                }
            } else if bytes.starts_with(b"-o") {
                use std::os::unix::ffi::OsStringExt;
                out_file = Some(OsString::from_vec(bytes[2..].to_vec()));
            } else {
                in_files.push(Path::new(arg));
            }
            i += 1;
        }

        let in_path = if in_files.is_empty() { Path::new("-") } else { in_files[0] };
        let content = read_bytes_or_stdin(in_path)?;
        let lines: Vec<&[u8]> = content.split(|&b| b == b'\n').collect();

        let mut line_idx = 0;
        let mut is_base64 = false;
        let mut mode = 0o644;
        let mut target_name = String::new();

        while line_idx < lines.len() {
            let line = lines[line_idx];
            line_idx += 1;
            let trimmed = if line.ends_with(b"\r") { &line[..line.len()-1] } else { line };

            if trimmed.starts_with(b"begin-base64 ") {
                is_base64 = true;
                let rest = &trimmed[b"begin-base64 ".len()..];
                let parts: Vec<&[u8]> = rest.splitn(2, |&b| b == b' ').collect();
                if parts.len() == 2 {
                    mode = u32::from_str_radix(std::str::from_utf8(parts[0]).unwrap_or("644"), 8).unwrap_or(0o644);
                    target_name = String::from_utf8_lossy(parts[1]).trim().to_string();
                }
                break;
            } else if trimmed.starts_with(b"begin ") {
                let rest = &trimmed[b"begin ".len()..];
                let parts: Vec<&[u8]> = rest.splitn(2, |&b| b == b' ').collect();
                if parts.len() == 2 {
                    mode = u32::from_str_radix(std::str::from_utf8(parts[0]).unwrap_or("644"), 8).unwrap_or(0o644);
                    target_name = String::from_utf8_lossy(parts[1]).trim().to_string();
                }
                break;
            }
        }

        if target_name.is_empty() {
            eprintln!("uudecode: no 'begin' line");
            return Ok(1);
        }

        let dest_is_stdout = if let Some(ref o) = out_file {
            o.as_bytes() == b"-" || o.as_bytes() == b"/dev/stdout"
        } else {
            target_name == "-" || target_name == "/dev/stdout"
        };

        let mut decoded_data = Vec::new();

        if is_base64 {
            let mut b64_str = String::new();
            while line_idx < lines.len() {
                let line = lines[line_idx];
                line_idx += 1;
                let trimmed = if line.ends_with(b"\r") { &line[..line.len()-1] } else { line };
                if trimmed.starts_with(b"====") { break; }
                b64_str.push_str(&String::from_utf8_lossy(trimmed));
            }
            decoded_data = decode_base64(&b64_str);
        } else {
            while line_idx < lines.len() {
                let line = lines[line_idx];
                line_idx += 1;
                let trimmed = if line.ends_with(b"\r") { &line[..line.len()-1] } else { line };
                if trimmed == b"end" { break; }
                if trimmed.is_empty() || trimmed == b"`" { continue; }

                let len_byte = (trimmed[0].wrapping_sub(0x20)) & 0x3f;
                let nbytes = len_byte as usize;

                let mut row = Vec::new();
                for &b in &trimmed[1..] {
                    row.push((b.wrapping_sub(0x20)) & 0x3f);
                }

                let mut j = 0;
                let mut produced = 0;
                while j + 3 < row.len() && produced < nbytes {
                    let b0 = (row[j] << 2) | (row[j + 1] >> 4);
                    decoded_data.push(b0);
                    produced += 1;
                    if produced < nbytes {
                        let b1 = ((row[j + 1] & 0x0f) << 4) | (row[j + 2] >> 2);
                        decoded_data.push(b1);
                        produced += 1;
                    }
                    if produced < nbytes {
                        let b2 = ((row[j + 2] & 0x03) << 6) | row[j + 3];
                        decoded_data.push(b2);
                        produced += 1;
                    }
                    j += 4;
                }
            }
        }

        if dest_is_stdout {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            handle.write_all(&decoded_data)?;
        } else {
            let path = out_file.as_deref().unwrap_or_else(|| std::ffi::OsStr::new(&target_name));
            let mut f = File::create(path)?;
            f.set_permissions(std::fs::Permissions::from_mode(mode))?;
            f.write_all(&decoded_data)?;
        }

        Ok(0)
    }
}

fn decode_base64(s: &str) -> Vec<u8> {
    const B64_REV: [i8; 128] = [
        -1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,
        -1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,
        -1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,62,-1,-1,-1,63,
        52,53,54,55,56,57,58,59,60,61,-1,-1,-1,-1,-1,-1,
        -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9,10,11,12,13,14,
        15,16,17,18,19,20,21,22,23,24,25,-1,-1,-1,-1,-1,
        -1,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,
        41,42,43,44,45,46,47,48,49,50,51,-1,-1,-1,-1,-1
    ];

    let mut out = Vec::new();
    let mut buf = [0u8; 4];
    let mut count = 0;

    for &b in s.as_bytes() {
        if b == b'=' { break; }
        if (b as usize) < 128 && B64_REV[b as usize] >= 0 {
            buf[count] = B64_REV[b as usize] as u8;
            count += 1;
            if count == 4 {
                out.push((buf[0] << 2) | (buf[1] >> 4));
                out.push((buf[1] << 4) | (buf[2] >> 2));
                out.push((buf[2] << 6) | buf[3]);
                count = 0;
            }
        }
    }
    if count == 2 {
        out.push((buf[0] << 2) | (buf[1] >> 4));
    } else if count == 3 {
        out.push((buf[0] << 2) | (buf[1] >> 4));
        out.push((buf[1] << 4) | (buf[2] >> 2));
    }
    out
}
