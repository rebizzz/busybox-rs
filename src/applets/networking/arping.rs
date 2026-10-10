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

pub struct ArpingApplet;
impl Applet for ArpingApplet {
    fn name(&self) -> &'static str {
        "arping"
    }
    fn description(&self) -> &'static str {
        "Send ARP REQUEST to a neighbour host"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut count = 4;
        let mut iface_name = "eth0".to_string();
        let mut target_ip: Option<Ipv4Addr> = None;

        let mut i = 0;
        while i < args.len() {
            let s = args[i].to_string_lossy();
            if s == "-c" && i + 1 < args.len() {
                count = args[i + 1].to_string_lossy().parse().unwrap_or(4);
                i += 1;
            } else if s == "-I" && i + 1 < args.len() {
                iface_name = args[i + 1].to_string_lossy().to_string();
                i += 1;
            } else if !s.starts_with('-') {
                target_ip = s.parse().ok();
            }
            i += 1;
        }

        let dst_ip = match target_ip {
            Some(ip) => ip,
            None => {
                eprintln!("Usage: arping [-c count] [-I iface] <destination>");
                return Ok(1);
            }
        };

        let fd = unsafe {
            libc::socket(
                libc::AF_PACKET,
                libc::SOCK_RAW,
                (libc::ETH_P_ARP as u16).to_be() as i32,
            )
        };
        if fd < 0 {
            eprintln!("arping: socket(AF_PACKET): {}", io::Error::last_os_error());
            return Ok(1);
        }

        let mut ifr: libc::ifreq = unsafe { mem::zeroed() };
        set_ifr_name(&mut ifr, iface_name.as_bytes());

        let dgram_fd = open_socket_dgram().unwrap_or(fd);
        let ifindex = unsafe { libc::if_nametoindex(ifr.ifr_name.as_ptr()) };
        let mut src_mac = [0u8; 6];
        let mut src_ip = Ipv4Addr::new(0, 0, 0, 0);

        unsafe {
            if libc::ioctl(dgram_fd, libc::SIOCGIFHWADDR as _, &mut ifr) >= 0 {
                for (j, slot) in src_mac.iter_mut().enumerate() {
                    *slot = ifr.ifr_ifru.ifru_hwaddr.sa_data[j] as u8;
                }
            }
            if libc::ioctl(dgram_fd, libc::SIOCGIFADDR as _, &mut ifr) >= 0 {
                let sin = &ifr.ifr_ifru.ifru_addr as *const _ as *const libc::sockaddr_in;
                src_ip = Ipv4Addr::from((*sin).sin_addr.s_addr.to_ne_bytes());
            }
            if dgram_fd != fd {
                libc::close(dgram_fd);
            }
        }

        let mut sll: libc::sockaddr_ll = unsafe { mem::zeroed() };
        sll.sll_family = libc::AF_PACKET as libc::c_ushort;
        sll.sll_ifindex = ifindex as libc::c_int;
        sll.sll_protocol = (libc::ETH_P_ARP as u16).to_be();
        sll.sll_halen = 6;
        sll.sll_addr.fill(0xff);

        let mut frame = [0u8; 42];
        frame[0..6].fill(0xff);
        frame[6..12].copy_from_slice(&src_mac);
        frame[12..14].copy_from_slice(&(libc::ETH_P_ARP as u16).to_be_bytes());

        frame[14..16].copy_from_slice(&1u16.to_be_bytes());
        frame[16..18].copy_from_slice(&(libc::ETH_P_IP as u16).to_be_bytes());
        frame[18] = 6;
        frame[19] = 4;
        frame[20..22].copy_from_slice(&1u16.to_be_bytes());
        frame[22..28].copy_from_slice(&src_mac);
        frame[28..32].copy_from_slice(&src_ip.octets());
        frame[32..38].fill(0);
        frame[38..42].copy_from_slice(&dst_ip.octets());

        let stdout = io::stdout();
        let mut out = stdout.lock();
        let _ = writeln!(
            out,
            "ARPING to {} from {} via {}",
            dst_ip, src_ip, iface_name
        );

        let mut sent = 0;
        let mut received = 0;

        for _ in 0..count {
            let ret = unsafe {
                libc::sendto(
                    fd,
                    frame.as_ptr() as *const libc::c_void,
                    frame.len(),
                    0,
                    &sll as *const _ as *const libc::sockaddr,
                    mem::size_of_val(&sll) as libc::socklen_t,
                )
            };
            if ret > 0 {
                sent += 1;
            }

            let tv = libc::timeval {
                tv_sec: 1,
                tv_usec: 0,
            };
            unsafe {
                libc::setsockopt(
                    fd,
                    libc::SOL_SOCKET,
                    libc::SO_RCVTIMEO,
                    &tv as *const _ as *const libc::c_void,
                    mem::size_of_val(&tv) as libc::socklen_t,
                );
            }

            let mut rx_buf = [0u8; 1500];
            let n = unsafe {
                libc::recv(
                    fd,
                    rx_buf.as_mut_ptr() as *mut libc::c_void,
                    rx_buf.len(),
                    0,
                )
            };
            if n >= 42 {
                let op = u16::from_be_bytes([rx_buf[20], rx_buf[21]]);
                if op == 2 && rx_buf[28..32] == dst_ip.octets() {
                    received += 1;
                    let rep_mac = format_mac(&rx_buf[22..28]);
                    let _ = writeln!(out, "Unicast reply from {} [{}]", dst_ip, rep_mac);
                }
            }
            thread::sleep(Duration::from_millis(500));
        }

        unsafe { libc::close(fd) };
        let _ = writeln!(
            out,
            "Sent {} probe(s) ({} broadcast(s))\nReceived {} response(s)",
            sent, sent, received
        );
        Ok(if received > 0 { 0 } else { 1 })
    }
}
