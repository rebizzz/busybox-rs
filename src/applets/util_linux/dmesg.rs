use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self};
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::FromRawFd;

pub struct DmesgApplet;
impl Applet for DmesgApplet {
    fn name(&self) -> &'static str {
        "dmesg"
    }
    fn description(&self) -> &'static str {
        "Print the kernel ring buffer"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut raw = false;
        let mut size: usize = usize::MAX;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-c" {
            } else if b == b"-r" {
                raw = true;
            } else if b == b"-n" {
                i += 1;
                if i >= args.len() {
                    eprintln!("dmesg: option requires an argument -- 'n'");
                    return Ok(1);
                }
            } else if b == b"-s" {
                i += 1;
                if i >= args.len() {
                    eprintln!("dmesg: option requires an argument -- 's'");
                    return Ok(1);
                }
                match std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<usize>().ok())
                {
                    Some(n) => size = n,
                    None => {
                        eprintln!("dmesg: invalid size");
                        return Ok(1);
                    }
                }
            } else {
                eprintln!("dmesg: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            }
            i += 1;
        }
        let path_c = match CString::new("/dev/kmsg") {
            Ok(c) => c,
            Err(_) => return Ok(1),
        };

        let fd = unsafe {
            libc::open(
                path_c.as_ptr(),
                libc::O_RDONLY | libc::O_NONBLOCK | libc::O_CLOEXEC,
            )
        };
        if fd < 0 {
            eprintln!(
                "dmesg: cannot open /dev/kmsg: {}",
                io::Error::last_os_error()
            );
            return Ok(1);
        }

        let mut f = unsafe { fs::File::from_raw_fd(fd) };
        let stdout = io::stdout();
        let mut out = stdout.lock();
        let mut buf = [0u8; 8192];
        let mut total = 0usize;
        let mut tail: Vec<u8> = Vec::new();
        loop {
            if total >= size {
                break;
            }
            let cap = buf.len().min(size - total);
            match f.read(&mut buf[..cap]) {
                Ok(0) => break,
                Ok(n) => {
                    total += n;
                    let mut start = 0;
                    for (k, &c) in buf[..n].iter().enumerate() {
                        if c == b'\n' {
                            tail.extend_from_slice(&buf[start..k]);
                            emit_kmsg(&mut out, &tail, raw)?;
                            tail.clear();
                            start = k + 1;
                        }
                    }
                    tail.extend_from_slice(&buf[start..n]);
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    eprintln!("dmesg: read error: {}", e);
                    return Ok(1);
                }
            }
        }
        if !tail.is_empty() {
            emit_kmsg(&mut out, &tail, raw)?;
        }
        out.flush()?;

        Ok(0)
    }
}

fn emit_kmsg(out: &mut impl Write, rec: &[u8], raw: bool) -> io::Result<()> {
    if raw || rec.is_empty() {
        out.write_all(rec)?;
        out.write_all(b"\n")?;
        return Ok(());
    }

    match rec.iter().position(|&b| b == b';') {
        Some(p) => {
            out.write_all(&rec[p + 1..])?;
            out.write_all(b"\n")?;
        }
        None => {
            let mut msg = rec;
            if msg.starts_with(b"<") {
                if let Some(end) = msg.iter().position(|&b| b == b'>') {
                    msg = &msg[end + 1..];
                }
            }
            out.write_all(msg)?;
            out.write_all(b"\n")?;
        }
    }
    Ok(())
}
