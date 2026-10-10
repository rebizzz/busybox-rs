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

pub struct NcApplet;
impl Applet for NcApplet {
    fn name(&self) -> &'static str {
        "nc"
    }
    fn description(&self) -> &'static str {
        "Arbitrary TCP and UDP connections and listens"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut listen = false;
        let mut udp = false;
        let mut port_opt: Option<u16> = None;
        let mut host_arg: Option<String> = None;
        let mut port_arg: Option<u16> = None;

        let mut i = 0;
        while i < args.len() {
            let s = args[i].to_string_lossy();
            if s == "-l" {
                listen = true;
            } else if s == "-u" {
                udp = true;
            } else if s == "-p" && i + 1 < args.len() {
                port_opt = args[i + 1].to_string_lossy().parse().ok();
                i += 1;
            } else if !s.starts_with('-') {
                if host_arg.is_none() {
                    host_arg = Some(s.to_string());
                } else if port_arg.is_none() {
                    port_arg = s.parse().ok();
                }
            }
            i += 1;
        }

        if listen {
            let port = port_opt.or(port_arg).unwrap_or(0);
            let host = host_arg.unwrap_or_else(|| "0.0.0.0".to_string());
            let bind_addr = format!("{}:{}", host, port);

            if udp {
                let socket = match UdpSocket::bind(&bind_addr) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("nc: bind {}: {}", bind_addr, e);
                        return Ok(1);
                    }
                };

                let mut buf = [0u8; 8192];
                let (len, peer) = match socket.recv_from(&mut buf) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("nc: recv: {}", e);
                        return Ok(1);
                    }
                };
                let stdout = io::stdout();
                let mut out = stdout.lock();
                let _ = out.write_all(&buf[..len]);
                let _ = out.flush();

                let sock_send = socket.try_clone()?;
                thread::spawn(move || {
                    let mut stdin = io::stdin().lock();
                    let mut sbuf = [0u8; 8192];
                    while let Ok(n) = stdin.read(&mut sbuf) {
                        if n == 0 {
                            break;
                        }
                        if sock_send.send_to(&sbuf[..n], peer).is_err() {
                            break;
                        }
                    }
                });

                while let Ok((n, _)) = socket.recv_from(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    if out.write_all(&buf[..n]).is_err() {
                        break;
                    }
                    let _ = out.flush();
                }
                return Ok(0);
            }

            let listener = match TcpListener::bind(&bind_addr) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("nc: bind {}: {}", bind_addr, e);
                    return Ok(1);
                }
            };

            let (stream, _) = match listener.accept() {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("nc: accept: {}", e);
                    return Ok(1);
                }
            };

            forward_stream(stream)?;
            return Ok(0);
        }

        let host = match host_arg {
            Some(h) => h,
            None => {
                eprintln!("nc: missing host");
                return Ok(1);
            }
        };
        let port = match port_arg.or(port_opt) {
            Some(p) => p,
            None => {
                eprintln!("nc: missing port");
                return Ok(1);
            }
        };

        if udp {
            let socket = match UdpSocket::bind("0.0.0.0:0") {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("nc: bind: {}", e);
                    return Ok(1);
                }
            };
            let remote = format!("{}:{}", host, port);
            if let Err(e) = socket.connect(&remote) {
                eprintln!("nc: connect {}: {}", remote, e);
                return Ok(1);
            }

            let sock_send = socket.try_clone()?;
            thread::spawn(move || {
                let mut stdin = io::stdin().lock();
                let mut buf = [0u8; 8192];
                while let Ok(n) = stdin.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    if sock_send.send(&buf[..n]).is_err() {
                        break;
                    }
                }
            });

            let stdout = io::stdout();
            let mut out = stdout.lock();
            let mut buf = [0u8; 8192];
            while let Ok(n) = socket.recv(&mut buf) {
                if n == 0 {
                    break;
                }
                if out.write_all(&buf[..n]).is_err() {
                    break;
                }
                let _ = out.flush();
            }
            return Ok(0);
        }

        let stream = match TcpStream::connect((host.as_str(), port)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("nc: connect {}:{}: {}", host, port, e);
                return Ok(1);
            }
        };

        forward_stream(stream)?;
        Ok(0)
    }
}

fn forward_stream(stream: TcpStream) -> Result<()> {
    let mut reader = stream.try_clone()?;
    let mut writer = stream;

    let t = thread::spawn(move || {
        let mut stdin = io::stdin().lock();
        let mut buf = [0u8; 8192];
        while let Ok(n) = stdin.read(&mut buf) {
            if n == 0 {
                break;
            }
            if writer.write_all(&buf[..n]).is_err() {
                break;
            }
            let _ = writer.flush();
        }
    });

    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut buf = [0u8; 8192];
    while let Ok(n) = reader.read(&mut buf) {
        if n == 0 {
            break;
        }
        if out.write_all(&buf[..n]).is_err() {
            break;
        }
        let _ = out.flush();
    }

    let _ = t.join();
    Ok(())
}

pub fn checksum(data: &[u8]) -> u16 {
    let mut sum = 0u32;
    let mut i = 0;
    while i + 1 < data.len() {
        let word = u16::from_be_bytes([data[i], data[i + 1]]);
        sum = sum.wrapping_add(word as u32);
        i += 2;
    }
    if i < data.len() {
        let word = (data[i] as u32) << 8;
        sum = sum.wrapping_add(word);
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !sum as u16
}
