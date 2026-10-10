use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::mem;
use std::net::Ipv4Addr;
use std::os::unix::ffi::OsStrExt;
use std::process::Command;
use std::thread;
use std::time::Duration;

const SIOCBRADDBR: libc::c_ulong = 0x89a0;
const SIOCBRDELBR: libc::c_ulong = 0x89a1;
const SIOCBRADDIF: libc::c_ulong = 0x89a2;
const SIOCBRDELIF: libc::c_ulong = 0x89a3;
const SIOCSIFVLAN: libc::c_ulong = 0x8983;
const SIOCBONDENSLAVE: libc::c_ulong = 0x8990;
const SIOCBONDRELEASE: libc::c_ulong = 0x8991;
const TUNSETIFF: libc::c_ulong = 0x400454ca;
const TUNSETPERSIST: libc::c_ulong = 0x400454cb;
const TUNSETOWNER: libc::c_ulong = 0x400454cc;
const IFF_TAP: libc::c_short = 0x0002;
const IFF_NO_PI: libc::c_short = 0x1000;
const SIOCADDTUNNEL: libc::c_ulong = 0x89f0 + 1;
const SIOCDELTUNNEL: libc::c_ulong = 0x89f0 + 2;

fn set_ifr_name(ifr: &mut libc::ifreq, name: &[u8]) {
    for b in ifr.ifr_name.iter_mut() {
        *b = 0;
    }
    let len = name.len().min(libc::IFNAMSIZ - 1);
    for (i, &b) in name[..len].iter().enumerate() {
        ifr.ifr_name[i] = b as libc::c_char;
    }
}

fn open_socket_dgram() -> io::Result<libc::c_int> {
    let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
    if fd < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(fd)
    }
}

fn format_mac(hw: &[u8]) -> String {
    if hw.len() >= 6 {
        format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            hw[0], hw[1], hw[2], hw[3], hw[4], hw[5]
        )
    } else {
        String::new()
    }
}

fn parse_mac(s: &str) -> Option<[u8; 6]> {
    let parts: Vec<&str> = if s.contains(':') {
        s.split(':').collect()
    } else if s.contains('-') {
        s.split('-').collect()
    } else {
        return None;
    };
    if parts.len() != 6 {
        return None;
    }
    let mut mac = [0u8; 6];
    for (i, p) in parts.iter().enumerate() {
        mac[i] = u8::from_str_radix(p, 16).ok()?;
    }
    Some(mac)
}

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

fn run_ip_link(args: &[OsString]) -> Result<i32> {
    if args.is_empty() || args[0].to_string_lossy() == "show" || args[0].to_string_lossy() == "list"
    {
        return IfconfigApplet.run(&[OsString::from("-a")]);
    }

    if args[0].to_string_lossy() == "set" && args.len() >= 3 {
        let ifname = &args[1];
        let action = &args[2];
        return IfconfigApplet.run(&[ifname.clone(), action.clone()]);
    }

    IfconfigApplet.run(&[OsString::from("-a")])
}

fn run_ip_addr(args: &[OsString]) -> Result<i32> {
    if args.is_empty() || args[0].to_string_lossy() == "show" || args[0].to_string_lossy() == "list"
    {
        return IfconfigApplet.run(&[OsString::from("-a")]);
    }

    if (args[0].to_string_lossy() == "add" || args[0].to_string_lossy() == "del") && args.len() >= 4
    {
        let ip_mask = args[1].to_string_lossy();
        let mut dev = OsString::new();
        let mut i = 2;
        while i < args.len() {
            if args[i].to_string_lossy() == "dev" && i + 1 < args.len() {
                dev = args[i + 1].clone();
                break;
            }
            i += 1;
        }

        let ip_only = ip_mask.split('/').next().unwrap_or(&ip_mask);
        return IfconfigApplet.run(&[dev, OsString::from(ip_only)]);
    }

    IfconfigApplet.run(&[OsString::from("-a")])
}

