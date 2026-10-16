use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::io::{self};
use std::os::unix::ffi::OsStrExt;
use std::process::Command;
use std::env;

pub struct NsenterApplet;
impl Applet for NsenterApplet {
    fn name(&self) -> &'static str {
        "nsenter"
    }
    fn description(&self) -> &'static str {
        "Enter name space of other processes"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut target_pid: Option<libc::pid_t> = None;
        let mut nstypes: Vec<(&'static str, libc::c_int)> = Vec::new();
        let mut cmd_idx = args.len();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if (b == b"-t" || b == b"--target") && i + 1 < args.len() {
                target_pid = args[i + 1].to_string_lossy().parse().ok();
                i += 2;
                continue;
            } else if b == b"-m" || b == b"--mount" {
                nstypes.push(("mnt", libc::CLONE_NEWNS));
            } else if b == b"-u" || b == b"--uts" {
                nstypes.push(("uts", libc::CLONE_NEWUTS));
            } else if b == b"-i" || b == b"--ipc" {
                nstypes.push(("ipc", libc::CLONE_NEWIPC));
            } else if b == b"-n" || b == b"--net" {
                nstypes.push(("net", libc::CLONE_NEWNET));
            } else if b == b"-p" || b == b"--pid" {
                nstypes.push(("pid", libc::CLONE_NEWPID));
            } else if b == b"-U" || b == b"--user" {
                nstypes.push(("user", libc::CLONE_NEWUSER));
            } else if !b.starts_with(b"-") {
                cmd_idx = i;
                break;
            }
            i += 1;
        }

        let pid = match target_pid {
            Some(p) => p,
            None => {
                eprintln!("nsenter: -t PID required");
                return Ok(1);
            }
        };

        for (ns_name, clone_flag) in nstypes {
            let ns_path = format!("/proc/{}/ns/{}", pid, ns_name);
            let c_path = CString::new(ns_path).unwrap();
            let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDONLY) };
            if fd < 0 {
                let err = io::Error::last_os_error();
                eprintln!("nsenter: open /proc/{}/ns/{}: {}", pid, ns_name, err);
                return Ok(1);
            }
            let res = unsafe { libc::setns(fd, clone_flag) };
            unsafe { libc::close(fd) };
            if res != 0 {
                let err = io::Error::last_os_error();
                eprintln!("nsenter: setns: {}", err);
                return Ok(1);
            }
        }

        let cmd_parts = if cmd_idx < args.len() {
            &args[cmd_idx..]
        } else {
            &[]
        };

        let prog = if !cmd_parts.is_empty() {
            cmd_parts[0].to_string_lossy().to_string()
        } else {
            env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into())
        };

        let mut cmd = Command::new(&prog);
        if cmd_parts.len() > 1 {
            cmd.args(&cmd_parts[1..]);
        }

        match cmd.status() {
            Ok(st) => Ok(st.code().unwrap_or(0)),
            Err(e) => {
                eprintln!("nsenter: {}: {}", prog, e);
                Ok(127)
            }
        }
    }
}
