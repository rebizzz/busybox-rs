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

pub struct TcpsvdApplet;
impl Applet for TcpsvdApplet {
    fn name(&self) -> &'static str {
        "tcpsvd"
    }
    fn description(&self) -> &'static str {
        "TCP service daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut verbose = false;
        let mut idx = 0;

        while idx < args.len() {
            let s = args[idx].to_string_lossy();
            if s == "-v" {
                verbose = true;
                idx += 1;
            } else if s.starts_with('-') {
                idx += 1;
            } else {
                break;
            }
        }

        if args.len() < idx + 3 {
            eprintln!("Usage: tcpsvd [-v] <host> <port> <prog> [args...]");
            return Ok(1);
        }

        let host = &args[idx].to_string_lossy();
        let port = &args[idx + 1].to_string_lossy();
        let prog = &args[idx + 2];
        let prog_args = &args[idx + 3..];

        let addr = format!("{}:{}", host, port);
        let listener = match TcpListener::bind(&addr) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("tcpsvd: bind {}: {}", addr, e);
                return Ok(1);
            }
        };

        if verbose {
            eprintln!("tcpsvd: listening on {}", addr);
        }

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let fd = stream.as_raw_fd();
                    if verbose {
                        if let Ok(peer) = stream.peer_addr() {
                            eprintln!("tcpsvd: connection from {}", peer);
                        }
                    }

                    let stdin_fd = unsafe { libc::dup(fd) };
                    let stdout_fd = unsafe { libc::dup(fd) };

                    let mut cmd = Command::new(prog);
                    cmd.args(prog_args);
                    unsafe {
                        cmd.stdin(Stdio::from_raw_fd(stdin_fd));
                        cmd.stdout(Stdio::from_raw_fd(stdout_fd));
                    }
                    let _ = cmd.status();
                }
                Err(e) => {
                    if verbose {
                        eprintln!("tcpsvd: accept: {}", e);
                    }
                }
            }
        }

        Ok(0)
    }
}
