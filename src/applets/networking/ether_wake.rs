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

pub struct EtherWakeApplet;
impl Applet for EtherWakeApplet {
    fn name(&self) -> &'static str {
        "ether-wake"
    }
    fn description(&self) -> &'static str {
        "Send a Wake-On-LAN magic packet"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut target_mac: Option<[u8; 6]> = None;
        let mut iface_name = "eth0".to_string();

        let mut i = 0;
        while i < args.len() {
            let s = args[i].to_string_lossy();
            if s == "-i" && i + 1 < args.len() {
                iface_name = args[i + 1].to_string_lossy().to_string();
                i += 1;
            } else if !s.starts_with('-') {
                target_mac = parse_mac(&s);
            }
            i += 1;
        }

        let mac = match target_mac {
            Some(m) => m,
            None => {
                eprintln!("Usage: ether-wake [-i iface] <MAC>");
                return Ok(1);
            }
        };

        let mut magic = [0u8; 102];
        magic[0..6].fill(0xff);
        for k in 0..16 {
            magic[6 + k * 6..6 + (k + 1) * 6].copy_from_slice(&mac);
        }

        let fd = unsafe {
            libc::socket(
                libc::AF_PACKET,
                libc::SOCK_RAW,
                (libc::ETH_P_IP as u16).to_be() as i32,
            )
        };
        if fd >= 0 {
            let mut ifr: libc::ifreq = unsafe { mem::zeroed() };
            set_ifr_name(&mut ifr, iface_name.as_bytes());
            let ifindex = unsafe { libc::if_nametoindex(ifr.ifr_name.as_ptr()) };

            let mut sll: libc::sockaddr_ll = unsafe { mem::zeroed() };
            sll.sll_family = libc::AF_PACKET as libc::c_ushort;
            sll.sll_ifindex = ifindex as libc::c_int;
            sll.sll_halen = 6;
            sll.sll_addr.fill(0xff);

            let mut frame = Vec::with_capacity(14 + magic.len());
            frame.extend_from_slice(&[0xff; 6]);
            frame.extend_from_slice(&[0; 6]);
            frame.extend_from_slice(&0x0842u16.to_be_bytes());
            frame.extend_from_slice(&magic);

            unsafe {
                libc::sendto(
                    fd,
                    frame.as_ptr() as *const libc::c_void,
                    frame.len(),
                    0,
                    &sll as *const _ as *const libc::sockaddr,
                    mem::size_of_val(&sll) as libc::socklen_t,
                );
                libc::close(fd);
            }
        } else {
            let ufd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
            if ufd >= 0 {
                let opt: libc::c_int = 1;
                unsafe {
                    libc::setsockopt(
                        ufd,
                        libc::SOL_SOCKET,
                        libc::SO_BROADCAST,
                        &opt as *const _ as *const libc::c_void,
                        mem::size_of_val(&opt) as libc::socklen_t,
                    );
                    let mut sin: libc::sockaddr_in = mem::zeroed();
                    sin.sin_family = libc::AF_INET as libc::sa_family_t;
                    sin.sin_port = 9u16.to_be();
                    sin.sin_addr.s_addr = 0xffffffff;
                    libc::sendto(
                        ufd,
                        magic.as_ptr() as *const libc::c_void,
                        magic.len(),
                        0,
                        &sin as *const _ as *const libc::sockaddr,
                        mem::size_of_val(&sin) as libc::socklen_t,
                    );
                    libc::close(ufd);
                }
            } else {
                eprintln!("ether-wake: socket failed: {}", io::Error::last_os_error());
                return Ok(1);
            }
        }

        Ok(0)
    }
}
