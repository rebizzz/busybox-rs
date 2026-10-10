use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CStr, CString, OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::net::Ipv4Addr;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::Path;

pub struct SetsidApplet;
impl Applet for SetsidApplet {
    fn name(&self) -> &'static str {
        "setsid"
    }
    fn description(&self) -> &'static str {
        "Run a program in a new session"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut prog_at: Option<usize> = None;
        for (k, a) in args.iter().enumerate() {
            if a.as_bytes() == b"-c" || a.as_bytes() == b"-w" {
                continue;
            } else if a.as_bytes().first() == Some(&b'-') {
                eprintln!("setsid: unknown option");
                return Ok(1);
            } else {
                prog_at = Some(k);
                break;
            }
        }

        unsafe {
            if libc::getpgrp() == libc::getpid() {
                let pid = libc::fork();
                if pid < 0 {
                    eprintln!("setsid: {}", std::io::Error::last_os_error());
                    return Ok(1);
                }
                if pid != 0 {
                    libc::_exit(0);
                }
            }
            if libc::setsid() < 0 {
                eprintln!("setsid: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
        }
        match prog_at {
            Some(p) => {
                let cmd: Vec<OsString> = args[p..].to_vec();
                Ok(exec_prog(cmd[0].as_bytes(), &cmd[1..]))
            }
            None => Ok(0),
        }
    }
}
