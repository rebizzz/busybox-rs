use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self};
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::process::Command;

pub struct FlockApplet;
impl Applet for FlockApplet {
    fn name(&self) -> &'static str {
        "flock"
    }
    fn description(&self) -> &'static str {
        "Lock a file and run a command"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut shared = false;
        let mut nonblock = false;
        let mut unlock = false;
        let mut pos: Vec<&[u8]> = Vec::new();
        for a in args {
            let b = a.as_bytes();
            if b.len() > 1 && b.starts_with(b"-") && !b.starts_with(b"--") && b != b"-" {
                for &c in &b[1..] {
                    match c {
                        b's' => shared = true,
                        b'x' => shared = false,
                        b'n' => nonblock = true,
                        b'u' => unlock = true,
                        _ => {
                            eprintln!("flock: invalid option -- '{}'", c as char);
                            return Ok(1);
                        }
                    }
                }
            } else {
                pos.push(b);
            }
        }
        if pos.is_empty() {
            eprintln!("flock: no file specified");
            return Ok(1);
        }
        let mut op: libc::c_int = if unlock {
            libc::LOCK_UN
        } else if shared {
            libc::LOCK_SH
        } else {
            libc::LOCK_EX
        };
        if nonblock {
            op |= libc::LOCK_NB;
        }
        let target = pos[0];
        let cmd_args = &pos[1..];

        if !target.is_empty() && target.iter().all(|c| c.is_ascii_digit()) {
            let fd: i32 = std::str::from_utf8(target)
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(-1);
            if fd < 0 {
                eprintln!("flock: bad file descriptor");
                return Ok(1);
            }

            if unsafe { libc::flock(fd, op) } != 0 {
                eprintln!(
                    "flock: {}: {}",
                    String::from_utf8_lossy(target),
                    io::Error::last_os_error()
                );
                return Ok(1);
            }
            if cmd_args.is_empty() {
                return Ok(0);
            }
            return run_cmd(cmd_args);
        }
        let path = Path::new(std::ffi::OsStr::from_bytes(target));
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path);
        let file = match file {
            Ok(f) => f,
            Err(e) => {
                eprintln!("flock: {}: {}", String::from_utf8_lossy(target), e);
                return Ok(1);
            }
        };
        use std::os::unix::io::AsRawFd;

        if unsafe { libc::flock(file.as_raw_fd(), op) } != 0 {
            eprintln!(
                "flock: {}: {}",
                String::from_utf8_lossy(target),
                io::Error::last_os_error()
            );
            return Ok(1);
        }
        if cmd_args.is_empty() {
            return Ok(0);
        }
        let rc = run_cmd(cmd_args);
        drop(file);
        rc
    }
}

fn run_cmd(cmd_args: &[&[u8]]) -> Result<i32> {
    let prog = String::from_utf8_lossy(cmd_args[0]).into_owned();
    let rest: Vec<String> = cmd_args[1..]
        .iter()
        .map(|s| String::from_utf8_lossy(s).into_owned())
        .collect();
    match Command::new(&prog).args(&rest).status() {
        Ok(st) => Ok(st.code().unwrap_or(1)),
        Err(e) => {
            eprintln!("flock: {}: {}", prog, e);
            Ok(127)
        }
    }
}
