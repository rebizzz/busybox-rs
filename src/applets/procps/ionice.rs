use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;

pub struct IoniceApplet;
impl Applet for IoniceApplet {
    fn name(&self) -> &'static str {
        "ionice"
    }
    fn description(&self) -> &'static str {
        "Set or get I/O scheduling class and priority"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut class: Option<i32> = None;
        let mut level: Option<i32> = None;
        let mut pid: i32 = 0;
        let mut prog: Option<usize> = None;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-c" {
                i += 1;
                if i >= args.len() {
                    eprintln!("ionice: -c needs a class");
                    return Ok(1);
                }
                match std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    Some(c @ 0..=3) => class = Some(c),
                    _ => {
                        eprintln!("ionice: bad class (0-3)");
                        return Ok(1);
                    }
                }
            } else if b.len() == 3 && b[..2] == *b"-c" && b[2].is_ascii_digit() {
                let c = (b[2] - b'0') as i32;
                if c > 3 {
                    eprintln!("ionice: bad class (0-3)");
                    return Ok(1);
                }
                class = Some(c);
            } else if b == b"-n" {
                i += 1;
                if i >= args.len() {
                    eprintln!("ionice: -n needs a level");
                    return Ok(1);
                }
                match std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    Some(n @ 0..=7) => level = Some(n),
                    _ => {
                        eprintln!("ionice: bad level (0-7)");
                        return Ok(1);
                    }
                }
            } else if b.len() == 3 && b[..2] == *b"-n" && b[2].is_ascii_digit() {
                let n = (b[2] - b'0') as i32;
                if n > 7 {
                    eprintln!("ionice: bad level (0-7)");
                    return Ok(1);
                }
                level = Some(n);
            } else if b == b"-p" {
                i += 1;
                if i >= args.len() {
                    eprintln!("ionice: -p needs a pid");
                    return Ok(1);
                }
                match std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    Some(p) => pid = p,
                    None => {
                        eprintln!("ionice: invalid pid");
                        return Ok(1);
                    }
                }
            } else if b.first() == Some(&b'-') {
                eprintln!("ionice: unknown option");
                return Ok(1);
            } else {
                prog = Some(i);
                break;
            }
            i += 1;
        }
        unsafe extern "C" {
            fn syscall(num: libc::c_long, ...) -> libc::c_long;
        }
        const IOPRIO_CLASS_SHIFT: i32 = 13;
        if let Some(pi) = prog {
            if class.is_some() || level.is_some() {
                let c = class.unwrap_or(2);
                let n = level.unwrap_or(4);
                let v = ((c << IOPRIO_CLASS_SHIFT) | n) as libc::c_long;
                let r = unsafe { syscall(libc::SYS_ioprio_set as libc::c_long, 1, 0, v) };
                if r != 0 {
                    eprintln!("ionice: {}", std::io::Error::last_os_error());
                    return Ok(1);
                }
            }
            let cmd: Vec<OsString> = args[pi..].to_vec();
            return Ok(exec_prog(cmd[0].as_bytes(), &cmd[1..]));
        }
        if class.is_none() && level.is_none() {
            let r = unsafe { syscall(libc::SYS_ioprio_get as libc::c_long, 1, pid) };
            if r < 0 {
                eprintln!("ionice: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
            let c = (r >> IOPRIO_CLASS_SHIFT) as i32;
            let n = (r & 0xff) as i32;
            let cname = match c {
                0 => "none",
                1 => "realtime",
                2 => "best-effort",
                3 => "idle",
                _ => "unknown",
            };
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            let mut line = Vec::with_capacity(48);
            line.extend_from_slice(cname.as_bytes());
            line.extend_from_slice(b": prio ");
            push_u64(&mut line, n as u64);
            line.push(b'\n');
            out.write_all(&line)?;
            out.flush()?;
            return Ok(0);
        }
        let c = class.unwrap_or(2);
        let n = level.unwrap_or(4);
        let v = ((c << IOPRIO_CLASS_SHIFT) | n) as libc::c_long;
        let r = unsafe { syscall(libc::SYS_ioprio_set as libc::c_long, 1, pid, v) };
        if r != 0 {
            eprintln!("ionice: {}", std::io::Error::last_os_error());
            return Ok(1);
        }
        Ok(0)
    }
}
