use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self};
use std::io::{self, BufRead, BufReader, Write};
use std::mem;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, TcpStream, ToSocketAddrs};
use std::os::unix::ffi::OsStrExt;
use std::thread;
use std::time::{Duration, Instant};


pub const SIOCBRADDBR: libc::c_ulong = 0x89a0;
pub const SIOCBRDELBR: libc::c_ulong = 0x89a1;
pub const SIOCBRADDIF: libc::c_ulong = 0x89a2;
pub const SIOCBRDELIF: libc::c_ulong = 0x89a3;
pub const SIOCSIFVLAN: libc::c_ulong = 0x8983;
pub const SIOCBONDENSLAVE: libc::c_ulong = 0x8990;
pub const SIOCBONDRELEASE: libc::c_ulong = 0x8991;
pub const TUNSETIFF: libc::c_ulong = 0x400454ca;
pub const TUNSETPERSIST: libc::c_ulong = 0x400454cb;
pub const TUNSETOWNER: libc::c_ulong = 0x400454cc;
pub const IFF_TAP: libc::c_short = 0x0002;
pub const IFF_NO_PI: libc::c_short = 0x1000;
pub const SIOCADDTUNNEL: libc::c_ulong = 0x89f0 + 1;
pub const SIOCDELTUNNEL: libc::c_ulong = 0x89f0 + 2;

pub fn set_ifr_name(ifr: &mut libc::ifreq, name: &[u8]) {
    for b in ifr.ifr_name.iter_mut() {
        *b = 0;
    }
    let len = name.len().min(libc::IFNAMSIZ - 1);
    for (i, &b) in name[..len].iter().enumerate() {
        ifr.ifr_name[i] = b as libc::c_char;
    }
}

pub fn open_socket_dgram() -> io::Result<libc::c_int> {
    let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
    if fd < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(fd)
    }
}

pub fn format_mac(hw: &[u8]) -> String {
    if hw.len() >= 6 {
        format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            hw[0], hw[1], hw[2], hw[3], hw[4], hw[5]
        )
    } else {
        String::new()
    }
}

pub fn parse_mac(s: &str) -> Option<[u8; 6]> {
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



pub fn percent_decode(s: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        if s[i] == b'%' && i + 2 < s.len() {
            let h1 = (s[i + 1] as char).to_digit(16);
            let h2 = (s[i + 2] as char).to_digit(16);
            if let (Some(h1), Some(h2)) = (h1, h2) {
                out.push(((h1 << 4) | h2) as u8);
                i += 3;
                continue;
            }
        }
        out.push(s[i]);
        i += 1;
    }
    out
}



pub fn run_ip_link(args: &[OsString]) -> Result<i32> {
    if args.is_empty() || args[0].to_string_lossy() == "show" || args[0].to_string_lossy() == "list"
    {
        return crate::applets::networking::ifconfig::IfconfigApplet.run(&[OsString::from("-a")]);
    }

    if args[0].to_string_lossy() == "set" && args.len() >= 3 {
        let ifname = &args[1];
        let action = &args[2];
        return crate::applets::networking::ifconfig::IfconfigApplet.run(&[ifname.clone(), action.clone()]);
    }

    crate::applets::networking::ifconfig::IfconfigApplet.run(&[OsString::from("-a")])
}

pub fn run_ip_addr(args: &[OsString]) -> Result<i32> {
    if args.is_empty() || args[0].to_string_lossy() == "show" || args[0].to_string_lossy() == "list"
    {
        return crate::applets::networking::ifconfig::IfconfigApplet.run(&[OsString::from("-a")]);
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
        return crate::applets::networking::ifconfig::IfconfigApplet.run(&[dev, OsString::from(ip_only)]);
    }

    crate::applets::networking::ifconfig::IfconfigApplet.run(&[OsString::from("-a")])
}

pub fn run_ip_route(args: &[OsString]) -> Result<i32> {
    if args.is_empty() || args[0].to_string_lossy() == "show" || args[0].to_string_lossy() == "list"
    {
        return crate::applets::networking::route::RouteApplet.run(&[]);
    }
    crate::applets::networking::route::RouteApplet.run(args)
}

pub fn run_ip_neigh(args: &[OsString]) -> Result<i32> {
    if args.is_empty() || args[0].to_string_lossy() == "show" || args[0].to_string_lossy() == "list"
    {
        return crate::applets::networking::arp::ArpApplet.run(&[]);
    }
    crate::applets::networking::arp::ArpApplet.run(args)
}

pub fn run_ip_rule(_args: &[OsString]) -> Result<i32> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let _ = writeln!(
        out,
        "0:\tfrom all lookup local\n32766:\tfrom all lookup main\n32767:\tfrom all lookup default"
    );
    Ok(0)
}

