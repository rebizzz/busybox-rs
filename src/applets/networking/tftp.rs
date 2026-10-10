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

pub struct TftpApplet;

impl Applet for TftpApplet {
    fn name(&self) -> &'static str {
        "tftp"
    }
    fn description(&self) -> &'static str {
        "Transfer file to/from TFTP server"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut is_get = true;
        let mut remote_file = None;
        let mut local_file = None;
        let mut host = None;
        let mut port = 69u16;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-g" {
                is_get = true;
            } else if b == b"-p" {
                is_get = false;
            } else if b == b"-r" && i + 1 < args.len() {
                i += 1;
                remote_file = Some(&args[i]);
            } else if b == b"-l" && i + 1 < args.len() {
                i += 1;
                local_file = Some(&args[i]);
            } else if !b.starts_with(b"-") && host.is_none() {
                host = Some(&args[i]);
            } else if !b.starts_with(b"-") {
                if let Ok(p) = String::from_utf8_lossy(b).parse::<u16>() {
                    port = p;
                }
            }
            i += 1;
        }

        let host = match host {
            Some(h) => String::from_utf8_lossy(h.as_bytes()).to_string(),
            None => {
                eprintln!("tftp: missing host");
                return Ok(1);
            }
        };

        let rem = match remote_file {
            Some(r) => String::from_utf8_lossy(r.as_bytes()).to_string(),
            None => match local_file {
                Some(l) => String::from_utf8_lossy(l.as_bytes()).to_string(),
                None => {
                    eprintln!("tftp: missing file");
                    return Ok(1);
                }
            },
        };

        let loc = local_file.unwrap_or(remote_file.unwrap());

        let sock = match UdpSocket::bind("0.0.0.0:0") {
            Ok(s) => s,
            Err(e) => {
                eprintln!("tftp: bind: {}", e);
                return Ok(1);
            }
        };
        let _ = sock.set_read_timeout(Some(Duration::from_secs(5)));

        let server_addr: SocketAddr = match format!("{}:{}", host, port).parse() {
            Ok(a) => a,
            Err(_) => {
                eprintln!("tftp: invalid server address");
                return Ok(1);
            }
        };

        if is_get {
            let mut req = vec![0, 1];
            req.extend_from_slice(rem.as_bytes());
            req.push(0);
            req.extend_from_slice(b"octet\0");
            if sock.send_to(&req, server_addr).is_err() {
                eprintln!("tftp: send error");
                return Ok(1);
            }

            let mut out = match File::create(loc) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("tftp: local file: {}", e);
                    return Ok(1);
                }
            };

            let mut expected_block = 1u16;
            let mut buf = [0u8; 1024];

            loop {
                match sock.recv_from(&mut buf) {
                    Ok((len, from)) => {
                        if len < 4 {
                            break;
                        }
                        let opcode = u16::from_be_bytes([buf[0], buf[1]]);
                        if opcode == 5 {
                            eprintln!("tftp: server returned error");
                            return Ok(1);
                        }
                        if opcode == 3 {
                            let block = u16::from_be_bytes([buf[2], buf[3]]);
                            if block == expected_block {
                                let _ = out.write_all(&buf[4..len]);

                                let ack = [0, 4, buf[2], buf[3]];
                                let _ = sock.send_to(&ack, from);
                                expected_block = expected_block.wrapping_add(1);
                                if len - 4 < 512 {
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("tftp: recv timeout: {}", e);
                        return Ok(1);
                    }
                }
            }
        } else {
            let mut req = vec![0, 2];
            req.extend_from_slice(rem.as_bytes());
            req.push(0);
            req.extend_from_slice(b"octet\0");
            if sock.send_to(&req, server_addr).is_err() {
                eprintln!("tftp: send error");
                return Ok(1);
            }

            let mut f = match File::open(loc) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("tftp: open local file: {}", e);
                    return Ok(1);
                }
            };

            let mut block = 0u16;
            let mut buf = [0u8; 516];

            let mut ack_buf = [0u8; 16];
            let remote = match sock.recv_from(&mut ack_buf) {
                Ok((len, from)) => {
                    if len < 4 || u16::from_be_bytes([ack_buf[0], ack_buf[1]]) != 4 {
                        eprintln!("tftp: WRQ rejected");
                        return Ok(1);
                    }
                    from
                }
                Err(e) => {
                    eprintln!("tftp: timeout waiting for WRQ ack: {}", e);
                    return Ok(1);
                }
            };

            loop {
                block = block.wrapping_add(1);
                buf[0] = 0;
                buf[1] = 3;
                buf[2..4].copy_from_slice(&block.to_be_bytes());
                let n = f.read(&mut buf[4..]).unwrap_or_default();
                let pkt_len = 4 + n;
                if sock.send_to(&buf[..pkt_len], remote).is_err() {
                    eprintln!("tftp: send data error");
                    return Ok(1);
                }

                match sock.recv_from(&mut ack_buf) {
                    Ok((len, _)) => {
                        if len >= 4 {
                            let ack_op = u16::from_be_bytes([ack_buf[0], ack_buf[1]]);
                            let ack_blk = u16::from_be_bytes([ack_buf[2], ack_buf[3]]);
                            if ack_op == 4 && ack_blk == block {
                                if n < 512 {
                                    break;
                                }
                                continue;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("tftp: ACK timeout: {}", e);
                        return Ok(1);
                    }
                }
            }
        }

        Ok(0)
    }
}
