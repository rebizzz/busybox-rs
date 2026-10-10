use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CStr, CString, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::mem;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, TcpStream, ToSocketAddrs, UdpSocket};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub struct SlattachApplet;
impl Applet for SlattachApplet {
    fn name(&self) -> &'static str {
        "slattach"
    }
    fn description(&self) -> &'static str {
        "Attach serial line to network interface (SLIP)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut tty_dev: Option<String> = None;

        for arg in args {
            let s = arg.to_string_lossy();
            if !s.starts_with('-') {
                tty_dev = Some(s.to_string());
                break;
            }
        }

        let dev = match tty_dev {
            Some(d) => d,
            None => {
                eprintln!("Usage: slattach [-p protocol] <tty>");
                return Ok(1);
            }
        };

        let path = if dev.starts_with('/') {
            dev
        } else {
            format!("/dev/{}", dev)
        };
        let c_path = match CString::new(path.as_bytes()) {
            Ok(c) => c,
            Err(_) => return Ok(1),
        };

        let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDWR | libc::O_NOCTTY) };
        if fd < 0 {
            eprintln!("slattach: open {}: {}", path, io::Error::last_os_error());
            return Ok(1);
        }

        let ldisc: libc::c_int = 1;
        let ret = unsafe { libc::ioctl(fd, libc::TIOCSETD as _, &ldisc) };
        if ret < 0 {
            eprintln!("slattach: TIOCSETD: {}", io::Error::last_os_error());
            unsafe { libc::close(fd) };
            return Ok(1);
        }

        unsafe { libc::close(fd) };
        Ok(0)
    }
}
