use super::common::*;
use crate::core::{Applet, Result};
use std::collections::HashMap;
use std::ffi::{CStr, CString, OsString};
use std::fs::{self, File};
use std::io::{self, BufRead, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixDatagram;
use std::path::{Path, PathBuf};

pub struct LsmodApplet;

impl Applet for LsmodApplet {
    fn name(&self) -> &'static str {
        "lsmod"
    }
    fn description(&self) -> &'static str {
        "List loaded kernel modules"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let content = match fs::read("/proc/modules") {
            Ok(c) => c,
            Err(e) => {
                eprintln!("lsmod: cannot open /proc/modules: {}", e);
                return Ok(1);
            }
        };

        let mut out = Vec::new();
        out.extend_from_slice(b"Module                  Size  Used by\n");

        for line in content.split(|&b| b == b'\n') {
            if line.is_empty() {
                continue;
            }
            let fields: Vec<&[u8]> = line
                .split(|&b| b == b' ' || b == b'\t')
                .filter(|s| !s.is_empty())
                .collect();
            if fields.len() < 3 {
                continue;
            }
            let name = fields[0];
            let size = fields[1];
            let refcnt = fields[2];
            let used_by = if fields.len() > 3 && fields[3] != b"-" {
                let s = fields[3];
                if s.ends_with(b",") {
                    &s[..s.len() - 1]
                } else {
                    s
                }
            } else {
                b""
            };

            out.extend_from_slice(name);
            let n_len = name.len();
            if n_len < 19 {
                out.resize(out.len() + 19 - n_len, b' ');
            }
            out.push(b' ');

            let s_len = size.len();
            if s_len < 8 {
                out.resize(out.len() + 8 - s_len, b' ');
            }
            out.extend_from_slice(size);
            out.push(b' ');

            let r_len = refcnt.len();
            if r_len < 2 {
                out.resize(out.len() + 2 - r_len, b' ');
            }
            out.extend_from_slice(refcnt);

            if !used_by.is_empty() {
                out.push(b' ');
                out.extend_from_slice(used_by);
            }
            out.push(b'\n');
        }

        print_bytes(&out);
        Ok(0)
    }
}
