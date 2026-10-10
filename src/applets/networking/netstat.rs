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

pub struct NetstatApplet;
impl Applet for NetstatApplet {
    fn name(&self) -> &'static str {
        "netstat"
    }
    fn description(&self) -> &'static str {
        "Print network connections, routing tables, and interface statistics"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut show_tcp = false;
        let mut show_udp = false;
        let mut show_raw = false;
        let mut show_unix = false;
        let mut show_all = false;
        let mut show_listen = false;

        for arg in args {
            let b = arg.as_bytes();
            if b.starts_with(b"-") {
                for &c in &b[1..] {
                    match c {
                        b't' => show_tcp = true,
                        b'u' => show_udp = true,
                        b'w' => show_raw = true,
                        b'x' => show_unix = true,
                        b'a' => show_all = true,
                        b'l' => show_listen = true,
                        _ => {}
                    }
                }
            }
        }

        if !show_tcp && !show_udp && !show_raw && !show_unix {
            show_tcp = true;
            show_udp = true;
            show_raw = true;
            show_unix = true;
        }

        let stdout = io::stdout();
        let mut out = stdout.lock();

        let parse_sock_addr = |hex: &str| -> String {
            let parts: Vec<&str> = hex.split(':').collect();
            if parts.len() != 2 {
                return "*:*".to_string();
            }
            let ip_num = u32::from_str_radix(parts[0], 16).unwrap_or(0);
            let port = u16::from_str_radix(parts[1], 16).unwrap_or(0);
            let ip = Ipv4Addr::from(ip_num.to_ne_bytes());
            if ip_num == 0 && port == 0 {
                "0.0.0.0:*".to_string()
            } else if port == 0 {
                format!("{}:*", ip)
            } else {
                format!("{}:{}", ip, port)
            }
        };

        let tcp_state = |st: &str| -> &'static str {
            match st {
                "01" => "ESTABLISHED",
                "02" => "SYN_SENT",
                "03" => "SYN_RECV",
                "04" => "FIN_WAIT1",
                "05" => "FIN_WAIT2",
                "06" => "TIME_WAIT",
                "07" => "CLOSE",
                "08" => "CLOSE_WAIT",
                "09" => "LAST_ACK",
                "0A" => "LISTEN",
                "0B" => "CLOSING",
                _ => "UNKNOWN",
            }
        };

        if show_tcp || show_udp || show_raw {
            let _ = writeln!(
                out,
                "Active Internet connections (servers and established)\nProto Recv-Q Send-Q Local Address           Foreign Address         State"
            );

            if show_tcp {
                if let Ok(content) = fs::read_to_string("/proc/net/tcp") {
                    for line in content.lines().skip(1) {
                        let cols: Vec<&str> = line.split_whitespace().collect();
                        if cols.len() < 4 {
                            continue;
                        }
                        let state = tcp_state(cols[3]);
                        if show_listen && state != "LISTEN" {
                            continue;
                        }
                        if !show_all && !show_listen && state == "LISTEN" {
                            continue;
                        }
                        let local = parse_sock_addr(cols[1]);
                        let rem = parse_sock_addr(cols[2]);
                        let _ = writeln!(
                            out,
                            "tcp        0      0 {:<23} {:<23} {}",
                            local, rem, state
                        );
                    }
                }
            }

            if show_udp {
                if let Ok(content) = fs::read_to_string("/proc/net/udp") {
                    for line in content.lines().skip(1) {
                        let cols: Vec<&str> = line.split_whitespace().collect();
                        if cols.len() < 4 {
                            continue;
                        }
                        let local = parse_sock_addr(cols[1]);
                        let rem = parse_sock_addr(cols[2]);
                        let _ = writeln!(out, "udp        0      0 {:<23} {:<23}", local, rem);
                    }
                }
            }

            if show_raw {
                if let Ok(content) = fs::read_to_string("/proc/net/raw") {
                    for line in content.lines().skip(1) {
                        let cols: Vec<&str> = line.split_whitespace().collect();
                        if cols.len() < 4 {
                            continue;
                        }
                        let local = parse_sock_addr(cols[1]);
                        let rem = parse_sock_addr(cols[2]);
                        let _ = writeln!(
                            out,
                            "raw        0      0 {:<23} {:<23} {}",
                            local, rem, cols[3]
                        );
                    }
                }
            }
        }

        if show_unix {
            if let Ok(content) = fs::read_to_string("/proc/net/unix") {
                let _ = writeln!(
                    out,
                    "Active UNIX domain sockets (servers and established)\nProto RefCnt Flags       Type       State         I-Node Path"
                );
                for line in content.lines().skip(1) {
                    let cols: Vec<&str> = line.split_whitespace().collect();
                    if cols.len() < 6 {
                        continue;
                    }
                    let refcnt = cols[1];
                    let flags = cols[3];
                    let stype = match cols[4] {
                        "0001" => "STREAM",
                        "0002" => "DGRAM",
                        "0005" => "SEQPACKET",
                        _ => "UNKNOWN",
                    };
                    let state = match cols[5] {
                        "01" => "CONNECTED",
                        "02" => "DISCONNECTING",
                        "03" => "CONNECTED",
                        _ => "",
                    };
                    let inode = cols[6];
                    let path = if cols.len() > 7 { cols[7] } else { "" };
                    let _ = writeln!(
                        out,
                        "unix  {:<6} [ {:<5} ]   {:<10} {:<13} {:<6} {}",
                        refcnt, flags, stype, state, inode, path
                    );
                }
            }
        }

        Ok(0)
    }
}
