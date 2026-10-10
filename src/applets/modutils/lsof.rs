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

pub struct LsofApplet;

impl Applet for LsofApplet {
    fn name(&self) -> &'static str {
        "lsof"
    }
    fn description(&self) -> &'static str {
        "List open files"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let mut out = Vec::new();
        out.extend_from_slice(b"COMMAND     PID USER   FD   TYPE DEVICE SIZE/OFF NODE NAME\n");

        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                let pid_str = file_name.to_string_lossy();
                let pid: u32 = match pid_str.parse() {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                let proc_path = entry.path();
                let comm_bytes = fs::read(proc_path.join("comm")).unwrap_or_default();
                let comm = if let Some(i) = comm_bytes.iter().position(|&b| b == b'\n') {
                    &comm_bytes[..i]
                } else {
                    &comm_bytes[..]
                };

                let fd_dir = proc_path.join("fd");
                if let Ok(fds) = fs::read_dir(&fd_dir) {
                    for fd_entry in fds.flatten() {
                        let fd_name = fd_entry.file_name();
                        if let Ok(target) = fs::read_link(fd_entry.path()) {
                            out.extend_from_slice(comm);
                            if comm.len() < 10 {
                                out.resize(out.len() + 10 - comm.len(), b' ');
                            }
                            out.push(b' ');

                            put_num_pad_left(&mut out, pid as u64, 5);
                            out.extend_from_slice(b" root   ");

                            let fd_b = fd_name.as_bytes();
                            out.extend_from_slice(fd_b);
                            if fd_b.len() < 4 {
                                out.resize(out.len() + 4 - fd_b.len(), b' ');
                            }
                            out.extend_from_slice(b" REG        0,0        0    0 ");
                            out.extend_from_slice(target.as_os_str().as_bytes());
                            out.push(b'\n');
                        }
                    }
                }
            }
        }

        print_bytes(&out);
        Ok(0)
    }
}
