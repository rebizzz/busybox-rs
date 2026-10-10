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

pub struct DhcprelayApplet;

impl Applet for DhcprelayApplet {
    fn name(&self) -> &'static str {
        "dhcprelay"
    }
    fn description(&self) -> &'static str {
        "DHCP relay agent"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut server_ip: Option<Ipv4Addr> = None;
        let mut client_port = 67u16;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if !b.starts_with(b"-") {
                if let Ok(ip) = String::from_utf8_lossy(b).parse::<Ipv4Addr>() {
                    server_ip = Some(ip);
                }
            } else if b == b"-p" && i + 1 < args.len() {
                i += 1;
                client_port = String::from_utf8_lossy(args[i].as_bytes())
                    .parse()
                    .unwrap_or(67);
            }
            i += 1;
        }

        let server_ip = match server_ip {
            Some(ip) => ip,
            None => {
                eprintln!("Usage: dhcprelay [options] server_ip");
                return Ok(1);
            }
        };

        let sock = match UdpSocket::bind(("0.0.0.0", client_port)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("dhcprelay: bind: {}", e);
                return Ok(1);
            }
        };
        let _ = sock.set_broadcast(true);

        let mut buf = [0u8; 1500];
        while let Ok((len, from)) = sock.recv_from(&mut buf) {
            if len < 240 {
                continue;
            }

            if from.ip() == IpAddr::V4(server_ip) {
                let _ = sock.send_to(&buf[..len], ("255.255.255.255", 68));
            } else {
                let _ = sock.send_to(&buf[..len], (server_ip, 67));
            }
        }
        Ok(0)
    }
}
