use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CStr, CString, OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::net::Ipv4Addr;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::Path;

pub struct PsApplet;
impl Applet for PsApplet {
    fn name(&self) -> &'static str {
        "ps"
    }
    fn description(&self) -> &'static str {
        "Report process status"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        out.write_all(b"  PID STAT COMMAND\n")?;
        let mut scratch = [0u8; 512];
        let mut line: Vec<u8> = Vec::with_capacity(96);
        let mut cmd = [0u8; 256];
        for_each_proc(&mut scratch, |p| {
            line.clear();
            line.extend_from_slice(b"  ");
            push_u64(&mut line, p.pid as u64);
            line.push(b' ');
            line.push(p.state);
            line.push(b' ');

            let mut plen = 6;
            let mut path = [0u8; 32];
            path[..6].copy_from_slice(b"/proc/");
            let mut tmp = [0u8; 12];
            let mut tl = 0;
            let mut v = p.pid;
            if v == 0 {
                tmp[tl] = b'0';
                tl += 1;
            } else {
                let mut rev = [0u8; 12];
                let mut rn = 0;
                while v > 0 {
                    rev[rn] = b'0' + (v % 10) as u8;
                    v /= 10;
                    rn += 1;
                }
                while rn > 0 {
                    rn -= 1;
                    tmp[tl] = rev[rn];
                    tl += 1;
                }
            }
            for &c in &tmp[..tl] {
                if plen + 9 < path.len() {
                    path[plen] = c;
                    plen += 1;
                }
            }
            path[plen..plen + 8].copy_from_slice(b"/cmdline");
            let ps = std::str::from_utf8(&path[..plen + 8]).unwrap_or("");
            let n = read_small(ps, &mut cmd);
            if n > 0 {
                for &c in &cmd[..n] {
                    line.push(if c == 0 { b' ' } else { c });
                }
                while line.last() == Some(&b' ') {
                    line.pop();
                }
            } else {
                line.push(b'[');
                line.extend_from_slice(p.comm);
                line.push(b']');
            }
            line.push(b'\n');
            let _ = out.write_all(&line);
        });
        out.flush()?;
        Ok(0)
    }
}
