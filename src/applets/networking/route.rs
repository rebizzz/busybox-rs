use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::mem;
use std::net::Ipv4Addr;
use std::os::unix::ffi::OsStrExt;

pub struct RouteApplet;
impl Applet for RouteApplet {
    fn name(&self) -> &'static str {
        "route"
    }
    fn description(&self) -> &'static str {
        "Show or manipulate the IP routing table"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut add = false;
        let mut del = false;
        let mut idx = 0;

        while idx < args.len() {
            let s = args[idx].to_string_lossy();
            if s == "add" {
                add = true;
                idx += 1;
                break;
            } else if s == "del" {
                del = true;
                idx += 1;
                break;
            }
            idx += 1;
        }

        if add || del {
            let mut rt: libc::rtentry = unsafe { mem::zeroed() };
            let mut target_ip = Ipv4Addr::new(0, 0, 0, 0);
            let mut netmask_ip = Ipv4Addr::new(255, 255, 255, 255);
            let mut gw_ip = Ipv4Addr::new(0, 0, 0, 0);
            let mut dev_name: Option<CString> = None;
            let mut flags: libc::c_ushort = libc::RTF_UP as libc::c_ushort;

            while idx < args.len() {
                let arg = args[idx].to_string_lossy();
                idx += 1;
                match arg.as_ref() {
                    "default" => {
                        target_ip = Ipv4Addr::new(0, 0, 0, 0);
                        netmask_ip = Ipv4Addr::new(0, 0, 0, 0);
                    }
                    "-net" => {
                        if idx < args.len() {
                            if let Ok(ip) = args[idx].to_string_lossy().parse::<Ipv4Addr>() {
                                target_ip = ip;
                            }
                            idx += 1;
                        }
                    }
                    "-host" => {
                        if idx < args.len() {
                            if let Ok(ip) = args[idx].to_string_lossy().parse::<Ipv4Addr>() {
                                target_ip = ip;
                                flags |= libc::RTF_HOST as libc::c_ushort;
                            }
                            idx += 1;
                        }
                    }
                    "netmask" => {
                        if idx < args.len() {
                            if let Ok(ip) = args[idx].to_string_lossy().parse::<Ipv4Addr>() {
                                netmask_ip = ip;
                            }
                            idx += 1;
                        }
                    }
                    "gw" => {
                        if idx < args.len() {
                            if let Ok(ip) = args[idx].to_string_lossy().parse::<Ipv4Addr>() {
                                gw_ip = ip;
                                flags |= libc::RTF_GATEWAY as libc::c_ushort;
                            }
                            idx += 1;
                        }
                    }
                    "dev" => {
                        if idx < args.len() {
                            dev_name = CString::new(args[idx].as_bytes()).ok();
                            idx += 1;
                        }
                    }
                    ip_str => {
                        if let Ok(ip) = ip_str.parse::<Ipv4Addr>() {
                            target_ip = ip;
                        }
                    }
                }
            }

            let mut sin_dst: libc::sockaddr_in = unsafe { mem::zeroed() };
            sin_dst.sin_family = libc::AF_INET as libc::sa_family_t;
            sin_dst.sin_addr.s_addr = u32::from_ne_bytes(target_ip.octets());

            let mut sin_mask: libc::sockaddr_in = unsafe { mem::zeroed() };
            sin_mask.sin_family = libc::AF_INET as libc::sa_family_t;
            sin_mask.sin_addr.s_addr = u32::from_ne_bytes(netmask_ip.octets());

            let mut sin_gw: libc::sockaddr_in = unsafe { mem::zeroed() };
            sin_gw.sin_family = libc::AF_INET as libc::sa_family_t;
            sin_gw.sin_addr.s_addr = u32::from_ne_bytes(gw_ip.octets());

            unsafe {
                rt.rt_dst = *(&sin_dst as *const _ as *const libc::sockaddr);
                rt.rt_genmask = *(&sin_mask as *const _ as *const libc::sockaddr);
                rt.rt_gateway = *(&sin_gw as *const _ as *const libc::sockaddr);
                rt.rt_flags = flags;
                if let Some(ref d) = dev_name {
                    rt.rt_dev = d.as_ptr() as *mut libc::c_char;
                }
            }

            let fd = match open_socket_dgram() {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("route: {}", e);
                    return Ok(1);
                }
            };

            let req = if add {
                libc::SIOCADDRT as _
            } else {
                libc::SIOCDELRT as _
            };
            let ret = unsafe { libc::ioctl(fd, req, &rt) };
            unsafe { libc::close(fd) };

            if ret < 0 {
                eprintln!("route: ioctl error: {}", io::Error::last_os_error());
                return Ok(1);
            }
            return Ok(0);
        }

        let file = match File::open("/proc/net/route") {
            Ok(f) => f,
            Err(e) => {
                eprintln!("route: /proc/net/route: {}", e);
                return Ok(1);
            }
        };

        let stdout = io::stdout();
        let mut out = stdout.lock();
        let _ = writeln!(
            out,
            "Kernel IP routing table\nDestination     Gateway         Genmask         Flags Metric Ref    Use Iface"
        );

        let reader = BufReader::new(file);
        for line in reader.lines().skip(1).map_while(|l| l.ok()) {
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() < 11 {
                continue;
            }
            let iface = cols[0];
            let dest_hex = u32::from_str_radix(cols[1], 16).unwrap_or(0);
            let gw_hex = u32::from_str_radix(cols[2], 16).unwrap_or(0);
            let flags_num = u32::from_str_radix(cols[3], 16).unwrap_or(0);
            let metric = cols[6];
            let mask_hex = u32::from_str_radix(cols[7], 16).unwrap_or(0);

            let dest_ip = Ipv4Addr::from(dest_hex.to_ne_bytes());
            let gw_ip = Ipv4Addr::from(gw_hex.to_ne_bytes());
            let mask_ip = Ipv4Addr::from(mask_hex.to_ne_bytes());

            let mut flags = String::new();
            if flags_num & 1 != 0 {
                flags.push('U');
            }
            if flags_num & 2 != 0 {
                flags.push('G');
            }
            if flags_num & 4 != 0 {
                flags.push('H');
            }

            let dest_str = if dest_hex == 0 {
                "default".to_string()
            } else {
                dest_ip.to_string()
            };
            let gw_str = if gw_hex == 0 {
                "*".to_string()
            } else {
                gw_ip.to_string()
            };

            let _ = writeln!(
                out,
                "{:<15} {:<15} {:<15} {:<5} {:<6} 0      0 {}",
                dest_str, gw_str, mask_ip, flags, metric, iface
            );
        }

        Ok(0)
    }
}
