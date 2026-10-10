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

pub struct ArpApplet;
impl Applet for ArpApplet {
    fn name(&self) -> &'static str {
        "arp"
    }
    fn description(&self) -> &'static str {
        "Manipulate the system ARP cache"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut del_mode = false;
        let mut set_mode = false;
        let mut host_arg: Option<String> = None;
        let mut hw_arg: Option<String> = None;

        let mut i = 0;
        while i < args.len() {
            let s = args[i].to_string_lossy();
            if s == "-d" {
                del_mode = true;
                if i + 1 < args.len() {
                    host_arg = Some(args[i + 1].to_string_lossy().to_string());
                    i += 1;
                }
            } else if s == "-s" {
                set_mode = true;
                if i + 2 < args.len() {
                    host_arg = Some(args[i + 1].to_string_lossy().to_string());
                    hw_arg = Some(args[i + 2].to_string_lossy().to_string());
                    i += 2;
                }
            }
            i += 1;
        }

        if del_mode || set_mode {
            let host = match host_arg {
                Some(h) => h,
                None => {
                    eprintln!("arp: missing host");
                    return Ok(1);
                }
            };
            let ip = match host.parse::<Ipv4Addr>() {
                Ok(a) => a,
                Err(_) => {
                    eprintln!("arp: invalid IP: {}", host);
                    return Ok(1);
                }
            };

            let mut arpreq: libc::arpreq = unsafe { mem::zeroed() };
            let mut sin: libc::sockaddr_in = unsafe { mem::zeroed() };
            sin.sin_family = libc::AF_INET as libc::sa_family_t;
            sin.sin_addr.s_addr = u32::from_ne_bytes(ip.octets());
            arpreq.arp_pa = unsafe { *(&sin as *const _ as *const libc::sockaddr) };

            if set_mode {
                let hw = match hw_arg {
                    Some(ref h) => match parse_mac(h) {
                        Some(m) => m,
                        None => {
                            eprintln!("arp: invalid MAC: {}", h);
                            return Ok(1);
                        }
                    },
                    None => {
                        eprintln!("arp: missing MAC");
                        return Ok(1);
                    }
                };
                arpreq.arp_ha.sa_family = libc::ARPHRD_ETHER as libc::sa_family_t;
                for (j, &b) in hw.iter().enumerate() {
                    arpreq.arp_ha.sa_data[j] = b as libc::c_char;
                }
                arpreq.arp_flags = libc::ATF_PERM | libc::ATF_COM;
            }

            let fd = match open_socket_dgram() {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("arp: socket: {}", e);
                    return Ok(1);
                }
            };

            let req = if set_mode {
                libc::SIOCSARP as _
            } else {
                libc::SIOCDARP as _
            };
            let ret = unsafe { libc::ioctl(fd, req, &arpreq) };
            unsafe { libc::close(fd) };

            if ret < 0 {
                eprintln!("arp: ioctl: {}", io::Error::last_os_error());
                return Ok(1);
            }
            return Ok(0);
        }

        let file = match File::open("/proc/net/arp") {
            Ok(f) => f,
            Err(e) => {
                eprintln!("arp: /proc/net/arp: {}", e);
                return Ok(1);
            }
        };

        let stdout = io::stdout();
        let mut out = stdout.lock();
        let _ = writeln!(
            out,
            "Address                  HWtype  HWaddress           Flags Mask            Iface"
        );

        let reader = BufReader::new(file);
        for line in reader.lines().skip(1).map_while(|l| l.ok()) {
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() < 6 {
                continue;
            }
            let ip = cols[0];
            let hw_type = if cols[1] == "0x1" { "ether" } else { cols[1] };
            let flags = match cols[2] {
                "0x2" => "C",
                "0x6" => "CM",
                _ => cols[2],
            };
            let mac = cols[3];
            let mask = cols[4];
            let iface = cols[5];

            let _ = writeln!(
                out,
                "{:<24} {:<7} {:<19} {:<5} {:<15} {}",
                ip, hw_type, mac, flags, mask, iface
            );
        }

        Ok(0)
    }
}
