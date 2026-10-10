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

pub struct WhoisApplet;
impl Applet for WhoisApplet {
    fn name(&self) -> &'static str {
        "whois"
    }
    fn description(&self) -> &'static str {
        "Query WHOIS database"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut server = "whois.iana.org".to_string();
        let mut port = 43u16;
        let mut query: Option<String> = None;

        let mut i = 0;
        while i < args.len() {
            let s = args[i].to_string_lossy();
            if s == "-h" && i + 1 < args.len() {
                server = args[i + 1].to_string_lossy().to_string();
                i += 1;
            } else if s == "-p" && i + 1 < args.len() {
                port = args[i + 1].to_string_lossy().parse().unwrap_or(43);
                i += 1;
            } else if !s.starts_with('-') {
                query = Some(s.to_string());
            }
            i += 1;
        }

        let q = match query {
            Some(q) => q,
            None => {
                eprintln!("Usage: whois [-h server] [-p port] <query>");
                return Ok(1);
            }
        };

        let mut stream = match TcpStream::connect((server.as_str(), port)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("whois: connect {}:{}: {}", server, port, e);
                return Ok(1);
            }
        };

        let req = format!("{}\r\n", q);
        if let Err(e) = stream.write_all(req.as_bytes()) {
            eprintln!("whois: write: {}", e);
            return Ok(1);
        }
        let _ = stream.flush();

        let stdout = io::stdout();
        let mut out = stdout.lock();
        let mut buf = [0u8; 4096];
        while let Ok(n) = stream.read(&mut buf) {
            if n == 0 {
                break;
            }
            let _ = out.write_all(&buf[..n]);
        }

        Ok(0)
    }
}