fn run_ip_route(args: &[OsString]) -> Result<i32> {
    if args.is_empty() || args[0].to_string_lossy() == "show" || args[0].to_string_lossy() == "list"
    {
        return RouteApplet.run(&[]);
    }
    RouteApplet.run(args)
}

fn run_ip_neigh(args: &[OsString]) -> Result<i32> {
    if args.is_empty() || args[0].to_string_lossy() == "show" || args[0].to_string_lossy() == "list"
    {
        return ArpApplet.run(&[]);
    }
    ArpApplet.run(args)
}

fn run_ip_rule(_args: &[OsString]) -> Result<i32> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let _ = writeln!(
        out,
        "0:\tfrom all lookup local\n32766:\tfrom all lookup main\n32767:\tfrom all lookup default"
    );
    Ok(0)
}

fn run_ip_tunnel(args: &[OsString]) -> Result<i32> {
    if args.is_empty() || args[0].to_string_lossy() == "show" || args[0].to_string_lossy() == "list"
    {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        if let Ok(devs) = fs::read_to_string("/proc/net/dev") {
            for line in devs.lines().skip(2) {
                if let Some(colon) = line.find(':') {
                    let name = line[..colon].trim();
                    if name.starts_with("gre")
                        || name.starts_with("sit")
                        || name.starts_with("ipip")
                        || name.starts_with("tun")
                    {
                        let _ = writeln!(out, "{}: ip/ip remote any local any", name);
                    }
                }
            }
        }
        return Ok(0);
    }

    if (args[0].to_string_lossy() == "add" || args[0].to_string_lossy() == "del") && args.len() >= 2
    {
        let is_add = args[0].to_string_lossy() == "add";
        let tun_name = args[1].to_string_lossy();
        let mut ifr: libc::ifreq = unsafe { mem::zeroed() };
        set_ifr_name(&mut ifr, tun_name.as_bytes());

        let fd = open_socket_dgram()?;
        let req = if is_add { SIOCADDTUNNEL } else { SIOCDELTUNNEL };
        let ret = unsafe { libc::ioctl(fd, req as _, &ifr) };
        unsafe { libc::close(fd) };
        if ret < 0 {
            eprintln!("iptunnel: ioctl: {}", io::Error::last_os_error());
            return Ok(1);
        }
        return Ok(0);
    }

    Ok(0)
}

pub struct IpApplet;
impl Applet for IpApplet {
    fn name(&self) -> &'static str {
        "ip"
    }
    fn description(&self) -> &'static str {
        "Show / manipulate routing, network devices, interfaces and tunnels"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("Usage: ip [link|addr|route|neigh|rule|tunnel] ...");
            return Ok(1);
        }
        let sub = args[0].to_string_lossy();
        match sub.as_ref() {
            "link" | "l" => run_ip_link(&args[1..]),
            "addr" | "a" | "address" => run_ip_addr(&args[1..]),
            "route" | "r" | "ro" => run_ip_route(&args[1..]),
            "neigh" | "n" | "neighbor" => run_ip_neigh(&args[1..]),
            "rule" | "ru" => run_ip_rule(&args[1..]),
            "tunnel" | "tu" => run_ip_tunnel(&args[1..]),
            _ => {
                eprintln!("ip: unknown object '{}'", sub);
                Ok(1)
            }
        }
    }
}

pub struct IpaddrApplet;
impl Applet for IpaddrApplet {
    fn name(&self) -> &'static str {
        "ipaddr"
    }
    fn description(&self) -> &'static str {
        "Manage IP addresses"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_ip_addr(args)
    }
}

pub struct IplinkApplet;
impl Applet for IplinkApplet {
    fn name(&self) -> &'static str {
        "iplink"
    }
    fn description(&self) -> &'static str {
        "Manage network devices"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_ip_link(args)
    }
}

