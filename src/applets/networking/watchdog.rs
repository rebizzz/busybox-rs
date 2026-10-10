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

pub struct WatchdogApplet;

impl Applet for WatchdogApplet {
    fn name(&self) -> &'static str {
        "watchdog"
    }
    fn description(&self) -> &'static str {
        "Software watchdog daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dev = "/dev/watchdog".to_string();
        let mut reset_sec = 30u64;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-t" && i + 1 < args.len() {
                i += 1;
                reset_sec = String::from_utf8_lossy(args[i].as_bytes())
                    .parse()
                    .unwrap_or(30);
            } else if !b.starts_with(b"-") {
                dev = String::from_utf8_lossy(b).to_string();
            }
            i += 1;
        }

        let mut f = match OpenOptions::new().write(true).open(&dev) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("watchdog: {}: {}", dev, e);
                return Ok(1);
            }
        };

        loop {
            if f.write_all(b"\0").is_err() || f.flush().is_err() {
                eprintln!("watchdog: ping failed");
                return Ok(1);
            }
            std::thread::sleep(Duration::from_secs(reset_sec));
        }
    }
}
