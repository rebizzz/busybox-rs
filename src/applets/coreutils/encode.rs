use crate::core::fs::read_bytes_or_stdin;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub struct UuencodeApplet;
impl Applet for UuencodeApplet {
    fn name(&self) -> &'static str {
        "uuencode"
    }
    fn description(&self) -> &'static str {
        "Encode a file into email friendly format"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut base64_mode = false;
        let mut pos_args = Vec::new();

        for arg in args {
            if arg.as_bytes() == b"-m" {
                base64_mode = true;
            } else {
                pos_args.push(Path::new(arg));
            }
        }

        if pos_args.is_empty() {
            eprintln!("uuencode: missing operand");
            return Ok(1);
        }

        let (in_file, name) = if pos_args.len() == 1 {
            (Path::new("-"), pos_args[0])
        } else {
            (pos_args[0], pos_args[1])
        };

        let (content, mode) = if in_file.as_os_str() == "-" {
            let bytes = read_bytes_or_stdin(Path::new("-"))?;
            let umask = crate::core::platform::get_umask();
            let file_mode = 0o666 & !umask;
            (bytes, file_mode)
        } else {
            let meta = std::fs::metadata(in_file)?;
            let file_mode = meta.permissions().mode() & 0o777;
            let bytes = std::fs::read(in_file)?;
            (bytes, file_mode)
        };

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        if base64_mode {
            writeln!(handle, "begin-base64 {:o} {}", mode, name.display())?;
            let b64 = base64_encode(&content);
            let mut idx = 0;
            while idx < b64.len() {
                let end = (idx + 60).min(b64.len());
                writeln!(handle, "{}", &b64[idx..end])?;
                idx = end;
            }
            writeln!(handle, "====")?;
        } else {
            writeln!(handle, "begin {:o} {}", mode, name.display())?;
            let mut idx = 0;
            while idx < content.len() {
                let chunk_len = (content.len() - idx).min(45);
                let chunk = &content[idx..idx + chunk_len];
                handle.write_all(&[uu_char(chunk_len as u8)])?;

                let mut j = 0;
                while j < chunk.len() {
                    let b0 = chunk[j];
                    let b1 = if j + 1 < chunk.len() { chunk[j + 1] } else { 0 };
                    let b2 = if j + 2 < chunk.len() { chunk[j + 2] } else { 0 };

                    let c0 = uu_char(b0 >> 2);
                    let c1 = uu_char(((b0 & 0x03) << 4) | (b1 >> 4));
                    let c2 = uu_char(((b1 & 0x0f) << 2) | (b2 >> 6));
                    let c3 = uu_char(b2 & 0x3f);

                    handle.write_all(&[c0, c1, c2, c3])?;
                    j += 3;
                }
                handle.write_all(b"\n")?;
                idx += chunk_len;
            }
            writeln!(handle, "`")?;
            writeln!(handle, "end")?;
        }
        Ok(0)
    }
}

fn base64_encode(data: &[u8]) -> String {
    const B64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i];
        let b1 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] } else { 0 };

        let idx0 = (b0 >> 2) as usize;
        let idx1 = (((b0 & 0x03) << 4) | (b1 >> 4)) as usize;
        let idx2 = (((b1 & 0x0f) << 2) | (b2 >> 6)) as usize;
        let idx3 = (b2 & 0x3f) as usize;

        out.push(B64_CHARS[idx0] as char);
        out.push(B64_CHARS[idx1] as char);
        if i + 1 < data.len() {
            out.push(B64_CHARS[idx2] as char);
        } else {
            out.push('=');
        }
        if i + 2 < data.len() {
            out.push(B64_CHARS[idx3] as char);
        } else {
            out.push('=');
        }
        i += 3;
    }
    out
}

fn uu_char(b: u8) -> u8 {
    let v = b & 0x3f;
    if v == 0 {
        b'`'
    } else {
        v + 32
    }
}