pub struct IpneighApplet;
impl Applet for IpneighApplet {
    fn name(&self) -> &'static str {
        "ipneigh"
    }
    fn description(&self) -> &'static str {
        "Manage neighbour / ARP table"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_ip_neigh(args)
    }
}

pub struct IprouteApplet;
impl Applet for IprouteApplet {
    fn name(&self) -> &'static str {
        "iproute"
    }
    fn description(&self) -> &'static str {
        "Manage routing table"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_ip_route(args)
    }
}

pub struct IpruleApplet;
impl Applet for IpruleApplet {
    fn name(&self) -> &'static str {
        "iprule"
    }
    fn description(&self) -> &'static str {
        "Manage routing policy database"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_ip_rule(args)
    }
}

pub struct IptunnelApplet;
impl Applet for IptunnelApplet {
    fn name(&self) -> &'static str {
        "iptunnel"
    }
    fn description(&self) -> &'static str {
        "Manage IP tunnels"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_ip_tunnel(args)
    }
}

pub struct IfupApplet;
impl Applet for IfupApplet {
    fn name(&self) -> &'static str {
        "ifup"
    }
    fn description(&self) -> &'static str {
        "Bring a network interface up"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("Usage: ifup <interface>");
            return Ok(1);
        }
        let ifname = &args[0];
        IfconfigApplet.run(&[ifname.clone(), OsString::from("up")])
    }
}

pub struct IfdownApplet;
impl Applet for IfdownApplet {
    fn name(&self) -> &'static str {
        "ifdown"
    }
    fn description(&self) -> &'static str {
        "Take a network interface down"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("Usage: ifdown <interface>");
            return Ok(1);
        }
        let ifname = &args[0];
        IfconfigApplet.run(&[ifname.clone(), OsString::from("down")])
    }
}

pub struct IfenslaveApplet;
impl Applet for IfenslaveApplet {
    fn name(&self) -> &'static str {
        "ifenslave"
    }
    fn description(&self) -> &'static str {
        "Attach network interfaces to a bonding device"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut detach = false;
        let mut master: Option<String> = None;
        let mut slaves = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let s = args[i].to_string_lossy();
            if s == "-d" {
                detach = true;
            } else if master.is_none() {
                master = Some(s.to_string());
            } else {
                slaves.push(s.to_string());
            }
            i += 1;
        }

        let m = match master {
            Some(m) => m,
            None => {
                eprintln!("Usage: ifenslave [-d] <master> <slave>...");
                return Ok(1);
            }
        };

        let fd = open_socket_dgram()?;
        let req = if detach {
            SIOCBONDRELEASE
        } else {
            SIOCBONDENSLAVE
        };

        let mut status = 0;
        for slave in slaves {
            let mut ifr: libc::ifreq = unsafe { mem::zeroed() };
            set_ifr_name(&mut ifr, m.as_bytes());

            let mut slave_ifr: libc::ifreq = unsafe { mem::zeroed() };
            set_ifr_name(&mut slave_ifr, slave.as_bytes());

            unsafe {
                let dst_ptr = &mut ifr.ifr_ifru.ifru_slave as *mut _ as *mut u8;
                let src_ptr = slave_ifr.ifr_name.as_ptr() as *const u8;
                std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, libc::IFNAMSIZ);
                if libc::ioctl(fd, req as _, &ifr) < 0 {
                    eprintln!(
                        "ifenslave: ioctl failed on master {} slave {}: {}",
                        m,
                        slave,
                        io::Error::last_os_error()
                    );
                    status = 1;
                }
            }
        }

        unsafe { libc::close(fd) };
        Ok(status)
    }
}

