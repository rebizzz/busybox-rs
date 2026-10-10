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

pub struct IfconfigApplet;
impl Applet for IfconfigApplet {
    fn name(&self) -> &'static str {
        "ifconfig"
    }
    fn description(&self) -> &'static str {
        "Configure network interface parameters"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let fd = match open_socket_dgram() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("ifconfig: socket: {}", e);
                return Ok(1);
            }
        };

        let mut show_all = false;
        let mut target_iface: Option<String> = None;
        let mut i = 0;

        while i < args.len() {
            let s = args[i].to_string_lossy();
            if s == "-a" {
                show_all = true;
                i += 1;
            } else if !s.starts_with('-') {
                target_iface = Some(s.to_string());
                i += 1;
                break;
            } else {
                i += 1;
            }
        }

        if let Some(ref ifname) = target_iface {
            if i < args.len() {
                let mut ifr: libc::ifreq = unsafe { mem::zeroed() };
                set_ifr_name(&mut ifr, ifname.as_bytes());

                while i < args.len() {
                    let cmd = args[i].to_string_lossy();
                    i += 1;
                    match cmd.as_ref() {
                        "up" => unsafe {
                            if libc::ioctl(fd, libc::SIOCGIFFLAGS as _, &mut ifr) >= 0 {
                                ifr.ifr_ifru.ifru_flags |=
                                    (libc::IFF_UP | libc::IFF_RUNNING) as libc::c_short;
                                libc::ioctl(fd, libc::SIOCSIFFLAGS as _, &ifr);
                            }
                        },
                        "down" => unsafe {
                            if libc::ioctl(fd, libc::SIOCGIFFLAGS as _, &mut ifr) >= 0 {
                                ifr.ifr_ifru.ifru_flags &= !(libc::IFF_UP as libc::c_short);
                                libc::ioctl(fd, libc::SIOCSIFFLAGS as _, &ifr);
                            }
                        },
                        "netmask" => {
                            if i < args.len() {
                                if let Ok(ip) = args[i].to_string_lossy().parse::<Ipv4Addr>() {
                                    let mut sin: libc::sockaddr_in = unsafe { mem::zeroed() };
                                    sin.sin_family = libc::AF_INET as libc::sa_family_t;
                                    sin.sin_addr.s_addr = u32::from_ne_bytes(ip.octets());
                                    unsafe {
                                        let p = &sin as *const _ as *const libc::sockaddr;
                                        ifr.ifr_ifru.ifru_netmask = *p;
                                        libc::ioctl(fd, libc::SIOCSIFNETMASK as _, &ifr);
                                    }
                                }
                                i += 1;
                            }
                        }
                        "broadcast" => {
                            if i < args.len() {
                                if let Ok(ip) = args[i].to_string_lossy().parse::<Ipv4Addr>() {
                                    let mut sin: libc::sockaddr_in = unsafe { mem::zeroed() };
                                    sin.sin_family = libc::AF_INET as libc::sa_family_t;
                                    sin.sin_addr.s_addr = u32::from_ne_bytes(ip.octets());
                                    unsafe {
                                        let p = &sin as *const _ as *const libc::sockaddr;
                                        ifr.ifr_ifru.ifru_broadaddr = *p;
                                        libc::ioctl(fd, libc::SIOCSIFBRDADDR as _, &ifr);
                                    }
                                }
                                i += 1;
                            }
                        }
                        "mtu" => {
                            if i < args.len() {
                                if let Ok(mtu) = args[i].to_string_lossy().parse::<i32>() {
                                    ifr.ifr_ifru.ifru_mtu = mtu;
                                    unsafe {
                                        libc::ioctl(fd, libc::SIOCSIFMTU as _, &ifr);
                                    }
                                }
                                i += 1;
                            }
                        }
                        ip_str => {
                            if let Ok(ip) = ip_str.parse::<Ipv4Addr>() {
                                let mut sin: libc::sockaddr_in = unsafe { mem::zeroed() };
                                sin.sin_family = libc::AF_INET as libc::sa_family_t;
                                sin.sin_addr.s_addr = u32::from_ne_bytes(ip.octets());
                                unsafe {
                                    let p = &sin as *const _ as *const libc::sockaddr;
                                    ifr.ifr_ifru.ifru_addr = *p;
                                    libc::ioctl(fd, libc::SIOCSIFADDR as _, &ifr);
                                    if libc::ioctl(fd, libc::SIOCGIFFLAGS as _, &mut ifr) >= 0 {
                                        ifr.ifr_ifru.ifru_flags |=
                                            (libc::IFF_UP | libc::IFF_RUNNING) as libc::c_short;
                                        libc::ioctl(fd, libc::SIOCSIFFLAGS as _, &ifr);
                                    }
                                }
                            }
                        }
                    }
                }
                unsafe { libc::close(fd) };
                return Ok(0);
            }
        }

        let dev_data = fs::read_to_string("/proc/net/dev").unwrap_or_default();
        let stdout = io::stdout();
        let mut out = stdout.lock();

        for line in dev_data.lines().skip(2) {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() < 2 {
                continue;
            }
            let name = parts[0].trim();
            if let Some(ref target) = target_iface {
                if name != target {
                    continue;
                }
            }

            let mut ifr: libc::ifreq = unsafe { mem::zeroed() };
            set_ifr_name(&mut ifr, name.as_bytes());

            let flags = unsafe {
                if libc::ioctl(fd, libc::SIOCGIFFLAGS as _, &mut ifr) >= 0 {
                    ifr.ifr_ifru.ifru_flags as i32
                } else {
                    0
                }
            };

            let is_up = (flags & libc::IFF_UP) != 0;
            if !show_all && target_iface.is_none() && !is_up {
                continue;
            }

            let ip_str = unsafe {
                if libc::ioctl(fd, libc::SIOCGIFADDR as _, &mut ifr) >= 0 {
                    let sin = &ifr.ifr_ifru.ifru_addr as *const _ as *const libc::sockaddr_in;
                    let octets = (*sin).sin_addr.s_addr.to_ne_bytes();
                    Some(Ipv4Addr::from(octets).to_string())
                } else {
                    None
                }
            };

            let mask_str = unsafe {
                if libc::ioctl(fd, libc::SIOCGIFNETMASK as _, &mut ifr) >= 0 {
                    let sin = &ifr.ifr_ifru.ifru_netmask as *const _ as *const libc::sockaddr_in;
                    let octets = (*sin).sin_addr.s_addr.to_ne_bytes();
                    Some(Ipv4Addr::from(octets).to_string())
                } else {
                    None
                }
            };

            let mtu = unsafe {
                if libc::ioctl(fd, libc::SIOCGIFMTU as _, &mut ifr) >= 0 {
                    ifr.ifr_ifru.ifru_mtu
                } else {
                    1500
                }
            };

            let hw_str = unsafe {
                if libc::ioctl(fd, libc::SIOCGIFHWADDR as _, &mut ifr) >= 0 {
                    let sa_data = ifr.ifr_ifru.ifru_hwaddr.sa_data;
                    let bytes: Vec<u8> = sa_data.iter().take(6).map(|&b| b as u8).collect();
                    format_mac(&bytes)
                } else {
                    String::new()
                }
            };

            let stats: Vec<&str> = parts[1].split_whitespace().collect();
            let rx_bytes = stats.first().copied().unwrap_or("0");
            let rx_pkts = stats.get(1).copied().unwrap_or("0");
            let tx_bytes = stats.get(8).copied().unwrap_or("0");
            let tx_pkts = stats.get(9).copied().unwrap_or("0");

            let _ = writeln!(out, "{:<10} Link encap:Ethernet  HWaddr {}", name, hw_str);
            if let Some(ip) = ip_str {
                let _ = writeln!(
                    out,
                    "          inet addr:{}  Mask:{}",
                    ip,
                    mask_str.as_deref().unwrap_or("255.255.255.0")
                );
            }
            let _ = writeln!(out, "          UP BROADCAST RUNNING MULTICAST  MTU:{}", mtu);
            let _ = writeln!(out, "          RX packets:{} bytes:{}", rx_pkts, rx_bytes);
            let _ = writeln!(out, "          TX packets:{} bytes:{}\n", tx_pkts, tx_bytes);
        }

        unsafe { libc::close(fd) };
        Ok(0)
    }
}