pub fn run_ip_tunnel(args: &[OsString]) -> Result<i32> {
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



#[repr(C)]
pub struct VlanIoctlArgs {
    pub cmd: libc::c_int,
    pub device1: [libc::c_char; 24],
    pub u: VlanUnion,
}

#[repr(C)]
pub union VlanUnion {
    pub device2: [libc::c_char; 24],
    pub vlan_qos: libc::c_int,
    pub vlan_id: libc::c_uint,
    pub flag: libc::c_uint,
}



pub fn ftp_send_cmd(
    reader: &mut BufReader<TcpStream>,
    writer: &mut TcpStream,
    cmd: &str,
) -> io::Result<(u16, String)> {
    if !cmd.is_empty() {
        writer.write_all(cmd.as_bytes())?;
        writer.write_all(b"\r\n")?;
        writer.flush()?;
    }
    let mut line = String::new();
    let mut code = 0u16;
    let mut msg = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        msg.push_str(&line);
        if line.len() >= 3 {
            if let Ok(c) = line[..3].parse::<u16>() {
                code = c;
                if line.len() == 3 || line.as_bytes()[3] == b' ' {
                    break;
                }
            }
        }
    }
    Ok((code, msg))
}

pub fn ftp_enter_pasv(
    reader: &mut BufReader<TcpStream>,
    writer: &mut TcpStream,
) -> io::Result<TcpStream> {
    let (code, msg) = ftp_send_cmd(reader, writer, "PASV")?;
    if code != 227 {
        return Err(io::Error::other(format!("PASV failed: {}", msg)));
    }

    let start = msg
        .find('(')
        .ok_or_else(|| io::Error::other("Invalid PASV"))?;
    let end = msg
        .find(')')
        .ok_or_else(|| io::Error::other("Invalid PASV"))?;
    let parts: Vec<&str> = msg[start + 1..end].split(',').collect();
    if parts.len() != 6 {
        return Err(io::Error::other("Invalid PASV parts"));
    }
    let h1: u8 = parts[0].trim().parse().unwrap_or(0);
    let h2: u8 = parts[1].trim().parse().unwrap_or(0);
    let h3: u8 = parts[2].trim().parse().unwrap_or(0);
    let h4: u8 = parts[3].trim().parse().unwrap_or(0);
    let p1: u16 = parts[4].trim().parse().unwrap_or(0);
    let p2: u16 = parts[5].trim().parse().unwrap_or(0);
    let port = (p1 << 8) | p2;
    let ip = Ipv4Addr::new(h1, h2, h3, h4);
    TcpStream::connect((ip, port))
}



