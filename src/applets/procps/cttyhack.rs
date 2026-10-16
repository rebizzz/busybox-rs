use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

pub struct CttyhackApplet;
impl Applet for CttyhackApplet {
    fn name(&self) -> &'static str {
        "cttyhack"
    }
    fn description(&self) -> &'static str {
        "Give a shell a controlling terminal"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut at: Option<usize> = None;
        for (k, a) in args.iter().enumerate() {
            if a.as_bytes() == b"-v" {
                continue;
            } else if a.as_bytes().first() == Some(&b'-') {
                eprintln!("cttyhack: unknown option");
                return Ok(1);
            } else {
                at = Some(k);
                break;
            }
        }
        let p = match at {
            Some(v) => v,
            None => {
                eprintln!("usage: cttyhack PROG [ARGS...]");
                return Ok(1);
            }
        };
        unsafe {
            if libc::getpgrp() == libc::getpid() {
                let pid = libc::fork();
                if pid < 0 {
                    eprintln!("cttyhack: {}", std::io::Error::last_os_error());
                    return Ok(1);
                }
                if pid != 0 {
                    let mut st = 0;
                    libc::waitpid(pid, &mut st, 0);
                    if libc::WIFEXITED(st) {
                        return Ok(libc::WEXITSTATUS(st));
                    }
                    return Ok(1);
                }
            }
            if libc::setsid() < 0 {
                eprintln!("cttyhack: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
            for fd in 0..3 {
                libc::ioctl(fd, libc::TIOCSCTTY as libc::c_ulong, 1);
            }
        }
        let cmd: Vec<OsString> = args[p..].to_vec();
        Ok(exec_prog(cmd[0].as_bytes(), &cmd[1..]))
    }
}
