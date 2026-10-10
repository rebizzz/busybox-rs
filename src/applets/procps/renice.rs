use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CStr, CString, OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::net::Ipv4Addr;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::Path;

pub struct ReniceApplet;
impl Applet for ReniceApplet {
    fn name(&self) -> &'static str {
        "renice"
    }
    fn description(&self) -> &'static str {
        "Alter priority of running processes"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut which = libc::PRIO_PROCESS;
        let mut prio: Option<i32> = None;
        let mut ids: Vec<i32> = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-n" {
                i += 1;
                if i >= args.len() {
                    eprintln!("renice: -n needs a priority");
                    return Ok(1);
                }
                match std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    Some(p) => prio = Some(p),
                    None => {
                        eprintln!("renice: invalid priority");
                        return Ok(1);
                    }
                }
            } else if b == b"-g" {
                which = libc::PRIO_PGRP;
            } else if b == b"-p" {
                which = libc::PRIO_PROCESS;
            } else if b == b"-u" {
                which = libc::PRIO_USER;
            } else if b.len() > 1
                && b[0] == b'-'
                && (b[1].is_ascii_digit() || b[1] == b'+' || b[1] == b'-')
            {
                match std::str::from_utf8(b)
                    .ok()
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    Some(p) => prio = Some(p),
                    None => {
                        eprintln!("renice: invalid priority");
                        return Ok(1);
                    }
                }
            } else if prio.is_none() && (b[0].is_ascii_digit() || b[0] == b'+' || b[0] == b'-') {
                match std::str::from_utf8(b)
                    .ok()
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    Some(p) => prio = Some(p),
                    None => {
                        eprintln!("renice: invalid priority");
                        return Ok(1);
                    }
                }
            } else {
                if which == libc::PRIO_USER {
                    match uid_of_name(b) {
                        Some(u) => ids.push(u as i32),
                        None => {
                            eprintln!("renice: unknown user");
                            return Ok(1);
                        }
                    }
                } else {
                    match std::str::from_utf8(b)
                        .ok()
                        .and_then(|s| s.parse::<i32>().ok())
                    {
                        Some(p) => ids.push(p),
                        None => {
                            eprintln!("renice: invalid id");
                            return Ok(1);
                        }
                    }
                }
            }
            i += 1;
        }
        let prio = match prio {
            Some(p) => p,
            None => {
                eprintln!("usage: renice [-n] prio [-g|-p|-u] id...");
                return Ok(1);
            }
        };
        if ids.is_empty() {
            eprintln!("usage: renice [-n] prio [-g|-p|-u] id...");
            return Ok(1);
        }
        let mut rc = 0;
        for id in ids {
            if unsafe { libc::setpriority(which, id as u32, prio) } != 0 {
                eprintln!("renice: {}: {}", id, std::io::Error::last_os_error());
                rc = 1;
            }
        }
        Ok(rc)
    }
}
