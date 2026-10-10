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

pub struct TelnetdApplet;

impl Applet for TelnetdApplet {
    fn name(&self) -> &'static str {
        "telnetd"
    }
    fn description(&self) -> &'static str {
        "TELNET server daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut port = 23u16;
        let mut login_prog = "/bin/sh".to_string();
        let mut inetd_mode = false;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-p" && i + 1 < args.len() {
                i += 1;
                port = String::from_utf8_lossy(args[i].as_bytes())
                    .parse()
                    .unwrap_or(23);
            } else if b == b"-l" && i + 1 < args.len() {
                i += 1;
                login_prog = String::from_utf8_lossy(args[i].as_bytes()).to_string();
            } else if b == b"-i" {
                inetd_mode = true;
            }
            i += 1;
        }

        fn run_session(stream: TcpStream, prog: &str) {
            let prog_str = prog.to_string();
            std::thread::spawn(move || {
                let mut child = match Command::new(&prog_str)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                {
                    Ok(c) => c,
                    Err(_) => return,
                };

                let mut child_in = child.stdin.take().unwrap();
                let mut child_out = child.stdout.take().unwrap();
                let mut s_read = match stream.try_clone() {
                    Ok(s) => s,
                    Err(_) => return,
                };
                let mut s_write = stream;

                std::thread::spawn(move || {
                    let mut b = [0u8; 1024];
                    while let Ok(n) = child_out.read(&mut b) {
                        if n == 0 || s_write.write_all(&b[..n]).is_err() {
                            break;
                        }
                    }
                });

                let mut b = [0u8; 1024];
                while let Ok(n) = s_read.read(&mut b) {
                    if n == 0 || child_in.write_all(&b[..n]).is_err() {
                        break;
                    }
                }
                let _ = child.wait();
            });
        }

        if inetd_mode {
            let _ = Command::new(&login_prog).spawn().and_then(|mut c| c.wait());
            return Ok(0);
        }

        let listener = match TcpListener::bind(("0.0.0.0", port)) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("telnetd: bind: {}", e);
                return Ok(1);
            }
        };

        for stream in listener.incoming().flatten() {
            run_session(stream, &login_prog);
        }

        Ok(0)
    }
}