pub struct IfplugdApplet;
impl Applet for IfplugdApplet {
    fn name(&self) -> &'static str {
        "ifplugd"
    }
    fn description(&self) -> &'static str {
        "Link detection daemon for network interfaces"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut iface = "eth0".to_string();
        let mut run_cmd: Option<String> = None;

        let mut i = 0;
        while i < args.len() {
            let s = args[i].to_string_lossy();
            if s == "-i" && i + 1 < args.len() {
                iface = args[i + 1].to_string_lossy().to_string();
                i += 1;
            } else if s == "-r" && i + 1 < args.len() {
                run_cmd = Some(args[i + 1].to_string_lossy().to_string());
                i += 1;
            }
            i += 1;
        }

        let carrier_path = format!("/sys/class/net/{}/carrier", iface);
        let operstate_path = format!("/sys/class/net/{}/operstate", iface);

        let carrier = fs::read_to_string(&carrier_path)
            .map(|s| s.trim() == "1")
            .unwrap_or_else(|_| {
                fs::read_to_string(&operstate_path)
                    .map(|s| s.trim() == "up")
                    .unwrap_or(false)
            });

        let stdout = io::stdout();
        let mut out = stdout.lock();
        let _ = writeln!(
            out,
            "ifplugd: interface {} link is {}",
            iface,
            if carrier { "up" } else { "down" }
        );

        if let Some(cmd) = run_cmd {
            let arg = if carrier { "up" } else { "down" };
            let _ = Command::new(&cmd).arg(&iface).arg(arg).status();
        }

        Ok(0)
    }
}

#[repr(C)]
struct VlanIoctlArgs {
    cmd: libc::c_int,
    device1: [libc::c_char; 24],
    u: VlanUnion,
}

#[repr(C)]
union VlanUnion {
    device2: [libc::c_char; 24],
    vlan_qos: libc::c_int,
    vlan_id: libc::c_uint,
    flag: libc::c_uint,
}

pub struct VconfigApplet;
impl Applet for VconfigApplet {
    fn name(&self) -> &'static str {
        "vconfig"
    }
    fn description(&self) -> &'static str {
        "VLAN configuration utility"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("Usage: vconfig add <iface> <vlan_id> | rem <vlan_iface>");
            return Ok(1);
        }

        let cmd = args[0].to_string_lossy();
        let mut vargs: VlanIoctlArgs = unsafe { mem::zeroed() };

        match cmd.as_ref() {
            "add" => {
                if args.len() < 3 {
                    eprintln!("vconfig: add requires interface and vlan_id");
                    return Ok(1);
                }
                let iface = args[1].as_bytes();
                let vid: u32 = args[2].to_string_lossy().parse().unwrap_or(0);
                vargs.cmd = 0;
                let len = iface.len().min(23);
                for (k, &b) in iface[..len].iter().enumerate() {
                    vargs.device1[k] = b as libc::c_char;
                }
                vargs.u.vlan_id = vid;
            }
            "rem" => {
                let iface = args[1].as_bytes();
                vargs.cmd = 1;
                let len = iface.len().min(23);
                for (k, &b) in iface[..len].iter().enumerate() {
                    vargs.device1[k] = b as libc::c_char;
                }
            }
            _ => {
                eprintln!("vconfig: unknown command '{}'", cmd);
                return Ok(1);
            }
        }

        let fd = open_socket_dgram()?;
        let mut ifr: libc::ifreq = unsafe { mem::zeroed() };
        ifr.ifr_ifru.ifru_data = &mut vargs as *mut _ as *mut libc::c_char;

        let ret = unsafe { libc::ioctl(fd, SIOCSIFVLAN as _, &ifr) };
        unsafe { libc::close(fd) };

        if ret < 0 {
            eprintln!("vconfig: ioctl error: {}", io::Error::last_os_error());
            return Ok(1);
        }
        Ok(0)
    }
}

