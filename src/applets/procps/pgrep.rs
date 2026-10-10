use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CStr, CString, OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::net::Ipv4Addr;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::Path;

pub struct PgrepApplet;
impl Applet for PgrepApplet {
    fn name(&self) -> &'static str {
        "pgrep"
    }
    fn description(&self) -> &'static str {
        "Look up processes by name"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let (o, _) = match parse_pgrep(args, false) {
            Ok(v) => v,
            Err(rc) => return Ok(rc),
        };
        let m = collect_matches(&o);
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        if o.count {
            let mut line = Vec::with_capacity(16);
            push_u64(&mut line, m.len() as u64);
            line.push(b'\n');
            out.write_all(&line)?;
            out.flush()?;
            return Ok(if m.is_empty() { 1 } else { 0 });
        }
        for (pid, comm, _) in &m {
            let mut line = Vec::with_capacity(64);
            push_u64(&mut line, *pid as u64);
            if o.list {
                line.push(b' ');
                line.extend_from_slice(comm);
            }
            line.push(b'\n');
            out.write_all(&line)?;
        }
        out.flush()?;
        Ok(if m.is_empty() { 1 } else { 0 })
    }
}
