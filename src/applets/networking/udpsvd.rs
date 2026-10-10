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

pub struct UdpsvdApplet;
impl Applet for UdpsvdApplet {
    fn name(&self) -> &'static str {
        "udpsvd"
    }
    fn description(&self) -> &'static str {
        "UDP service daemon"
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
            eprintln!("Usage: udpsvd [-v] <host> <port> <prog> [args...]");
            return Ok(1);
        }

        let host = &args[idx].to_string_lossy();
        let port = &args[idx + 1].to_string_lossy();
        let prog = &args[idx + 2];
        let prog_args = &args[idx + 3..];

        let addr = format!("{}:{}", host, port);
        let socket = match UdpSocket::bind(&addr) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("udpsvd: bind {}: {}", addr, e);
                return Ok(1);
            }
        };

        if verbose {
            eprintln!("udpsvd: listening on {}", addr);
        }

        let mut buf = [0u8; 65535];
        loop {
            match socket.recv_from(&mut buf) {
                Ok((len, peer)) => {
                    if verbose {
                        eprintln!("udpsvd: packet {} bytes from {}", len, peer);
                    }
                    let mut child = match Command::new(prog)
                        .args(prog_args)
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn()
                    {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("udpsvd: failed to execute prog: {}", e);
                            continue;
                        }
                    };

                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(&buf[..len]);
                    }

                    if let Ok(output) = child.wait_with_output() {
                        if !output.stdout.is_empty() {
                            let _ = socket.send_to(&output.stdout, peer);
                        }
                    }
                }
                Err(e) => {
                    if verbose {
                        eprintln!("udpsvd: recv: {}", e);
                    }
                }
            }
        }
    }
}
