use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::process::CommandExt;
use std::process::Command;

pub struct SetprivApplet;
impl Applet for SetprivApplet {
    fn name(&self) -> &'static str {
        "setpriv"
    }
    fn description(&self) -> &'static str {
        "Drop privileges (--reuid/--regid/--clear-groups subset) then exec"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut uid: Option<u32> = None;
        let mut gid: Option<u32> = None;
        let mut clear_groups = false;
        let mut prog_at: Option<usize> = None;
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            let s = String::from_utf8_lossy(b).into_owned();
            if s == "--" {
                prog_at = Some(i + 1);
                break;
            }
            if let Some(v) = s.strip_prefix("--reuid=") {
                uid = pw_lookup(v).map(|(u, _)| u).or_else(|| v.parse().ok());
            } else if s == "--reuid" && i + 1 < args.len() {
                let v = args[i + 1].to_string_lossy().into_owned();
                uid = pw_lookup(&v).map(|(u, _)| u).or_else(|| v.parse().ok());
                i += 1;
            } else if let Some(v) = s.strip_prefix("--regid=") {
                gid = gr_lookup(v).or_else(|| v.parse().ok());
            } else if s == "--regid" && i + 1 < args.len() {
                let v = args[i + 1].to_string_lossy().into_owned();
                gid = gr_lookup(&v).or_else(|| v.parse().ok());
                i += 1;
            } else if s == "--clear-groups" {
                clear_groups = true;
            } else if s == "--reset-env" {
            } else if s.starts_with("--") {
                eprintln!("setpriv: unsupported option {s}");
                return Ok(1);
            } else {
                prog_at = Some(i);
                break;
            }
            i += 1;
        }
        let at = match prog_at {
            Some(a) if a < args.len() => a,
            _ => {
                eprintln!("setpriv: missing prog");
                return Ok(1);
            }
        };
        let mut cmd = Command::new(&args[at]);
        cmd.args(&args[at + 1..]);
        unsafe {
            cmd.pre_exec(move || {
                if clear_groups && libc::setgroups(0, std::ptr::null()) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                if let Some(g) = gid {
                    if libc::setgid(g) != 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                }
                if let Some(u) = uid {
                    if libc::setuid(u) != 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                }
                Ok(())
            });
        }
        let err = cmd.exec();
        eprintln!("setpriv: {}: {err}", args[at].to_string_lossy());
        Ok(1)
    }
}
