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

pub struct TftpdApplet;

impl Applet for TftpdApplet {
    fn name(&self) -> &'static str {
        "tftpd"
    }
    fn description(&self) -> &'static str {
        "TFTP server daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dir = PathBuf::from(".");
        let mut port = 69u16;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-p" && i + 1 < args.len() {
                i += 1;
                port = String::from_utf8_lossy(args[i].as_bytes())
                    .parse()
                    .unwrap_or(69);
            } else if !b.starts_with(b"-") {
                dir = PathBuf::from(&args[i]);
            }
            i += 1;
        }

        let sock = match UdpSocket::bind(("0.0.0.0", port)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("tftpd: bind: {}", e);
                return Ok(1);
            }
        };

        let mut buf = [0u8; 1024];
        while let Ok((len, client)) = sock.recv_from(&mut buf) {
            if len < 4 {
                continue;
            }
            let opcode = u16::from_be_bytes([buf[0], buf[1]]);
            if opcode == 1 {
                let slice = &buf[2..len];
                if let Some(pos) = slice.iter().position(|&b| b == 0) {
                    let filename = String::from_utf8_lossy(&slice[..pos]);
                    let filepath = dir.join(filename.trim_start_matches('/'));
                    if let Ok(mut f) = File::open(filepath) {
                        let mut block = 1u16;
                        let mut data_buf = [0u8; 516];
                        loop {
                            data_buf[0] = 0;
                            data_buf[1] = 3;
                            data_buf[2..4].copy_from_slice(&block.to_be_bytes());
                            let n = f.read(&mut data_buf[4..]).unwrap_or(0);
                            let _ = sock.send_to(&data_buf[..4 + n], client);

                            let mut ack = [0u8; 16];
                            let _ = sock.set_read_timeout(Some(Duration::from_secs(2)));
                            let _ = sock.recv_from(&mut ack);
                            if n < 512 {
                                break;
                            }
                            block = block.wrapping_add(1);
                        }
                    } else {
                        let err_pkt = [
                            0, 5, 0, 1, b'F', b'i', b'l', b'e', b' ', b'n', b'o', b't', b' ', b'f',
                            b'o', b'u', b'n', b'd', 0,
                        ];
                        let _ = sock.send_to(&err_pkt, client);
                    }
                }
            }
        }
        Ok(0)
    }
}
