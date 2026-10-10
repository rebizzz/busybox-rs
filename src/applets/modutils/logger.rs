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

pub struct LoggerApplet;

impl Applet for LoggerApplet {
    fn name(&self) -> &'static str {
        "logger"
    }
    fn description(&self) -> &'static str {
        "Log a message to syslog"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut tag: Option<&[u8]> = None;
        let priority: libc::c_int = libc::LOG_USER | libc::LOG_NOTICE;
        let mut msg_parts: Vec<&[u8]> = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-t" && i + 1 < args.len() {
                i += 1;
                tag = Some(args[i].as_bytes());
            } else if b == b"-p" && i + 1 < args.len() {
                i += 1;
            } else if !b.is_empty() && b[0] == b'-' {
            } else {
                msg_parts.push(b);
            }
            i += 1;
        }

        let message = if msg_parts.is_empty() {
            let mut buf = Vec::new();
            let _ = io::stdin().read_to_end(&mut buf);
            buf
        } else {
            let mut buf = Vec::new();
            for (idx, part) in msg_parts.iter().enumerate() {
                if idx > 0 {
                    buf.push(b' ');
                }
                buf.extend_from_slice(part);
            }
            buf
        };

        let log_sock_path = Path::new("/dev/log");
        if log_sock_path.exists() {
            if let Ok(sock) = UnixDatagram::unbound() {
                let tag_str = tag.unwrap_or(b"logger");
                let mut formatted = Vec::new();
                formatted.extend_from_slice(b"<13>");
                formatted.extend_from_slice(tag_str);
                formatted.extend_from_slice(b": ");
                formatted.extend_from_slice(&message);
                if sock.send_to(&formatted, log_sock_path).is_ok() {
                    return Ok(0);
                }
            }
        }

        let tag_cstr = tag
            .and_then(|t| CString::new(t).ok())
            .unwrap_or_else(|| CString::new("logger").unwrap());
        let msg_cstr = CString::new(message).unwrap_or_else(|_| CString::new("").unwrap());

        unsafe {
            libc::openlog(tag_cstr.as_ptr(), libc::LOG_PID, libc::LOG_USER);
            libc::syslog(priority, c"%s".as_ptr(), msg_cstr.as_ptr());
            libc::closelog();
        }

        Ok(0)
    }
}