pub struct TunctlApplet;
impl Applet for TunctlApplet {
    fn name(&self) -> &'static str {
        "tunctl"
    }
    fn description(&self) -> &'static str {
        "Create and manage persistent TUN/TAP interfaces"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut delete = false;
        let mut tun_name = "tap0".to_string();
        let mut user_uid: Option<u32> = None;

        let mut i = 0;
        while i < args.len() {
            let s = args[i].to_string_lossy();
            if s == "-d" && i + 1 < args.len() {
                delete = true;
                tun_name = args[i + 1].to_string_lossy().to_string();
                i += 1;
            } else if s == "-t" && i + 1 < args.len() {
                tun_name = args[i + 1].to_string_lossy().to_string();
                i += 1;
            } else if s == "-u" && i + 1 < args.len() {
                user_uid = args[i + 1].to_string_lossy().parse().ok();
                i += 1;
            }
            i += 1;
        }

        let tun_path = CString::new("/dev/net/tun").unwrap();
        let fd = unsafe { libc::open(tun_path.as_ptr(), libc::O_RDWR) };
        if fd < 0 {
            eprintln!(
                "tunctl: failed to open /dev/net/tun: {}",
                io::Error::last_os_error()
            );
            return Ok(1);
        }

        let mut ifr: libc::ifreq = unsafe { mem::zeroed() };
        set_ifr_name(&mut ifr, tun_name.as_bytes());
        ifr.ifr_ifru.ifru_flags = (IFF_TAP | IFF_NO_PI) as libc::c_short;

        let ret = unsafe { libc::ioctl(fd, TUNSETIFF as _, &ifr) };
        if ret < 0 {
            eprintln!("tunctl: TUNSETIFF: {}", io::Error::last_os_error());
            unsafe { libc::close(fd) };
            return Ok(1);
        }

        if delete {
            let ret = unsafe { libc::ioctl(fd, TUNSETPERSIST as _, 0) };
            unsafe { libc::close(fd) };
            if ret < 0 {
                eprintln!(
                    "tunctl: failed to delete {}: {}",
                    tun_name,
                    io::Error::last_os_error()
                );
                return Ok(1);
            }
            let stdout = io::stdout();
            let mut out = stdout.lock();
            let _ = writeln!(out, "Set '{}' non-persistent", tun_name);
            return Ok(0);
        }

        if let Some(uid) = user_uid {
            unsafe {
                libc::ioctl(fd, TUNSETOWNER as _, uid as libc::c_ulong);
            }
        }

        let ret = unsafe { libc::ioctl(fd, TUNSETPERSIST as _, 1) };
        unsafe { libc::close(fd) };
        if ret < 0 {
            eprintln!(
                "tunctl: failed to set persistent: {}",
                io::Error::last_os_error()
            );
            return Ok(1);
        }

        let stdout = io::stdout();
        let mut out = stdout.lock();
        let _ = writeln!(out, "Set '{}' persistent", tun_name);
        Ok(0)
    }
}

pub struct SlattachApplet;
impl Applet for SlattachApplet {
    fn name(&self) -> &'static str {
        "slattach"
    }
    fn description(&self) -> &'static str {
        "Attach serial line to network interface (SLIP)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut tty_dev: Option<String> = None;

        for arg in args {
            let s = arg.to_string_lossy();
            if !s.starts_with('-') {
                tty_dev = Some(s.to_string());
                break;
            }
        }

        let dev = match tty_dev {
            Some(d) => d,
            None => {
                eprintln!("Usage: slattach [-p protocol] <tty>");
                return Ok(1);
            }
        };

        let path = if dev.starts_with('/') {
            dev
        } else {
            format!("/dev/{}", dev)
        };
        let c_path = match CString::new(path.as_bytes()) {
            Ok(c) => c,
            Err(_) => return Ok(1),
        };

        let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDWR | libc::O_NOCTTY) };
        if fd < 0 {
            eprintln!("slattach: open {}: {}", path, io::Error::last_os_error());
            return Ok(1);
        }

        let ldisc: libc::c_int = 1;
        let ret = unsafe { libc::ioctl(fd, libc::TIOCSETD as _, &ldisc) };
        if ret < 0 {
            eprintln!("slattach: TIOCSETD: {}", io::Error::last_os_error());
            unsafe { libc::close(fd) };
            return Ok(1);
        }

        unsafe { libc::close(fd) };
        Ok(0)
    }
}

