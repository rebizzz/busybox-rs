use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CStr, CString, OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::net::Ipv4Addr;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::Path;

pub struct PwdxApplet;
impl Applet for PwdxApplet {
    fn name(&self) -> &'static str {
        "pwdx"
    }
    fn description(&self) -> &'static str {
        "Report current working directory of processes"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("usage: pwdx pid...");
            return Ok(1);
        }
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let mut rc = 0;
        for a in args {
            let pid: u32 = match std::str::from_utf8(a.as_bytes())
                .ok()
                .and_then(|s| s.parse().ok())
            {
                Some(p) => p,
                None => {
                    eprintln!("pwdx: invalid pid");
                    rc = 1;
                    continue;
                }
            };
            let mut path = [0u8; 48];
            let n = proc_path(pid, b"/cwd", &mut path);
            let ps = match std::str::from_utf8(&path[..n]) {
                Ok(s) => s,
                Err(_) => {
                    rc = 1;
                    continue;
                }
            };
            match std::fs::read_link(ps) {
                Ok(target) => {
                    let mut line = Vec::with_capacity(64);
                    push_u64(&mut line, pid as u64);
                    line.extend_from_slice(b": ");
                    line.extend_from_slice(target.as_os_str().as_bytes());
                    line.push(b'\n');
                    out.write_all(&line)?;
                }
                Err(e) => {
                    eprintln!("pwdx: {}: {}", pid, e);
                    rc = 1;
                }
            }
        }
        out.flush()?;
        Ok(rc)
    }
}
