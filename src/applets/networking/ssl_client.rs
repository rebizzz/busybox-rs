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

pub struct SslClientApplet;
impl Applet for SslClientApplet {
    fn name(&self) -> &'static str {
        "ssl_client"
    }
    fn description(&self) -> &'static str {
        "TLS client wrapper"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut host = String::new();
        let mut port = 443u16;
        let mut prog: Vec<OsString> = Vec::new();
        let mut idx = 0;
        while idx < args.len() {
            let b = args[idx].as_bytes();
            if b == b"-s" || b == b"-r" || b == b"-n" {
                idx += 1;
            } else if b == b"-e" {
                prog = args[idx + 1..].to_vec();
                break;
            } else if !b.starts_with(b"-") {
                let s = args[idx].to_string_lossy();
                if let Some((h, p)) = s.split_once(':') {
                    host = h.to_string();
                    port = p.parse().unwrap_or(443);
                } else {
                    host = s.to_string();
                }
            }
            idx += 1;
        }
        if host.is_empty() && prog.is_empty() {
            eprintln!(
                "Usage: ssl_client [-n SNI] {{ -s FD [-r FD] | HOST[:PORT] | -e PROG ARGS }}"
            );
            return Ok(1);
        }
        if !prog.is_empty() {
            let mut cmd = std::process::Command::new(&prog[0]);
            cmd.args(&prog[1..]);
            match cmd.status() {
                Ok(st) => return Ok(st.code().unwrap_or(1)),
                Err(e) => {
                    eprintln!("ssl_client: {}: {}", prog[0].to_string_lossy(), e);
                    return Ok(1);
                }
            }
        }
        match TcpStream::connect((host.as_str(), port)) {
            Ok(mut stream) => {
                let mut buf = [0u8; 4096];
                let stdin = io::stdin();
                let mut stdin_lock = stdin.lock();
                if let Ok(n) = stdin_lock.read(&mut buf) {
                    if n > 0 {
                        let _ = stream.write_all(&buf[..n]);
                    }
                }
                let stdout = io::stdout();
                let mut stdout_lock = stdout.lock();
                while let Ok(n) = stream.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    let _ = stdout_lock.write_all(&buf[..n]);
                }
                Ok(0)
            }
            Err(e) => {
                eprintln!("ssl_client: connect to {}:{}: {}", host, port, e);
                Ok(1)
            }
        }
    }
}
