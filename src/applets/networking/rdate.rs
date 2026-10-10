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

pub struct RdateApplet;

impl Applet for RdateApplet {
    fn name(&self) -> &'static str {
        "rdate"
    }
    fn description(&self) -> &'static str {
        "Get time from remote host via RFC 868"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut host = None;
        let mut set_time = false;
        let mut print_time = false;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-s" {
                set_time = true;
            } else if b == b"-p" {
                print_time = true;
            } else if !b.starts_with(b"-") {
                host = Some(&args[i]);
            }
            i += 1;
        }

        let host = match host {
            Some(h) => String::from_utf8_lossy(h.as_bytes()).to_string(),
            None => {
                eprintln!("Usage: rdate [-s] [-p] host");
                return Ok(1);
            }
        };

        let dest = format!("{}:37", host);
        let mut stream = match TcpStream::connect(&dest) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("rdate: connect: {}", e);
                return Ok(1);
            }
        };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));

        let mut buf = [0u8; 4];
        if stream.read_exact(&mut buf).is_err() {
            eprintln!("rdate: read error");
            return Ok(1);
        }

        let time_rfc868 = u32::from_be_bytes(buf);
        if time_rfc868 >= 2208988800 {
            let unix_sec = time_rfc868 - 2208988800;
            if print_time || !set_time {
                println!("{}", unix_sec);
            }
            if set_time {
                let tv = libc::timeval {
                    tv_sec: unix_sec as libc::time_t,
                    tv_usec: 0,
                };
                unsafe {
                    libc::settimeofday(&tv, std::ptr::null());
                }
            }
        }

        Ok(0)
    }
}
