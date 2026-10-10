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

pub struct SetconsoleApplet;

impl Applet for SetconsoleApplet {
    fn name(&self) -> &'static str {
        "setconsole"
    }
    fn description(&self) -> &'static str {
        "Redirect system console output to a device"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut reset = false;
        let mut dev = None;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-r" {
                reset = true;
            } else if !b.starts_with(b"-") {
                dev = Some(&args[i]);
            }
            i += 1;
        }

        let target = if reset {
            "/dev/console"
        } else {
            match dev {
                Some(d) => d.to_str().unwrap_or("/dev/tty"),
                None => "/dev/tty",
            }
        };

        let file = match OpenOptions::new().write(true).open(target) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("setconsole: {}: {}", target, e);
                return Ok(1);
            }
        };

        const TIOCCONS: libc::c_ulong = 0x541D;
        let res = unsafe { libc::ioctl(file.as_raw_fd(), TIOCCONS) };
        if res < 0 {
            let err = io::Error::last_os_error();
            eprintln!("setconsole: ioctl: {}", err);
            return Ok(1);
        }

        Ok(0)
    }
}