pub fn run_ping_generic(args: &[OsString], ipv6_only: bool) -> Result<i32> {
    let mut count = 4;
    let mut host_arg: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let s = args[i].to_string_lossy();
        if s == "-c" && i + 1 < args.len() {
            count = args[i + 1].to_string_lossy().parse().unwrap_or(4);
            i += 1;
        } else if !s.starts_with('-') {
            host_arg = Some(s.to_string());
        }
        i += 1;
    }

    let host = match host_arg {
        Some(h) => h,
        None => {
            eprintln!("Usage: ping [-c count] <host>");
            return Ok(1);
        }
    };

    let addr = match (host.as_str(), 0).to_socket_addrs() {
        Ok(mut iter) => {
            if ipv6_only {
                iter.find(|a| a.is_ipv6())
            } else {
                iter.find(|a| a.is_ipv4())
                    .or_else(|| iter.find(|a| a.is_ipv6()))
            }
        }
        Err(e) => {
            eprintln!("ping: bad address '{}': {}", host, e);
            return Ok(1);
        }
    };

    let target = match addr {
        Some(a) => a.ip(),
        None => {
            eprintln!("ping: unknown host {}", host);
            return Ok(1);
        }
    };

    let is_v6 = target.is_ipv6();
    let (family, proto) = if is_v6 {
        (libc::AF_INET6, libc::IPPROTO_ICMPV6)
    } else {
        (libc::AF_INET, libc::IPPROTO_ICMP)
    };

    let mut fd = unsafe { libc::socket(family, libc::SOCK_RAW, proto) };
    if fd < 0 {
        fd = unsafe { libc::socket(family, libc::SOCK_DGRAM, proto) };
    }
    if fd < 0 {
        eprintln!("ping: socket: {}", io::Error::last_os_error());
        return Ok(1);
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

    let stdout = io::stdout();
    let mut out = stdout.lock();
    let _ = writeln!(out, "PING {} ({}) 56(84) bytes of data.", host, target);

    let pid = std::process::id() as u16;
    let mut transmitted = 0;
    let mut received = 0;

    for seq in 1..=count {
        let mut packet = [0u8; 64];
        packet[0] = if is_v6 { 128 } else { 8 };
        packet[1] = 0;
        packet[4..6].copy_from_slice(&pid.to_be_bytes());
        packet[6..8].copy_from_slice(&(seq as u16).to_be_bytes());

        for (idx, b) in packet[8..].iter_mut().enumerate() {
            *b = (idx & 0xff) as u8;
        }

        if !is_v6 {
            let csum = checksum(&packet);
            packet[2..4].copy_from_slice(&csum.to_be_bytes());
        }

        let start = Instant::now();
        let ret = match target {
            IpAddr::V4(v4) => {
                let mut sin: libc::sockaddr_in = unsafe { mem::zeroed() };
                sin.sin_family = libc::AF_INET as libc::sa_family_t;
                sin.sin_addr.s_addr = u32::from_ne_bytes(v4.octets());
                unsafe {
                    libc::sendto(
                        fd,
                        packet.as_ptr() as *const libc::c_void,
                        packet.len(),
                        0,
                        &sin as *const _ as *const libc::sockaddr,
                        mem::size_of_val(&sin) as libc::socklen_t,
                    )
                }
            }
            IpAddr::V6(v6) => {
                let mut sin6: libc::sockaddr_in6 = unsafe { mem::zeroed() };
                sin6.sin6_family = libc::AF_INET6 as libc::sa_family_t;
                sin6.sin6_addr.s6_addr = v6.octets();
                unsafe {
                    libc::sendto(
                        fd,
                        packet.as_ptr() as *const libc::c_void,
                        packet.len(),
                        0,
                        &sin6 as *const _ as *const libc::sockaddr,
                        mem::size_of_val(&sin6) as libc::socklen_t,
                    )
                }
            }
        };

        if ret > 0 {
            transmitted += 1;
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

        let elapsed = start.elapsed();
        let rtt_ms = elapsed.as_secs_f64() * 1000.0;

        if n > 0 {
            received += 1;
            let _ = writeln!(
                out,
                "64 bytes from {}: icmp_seq={} ttl=64 time={:.2} ms",
                target, seq, rtt_ms
            );
        }

        thread::sleep(Duration::from_millis(500));
    }

    unsafe { libc::close(fd) };
    let _ = writeln!(
        out,
        "\n--- {} ping statistics ---\n{} packets transmitted, {} received, {}% packet loss",
        host,
        transmitted,
        received,
        if transmitted > 0 {
            ((transmitted - received) * 100) / transmitted
        } else {
            0
        }
    );

    Ok(if received > 0 { 0 } else { 1 })
}



pub fn run_traceroute_generic(args: &[OsString], ipv6_only: bool) -> Result<i32> {
    let mut max_ttl = 30;
    let mut host_arg: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let s = args[i].to_string_lossy();
        if s == "-m" && i + 1 < args.len() {
            max_ttl = args[i + 1].to_string_lossy().parse().unwrap_or(30);
            i += 1;
        } else if !s.starts_with('-') {
            host_arg = Some(s.to_string());
        }
        i += 1;
    }

    let host = match host_arg {
        Some(h) => h,
        None => {
            eprintln!("Usage: traceroute [-m max_ttl] <host>");
            return Ok(1);
        }
    };

    let addr = match (host.as_str(), 0).to_socket_addrs() {
        Ok(mut iter) => {
            if ipv6_only {
                iter.find(|a| a.is_ipv6())
            } else {
                iter.find(|a| a.is_ipv4())
                    .or_else(|| iter.find(|a| a.is_ipv6()))
            }
        }
        Err(e) => {
            eprintln!("traceroute: bad address '{}': {}", host, e);
            return Ok(1);
        }
    };

    let target = match addr {
        Some(a) => a.ip(),
        None => {
            eprintln!("traceroute: unknown host {}", host);
            return Ok(1);
        }
    };

    let stdout = io::stdout();
    let mut out = stdout.lock();
    let _ = writeln!(
        out,
        "traceroute to {} ({}), {} hops max, 60 byte packets",
        host, target, max_ttl
    );

    let is_v6 = target.is_ipv6();
    let dest_port = 33434u16;

    for ttl in 1..=max_ttl {
        let send_sock = match target {
            IpAddr::V4(_) => {
                let s = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
                if s >= 0 {
                    let ttl_val = ttl as libc::c_int;
                    unsafe {
                        libc::setsockopt(
                            s,
                            libc::IPPROTO_IP,
                            libc::IP_TTL,
                            &ttl_val as *const _ as *const libc::c_void,
                            mem::size_of_val(&ttl_val) as libc::socklen_t,
                        );
                    }
                }
                s
            }
            IpAddr::V6(_) => {
                let s = unsafe { libc::socket(libc::AF_INET6, libc::SOCK_DGRAM, 0) };
                if s >= 0 {
                    let ttl_val = ttl as libc::c_int;
                    unsafe {
                        libc::setsockopt(
                            s,
                            libc::IPPROTO_IPV6,
                            libc::IPV6_UNICAST_HOPS,
                            &ttl_val as *const _ as *const libc::c_void,
                            mem::size_of_val(&ttl_val) as libc::socklen_t,
                        );
                    }
                }
                s
            }
        };

        if send_sock < 0 {
            let _ = writeln!(out, " {:2}  *", ttl);
            continue;
        }

        let start = Instant::now();
        let payload = [0u8; 32];

        match target {
            IpAddr::V4(v4) => {
                let mut sin: libc::sockaddr_in = unsafe { mem::zeroed() };
                sin.sin_family = libc::AF_INET as libc::sa_family_t;
                sin.sin_port = dest_port.to_be();
                sin.sin_addr.s_addr = u32::from_ne_bytes(v4.octets());
                unsafe {
                    libc::sendto(
                        send_sock,
                        payload.as_ptr() as *const libc::c_void,
                        payload.len(),
                        0,
                        &sin as *const _ as *const libc::sockaddr,
                        mem::size_of_val(&sin) as libc::socklen_t,
                    );
                }
            }
            IpAddr::V6(v6) => {
                let mut sin6: libc::sockaddr_in6 = unsafe { mem::zeroed() };
                sin6.sin6_family = libc::AF_INET6 as libc::sa_family_t;
                sin6.sin6_port = dest_port.to_be();
                sin6.sin6_addr.s6_addr = v6.octets();
                unsafe {
                    libc::sendto(
                        send_sock,
                        payload.as_ptr() as *const libc::c_void,
                        payload.len(),
                        0,
                        &sin6 as *const _ as *const libc::sockaddr,
                        mem::size_of_val(&sin6) as libc::socklen_t,
                    );
                }
            }
        }
        unsafe { libc::close(send_sock) };

        let icmp_sock = unsafe {
            libc::socket(
                if is_v6 { libc::AF_INET6 } else { libc::AF_INET },
                libc::SOCK_RAW,
                if is_v6 {
                    libc::IPPROTO_ICMPV6
                } else {
                    libc::IPPROTO_ICMP
                },
            )
        };

        let mut responder: Option<IpAddr> = None;
        if icmp_sock >= 0 {
            let tv = libc::timeval {
                tv_sec: 1,
                tv_usec: 0,
            };
            unsafe {
                libc::setsockopt(
                    icmp_sock,
                    libc::SOL_SOCKET,
                    libc::SO_RCVTIMEO,
                    &tv as *const _ as *const libc::c_void,
                    mem::size_of_val(&tv) as libc::socklen_t,
                );
            }

            let mut rx_buf = [0u8; 512];
            let mut from_addr: libc::sockaddr_storage = unsafe { mem::zeroed() };
            let mut from_len = mem::size_of_val(&from_addr) as libc::socklen_t;

            let n = unsafe {
                libc::recvfrom(
                    icmp_sock,
                    rx_buf.as_mut_ptr() as *mut libc::c_void,
                    rx_buf.len(),
                    0,
                    &mut from_addr as *mut _ as *mut libc::sockaddr,
                    &mut from_len,
                )
            };

            if n > 0 {
                if from_addr.ss_family == libc::AF_INET as libc::sa_family_t {
                    let sin = &from_addr as *const _ as *const libc::sockaddr_in;
                    let octets = unsafe { (*sin).sin_addr.s_addr.to_ne_bytes() };
                    responder = Some(IpAddr::V4(Ipv4Addr::from(octets)));
                } else if from_addr.ss_family == libc::AF_INET6 as libc::sa_family_t {
                    let sin6 = &from_addr as *const _ as *const libc::sockaddr_in6;
                    let octets = unsafe { (*sin6).sin6_addr.s6_addr };
                    responder = Some(IpAddr::V6(Ipv6Addr::from(octets)));
                }
            }
            unsafe { libc::close(icmp_sock) };
        }

        let elapsed = start.elapsed();
        let rtt_ms = elapsed.as_secs_f64() * 1000.0;

        if let Some(resp) = responder {
            let _ = writeln!(out, " {:2}  {}  {:.3} ms", ttl, resp, rtt_ms);
            if resp == target {
                break;
            }
        } else {
            let _ = writeln!(out, " {:2}  *", ttl);
        }
    }

    Ok(0)
}
pub use super::nc::checksum;
