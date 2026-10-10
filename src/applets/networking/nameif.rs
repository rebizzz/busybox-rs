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

pub struct NameifApplet;
impl Applet for NameifApplet {
    fn name(&self) -> &'static str {
        "nameif"
    }
    fn description(&self) -> &'static str {
        "Name network interfaces based on MAC addresses"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut pairs: Vec<(String, [u8; 6])> = Vec::new();

        if args.len() >= 2 && !args[0].as_bytes().starts_with(b"-") {
            let name = args[0].to_string_lossy().to_string();
            if let Some(mac) = parse_mac(&args[1].to_string_lossy()) {
                pairs.push((name, mac));
            }
        } else {
            let conf_path = "/etc/mactab";
            if let Ok(content) = fs::read_to_string(conf_path) {
                for line in content.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 && !parts[0].starts_with('#') {
                        if let Some(mac) = parse_mac(parts[1]) {
                            pairs.push((parts[0].to_string(), mac));
                        }
                    }
                }
            }
        }

        if pairs.is_empty() {
            eprintln!("nameif: no interface definitions given");
            return Ok(1);
        }

        let fd = match open_socket_dgram() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("nameif: socket: {}", e);
                return Ok(1);
            }
        };

        let dev_data = fs::read_to_string("/proc/net/dev").unwrap_or_default();
        let mut current_ifaces = Vec::new();

        for line in dev_data.lines().skip(2) {
            if let Some(colon) = line.find(':') {
                let name = line[..colon].trim().to_string();
                let mut ifr: libc::ifreq = unsafe { mem::zeroed() };
                set_ifr_name(&mut ifr, name.as_bytes());
                unsafe {
                    if libc::ioctl(fd, libc::SIOCGIFHWADDR as _, &mut ifr) >= 0 {
                        let sa_data = ifr.ifr_ifru.ifru_hwaddr.sa_data;
                        let mac = [
                            sa_data[0] as u8,
                            sa_data[1] as u8,
                            sa_data[2] as u8,
                            sa_data[3] as u8,
                            sa_data[4] as u8,
                            sa_data[5] as u8,
                        ];
                        current_ifaces.push((name, mac));
                    }
                }
            }
        }

        let mut status = 0;
        for (new_name, target_mac) in pairs {
            if let Some((curr_name, _)) = current_ifaces.iter().find(|(_, m)| m == &target_mac) {
                if curr_name == &new_name {
                    continue;
                }
                let mut ifr: libc::ifreq = unsafe { mem::zeroed() };
                set_ifr_name(&mut ifr, curr_name.as_bytes());

                let new_len = new_name.len().min(libc::IFNAMSIZ - 1);
                unsafe {
                    let ptr = &mut ifr.ifr_ifru.ifru_newname as *mut _ as *mut u8;
                    for k in 0..libc::IFNAMSIZ {
                        *ptr.add(k) = 0;
                    }
                    std::ptr::copy_nonoverlapping(new_name.as_ptr(), ptr, new_len);
                    if libc::ioctl(fd, libc::SIOCSIFNAME as _, &ifr) < 0 {
                        eprintln!(
                            "nameif: cannot change name of {} to {}: {}",
                            curr_name,
                            new_name,
                            io::Error::last_os_error()
                        );
                        status = 1;
                    }
                }
            }
        }

        unsafe { libc::close(fd) };
        Ok(status)
    }
}