pub struct BrctlApplet;
impl Applet for BrctlApplet {
    fn name(&self) -> &'static str {
        "brctl"
    }
    fn description(&self) -> &'static str {
        "Ethernet bridge administration"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("Usage: brctl addbr|delbr|addif|delif|show ...");
            return Ok(1);
        }

        let cmd = args[0].to_string_lossy();
        let fd = open_socket_dgram()?;

        match cmd.as_ref() {
            "addbr" => {
                if args.len() < 2 {
                    eprintln!("brctl: addbr <bridge>");
                    unsafe { libc::close(fd) };
                    return Ok(1);
                }
                let brname = CString::new(args[1].as_bytes()).unwrap();
                let ret = unsafe { libc::ioctl(fd, SIOCBRADDBR as _, brname.as_ptr()) };
                unsafe { libc::close(fd) };
                if ret < 0 {
                    eprintln!("brctl addbr: {}", io::Error::last_os_error());
                    return Ok(1);
                }
            }
            "delbr" => {
                if args.len() < 2 {
                    eprintln!("brctl: delbr <bridge>");
                    unsafe { libc::close(fd) };
                    return Ok(1);
                }
                let brname = CString::new(args[1].as_bytes()).unwrap();
                let ret = unsafe { libc::ioctl(fd, SIOCBRDELBR as _, brname.as_ptr()) };
                unsafe { libc::close(fd) };
                if ret < 0 {
                    eprintln!("brctl delbr: {}", io::Error::last_os_error());
                    return Ok(1);
                }
            }
            "addif" | "delif" => {
                if args.len() < 3 {
                    eprintln!("brctl {} <bridge> <device>", cmd);
                    unsafe { libc::close(fd) };
                    return Ok(1);
                }
                let brname = args[1].as_bytes();
                let devname = args[2].as_bytes();

                let mut ifr: libc::ifreq = unsafe { mem::zeroed() };
                set_ifr_name(&mut ifr, brname);

                let dev_c = CString::new(devname).unwrap();
                let ifindex = unsafe { libc::if_nametoindex(dev_c.as_ptr()) };
                ifr.ifr_ifru.ifru_ifindex = ifindex as libc::c_int;

                let req = if cmd == "addif" {
                    SIOCBRADDIF
                } else {
                    SIOCBRDELIF
                };
                let ret = unsafe { libc::ioctl(fd, req as _, &ifr) };
                unsafe { libc::close(fd) };
                if ret < 0 {
                    eprintln!("brctl {}: {}", cmd, io::Error::last_os_error());
                    return Ok(1);
                }
            }
            "show" => {
                unsafe { libc::close(fd) };
                let stdout = io::stdout();
                let mut out = stdout.lock();
                let _ = writeln!(out, "bridge name\tbridge id\t\tSTP enabled\tinterfaces");

                if let Ok(entries) = fs::read_dir("/sys/class/net") {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let br_path = path.join("bridge");
                        if br_path.exists() {
                            let name = entry.file_name().to_string_lossy().to_string();
                            let mut ifaces = Vec::new();
                            let brif_path = path.join("brif");
                            if let Ok(ifs) = fs::read_dir(brif_path) {
                                for ife in ifs.flatten() {
                                    ifaces.push(ife.file_name().to_string_lossy().to_string());
                                }
                            }
                            let _ = writeln!(
                                out,
                                "{}\t\t8000.000000000000\tno\t\t{}",
                                name,
                                ifaces.join("\n\t\t\t\t\t\t\t")
                            );
                        }
                    }
                }
            }
            _ => {
                unsafe { libc::close(fd) };
                eprintln!("brctl: unknown command '{}'", cmd);
                return Ok(1);
            }
        }

        Ok(0)
    }
}
