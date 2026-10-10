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

pub struct PscanApplet;

impl Applet for PscanApplet {
    fn name(&self) -> &'static str {
        "pscan"
    }
    fn description(&self) -> &'static str {
        "Scan a host's ports"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut start_port = 1u16;
        let mut end_port = 1024u16;
        let mut host = None;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-p" && i + 1 < args.len() {
                i += 1;
                let s = String::from_utf8_lossy(args[i].as_bytes());
                if let Some(dash) = s.find('-') {
                    start_port = s[..dash].parse().unwrap_or(1);
                    end_port = s[dash + 1..].parse().unwrap_or(1024);
                } else if let Ok(p) = s.parse::<u16>() {
                    start_port = p;
                    end_port = p;
                }
            } else if !b.starts_with(b"-") {
                host = Some(&args[i]);
            }
            i += 1;
        }

        let host = match host {
            Some(h) => String::from_utf8_lossy(h.as_bytes()).to_string(),
            None => {
                eprintln!("Usage: pscan [-p start-end] host");
                return Ok(1);
            }
        };

        println!("Scanning {} ports {} to {}", host, start_port, end_port);

        for port in start_port..=end_port {
            let target = format!("{}:{}", host, port);
            if let Ok(addrs) = std::net::ToSocketAddrs::to_socket_addrs(&target) {
                for addr in addrs {
                    if TcpStream::connect_timeout(&addr, Duration::from_millis(150)).is_ok() {
                        println!("{:5} open", port);
                        break;
                    }
                }
            }
        }

        Ok(0)
    }
}
