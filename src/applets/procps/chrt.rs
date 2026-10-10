use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CStr, CString, OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::net::Ipv4Addr;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::Path;

pub struct ChrtApplet;
impl Applet for ChrtApplet {
    fn name(&self) -> &'static str {
        "chrt"
    }
    fn description(&self) -> &'static str {
        "Get or set real-time scheduling attributes"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut pid_mode = false;
        let mut policy: Option<i32> = None;
        let prio: i32;
        let mut max = false;
        let mut rest: Vec<OsString> = Vec::new();
        for a in args {
            let b = a.as_bytes();
            if b == b"-p" {
                pid_mode = true;
            } else if b == b"-f" {
                policy = Some(libc::SCHED_FIFO);
            } else if b == b"-r" {
                policy = Some(libc::SCHED_RR);
            } else if b == b"-o" {
                policy = Some(libc::SCHED_OTHER);
            } else if b == b"-b" {
                policy = Some(libc::SCHED_BATCH);
            } else if b == b"-i" {
                policy = Some(libc::SCHED_IDLE);
            } else if b == b"-m" {
                max = true;
            } else if b.first() == Some(&b'-') {
                eprintln!("chrt: unknown option");
                return Ok(1);
            } else {
                rest.push(a.clone());
            }
        }
        if max {
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            for (pol, nm) in [
                (libc::SCHED_FIFO, "SCHED_FIFO"),
                (libc::SCHED_RR, "SCHED_RR"),
                (libc::SCHED_BATCH, "SCHED_BATCH"),
                (libc::SCHED_IDLE, "SCHED_IDLE"),
                (libc::SCHED_OTHER, "SCHED_OTHER"),
            ] {
                let mn = unsafe { libc::sched_get_priority_min(pol) };
                let mx = unsafe { libc::sched_get_priority_max(pol) };
                let mut line = Vec::with_capacity(48);
                line.extend_from_slice(nm.as_bytes());
                line.extend_from_slice(b" min/max priority\t: ");
                push_u64(&mut line, mn.max(0) as u64);
                line.push(b'/');
                push_u64(&mut line, mx.max(0) as u64);
                line.push(b'\n');
                out.write_all(&line)?;
            }
            out.flush()?;
            return Ok(0);
        }
        let show = |pid: i32| -> Result<i32> {
            let p = unsafe { libc::sched_getscheduler(pid) };
            if p < 0 {
                eprintln!("chrt: {}: {}", pid, std::io::Error::last_os_error());
                return Ok(1);
            }
            let mut sp: libc::sched_param = unsafe { std::mem::zeroed() };
            if unsafe { libc::sched_getparam(pid, &mut sp) } != 0 {
                eprintln!("chrt: {}: {}", pid, std::io::Error::last_os_error());
                return Ok(1);
            }
            let nm = match p {
                libc::SCHED_FIFO => "SCHED_FIFO",
                libc::SCHED_RR => "SCHED_RR",
                libc::SCHED_BATCH => "SCHED_BATCH",
                libc::SCHED_IDLE => "SCHED_IDLE",
                _ => "SCHED_OTHER",
            };
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            let mut line = Vec::with_capacity(64);
            line.extend_from_slice(b"pid ");
            push_u64(&mut line, pid as u64);
            line.extend_from_slice(b"'s current scheduling policy: ");
            line.extend_from_slice(nm.as_bytes());
            line.push(b'\n');
            out.write_all(&line)?;
            let mut l2 = Vec::with_capacity(64);
            l2.extend_from_slice(b"pid ");
            push_u64(&mut l2, pid as u64);
            l2.extend_from_slice(b"'s current scheduling priority: ");
            push_u64(&mut l2, sp.sched_priority.max(0) as u64);
            l2.push(b'\n');
            out.write_all(&l2)?;
            out.flush()?;
            Ok(0)
        };
        if pid_mode {
            let mut nums: Vec<i32> = Vec::new();
            for r in &rest {
                match std::str::from_utf8(r.as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    Some(n) => nums.push(n),
                    None => {
                        eprintln!("usage: chrt -p [-f|-r|-o|-b|-i] [prio] pid");
                        return Ok(1);
                    }
                }
            }
            if policy.is_none() && nums.len() == 1 {
                return show(nums[0]);
            }
            let (pr, target) = match nums.len() {
                1 => (0, nums[0]),
                2 => (nums[0], nums[1]),
                _ => {
                    eprintln!("usage: chrt -p [-f|-r|-o|-b|-i] [prio] pid");
                    return Ok(1);
                }
            };
            let pol = policy.unwrap_or(libc::SCHED_RR);
            let mut sp: libc::sched_param = unsafe { std::mem::zeroed() };
            sp.sched_priority = pr;
            if unsafe { libc::sched_setscheduler(target, pol, &sp) } != 0 {
                eprintln!("chrt: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
            return Ok(0);
        }
        if rest.is_empty() {
            eprintln!("usage: chrt [-p pid] [-f|-r|-o|-b|-i] [prio] pid|cmd...");
            return Ok(1);
        }

        let pol = policy.unwrap_or(libc::SCHED_OTHER);
        let first_num = std::str::from_utf8(rest[0].as_bytes())
            .ok()
            .and_then(|s| s.parse::<i32>().ok());
        if rest.len() >= 2 && first_num.is_some() {
            prio = first_num.unwrap_or(0);

            if rest.len() == 2
                && std::str::from_utf8(rest[1].as_bytes())
                    .ok()
                    .and_then(|s| s.parse::<i32>().ok())
                    .is_some()
            {
                let target: i32 = std::str::from_utf8(rest[1].as_bytes())
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let mut sp: libc::sched_param = unsafe { std::mem::zeroed() };
                sp.sched_priority = prio;
                if unsafe { libc::sched_setscheduler(target, pol, &sp) } != 0 {
                    eprintln!("chrt: {}", std::io::Error::last_os_error());
                    return Ok(1);
                }
                return Ok(0);
            }
            let mut sp: libc::sched_param = unsafe { std::mem::zeroed() };
            sp.sched_priority = prio;
            if unsafe { libc::sched_setscheduler(0, pol, &sp) } != 0 {
                eprintln!("chrt: {}", std::io::Error::last_os_error());
                return Ok(1);
            }
            let cmd: Vec<OsString> = rest[1..].to_vec();
            return Ok(exec_prog(cmd[0].as_bytes(), &cmd[1..]));
        }

        if rest.len() == 1
            && std::str::from_utf8(rest[0].as_bytes())
                .ok()
                .and_then(|s| s.parse::<i32>().ok())
                .is_some()
        {
            let pid: i32 = std::str::from_utf8(rest[0].as_bytes())
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            return show(pid);
        }
        eprintln!("usage: chrt [-p pid] [-f|-r|-o|-b|-i] [prio] pid|cmd...");
        Ok(1)
    }
}
