use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self};
use std::os::unix::ffi::OsStrExt;
use std::process::Command;
use std::env;

pub struct UnshareApplet;
impl Applet for UnshareApplet {
    fn name(&self) -> &'static str {
        "unshare"
    }
    fn description(&self) -> &'static str {
        "Run program with some namespaces unshared from parent"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut flags: libc::c_int = 0;
        let mut cmd_idx = args.len();

        for (i, arg) in args.iter().enumerate() {
            let b = arg.as_bytes();
            if b == b"-m" || b == b"--mount" {
                flags |= libc::CLONE_NEWNS;
            } else if b == b"-u" || b == b"--uts" {
                flags |= libc::CLONE_NEWUTS;
            } else if b == b"-i" || b == b"--ipc" {
                flags |= libc::CLONE_NEWIPC;
            } else if b == b"-n" || b == b"--net" {
                flags |= libc::CLONE_NEWNET;
            } else if b == b"-p" || b == b"--pid" {
                flags |= libc::CLONE_NEWPID;
            } else if b == b"-U" || b == b"--user" {
                flags |= libc::CLONE_NEWUSER;
            } else if !b.starts_with(b"-") {
                cmd_idx = i;
                break;
            }
        }

        if flags != 0 {
            let res = unsafe { libc::unshare(flags) };
            if res != 0 {
                let err = io::Error::last_os_error();
                eprintln!("unshare: {}", err);
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
                eprintln!("unshare: {}: {}", prog, e);
                Ok(127)
            }
        }
    }
}
