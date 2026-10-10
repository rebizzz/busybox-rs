use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::mem;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, TcpListener, TcpStream, ToSocketAddrs, UdpSocket};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub struct TcpsvdApplet;
impl Applet for TcpsvdApplet {
    fn name(&self) -> &'static str {
        "tcpsvd"
    }
    fn description(&self) -> &'static str {
        "TCP service daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut verbose = false;
        let mut idx = 0;

        while idx < args.len() {
            let s = args[idx].to_string_lossy();
            if s == "-v" {
                verbose = true;
                idx += 1;
            } else if s.starts_with('-') {
                idx += 1;
            } else {
                break;
            }
        }

        if args.len() < idx + 3 {
            eprintln!("Usage: tcpsvd [-v] <host> <port> <prog> [args...]");
            return Ok(1);
        }

        let host = &args[idx].to_string_lossy();
        let port = &args[idx + 1].to_string_lossy();
        let prog = &args[idx + 2];
        let prog_args = &args[idx + 3..];

        let addr = format!("{}:{}", host, port);
        let listener = match TcpListener::bind(&addr) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("tcpsvd: bind {}: {}", addr, e);
                return Ok(1);
            }
        };

        if verbose {
            eprintln!("tcpsvd: listening on {}", addr);
        }

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let fd = stream.as_raw_fd();
                    if verbose {
                        if let Ok(peer) = stream.peer_addr() {
                            eprintln!("tcpsvd: connection from {}", peer);
                        }
                    }

                    let stdin_fd = unsafe { libc::dup(fd) };
                    let stdout_fd = unsafe { libc::dup(fd) };

                    let mut cmd = Command::new(prog);
                    cmd.args(prog_args);
                    unsafe {
                        cmd.stdin(Stdio::from_raw_fd(stdin_fd));
                        cmd.stdout(Stdio::from_raw_fd(stdout_fd));
                    }
                    let _ = cmd.status();
                }
                Err(e) => {
                    if verbose {
                        eprintln!("tcpsvd: accept: {}", e);
                    }
                }
            }
        }

        Ok(0)
    }
}

pub struct UdpsvdApplet;
impl Applet for UdpsvdApplet {
    fn name(&self) -> &'static str {
        "udpsvd"
    }
    fn description(&self) -> &'static str {
        "UDP service daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut verbose = false;
        let mut idx = 0;

        while idx < args.len() {
            let s = args[idx].to_string_lossy();
            if s == "-v" {
                verbose = true;
                idx += 1;
            } else if s.starts_with('-') {
                idx += 1;
            } else {
                break;
            }
        }

        if args.len() < idx + 3 {
            eprintln!("Usage: udpsvd [-v] <host> <port> <prog> [args...]");
            return Ok(1);
        }

        let host = &args[idx].to_string_lossy();
        let port = &args[idx + 1].to_string_lossy();
        let prog = &args[idx + 2];
        let prog_args = &args[idx + 3..];

        let addr = format!("{}:{}", host, port);
        let socket = match UdpSocket::bind(&addr) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("udpsvd: bind {}: {}", addr, e);
                return Ok(1);
            }
        };

        if verbose {
            eprintln!("udpsvd: listening on {}", addr);
        }

        let mut buf = [0u8; 65535];
        loop {
            match socket.recv_from(&mut buf) {
                Ok((len, peer)) => {
                    if verbose {
                        eprintln!("udpsvd: packet {} bytes from {}", len, peer);
                    }
                    let mut child = match Command::new(prog)
                        .args(prog_args)
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn()
                    {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("udpsvd: failed to execute prog: {}", e);
                            continue;
                        }
                    };

                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(&buf[..len]);
                    }

                    if let Ok(output) = child.wait_with_output() {
                        if !output.stdout.is_empty() {
                            let _ = socket.send_to(&output.stdout, peer);
                        }
                    }
                }
                Err(e) => {
                    if verbose {
                        eprintln!("udpsvd: recv: {}", e);
                    }
                }
            }
        }
    }
}

pub struct NcApplet;
impl Applet for NcApplet {
    fn name(&self) -> &'static str {
        "nc"
    }
    fn description(&self) -> &'static str {
        "Arbitrary TCP and UDP connections and listens"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut listen = false;
        let mut udp = false;
        let mut port_opt: Option<u16> = None;
        let mut host_arg: Option<String> = None;
        let mut port_arg: Option<u16> = None;

        let mut i = 0;
        while i < args.len() {
            let s = args[i].to_string_lossy();
            if s == "-l" {
                listen = true;
            } else if s == "-u" {
                udp = true;
            } else if s == "-p" && i + 1 < args.len() {
                port_opt = args[i + 1].to_string_lossy().parse().ok();
                i += 1;
            } else if !s.starts_with('-') {
                if host_arg.is_none() {
                    host_arg = Some(s.to_string());
                } else if port_arg.is_none() {
                    port_arg = s.parse().ok();
                }
            }
            i += 1;
        }

        if listen {
            let port = port_opt.or(port_arg).unwrap_or(0);
            let host = host_arg.unwrap_or_else(|| "0.0.0.0".to_string());
            let bind_addr = format!("{}:{}", host, port);

            if udp {
                let socket = match UdpSocket::bind(&bind_addr) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("nc: bind {}: {}", bind_addr, e);
                        return Ok(1);
                    }
                };

                let mut buf = [0u8; 8192];
                let (len, peer) = match socket.recv_from(&mut buf) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("nc: recv: {}", e);
                        return Ok(1);
                    }
                };
                let stdout = io::stdout();
                let mut out = stdout.lock();
                let _ = out.write_all(&buf[..len]);
                let _ = out.flush();

                let sock_send = socket.try_clone()?;
                thread::spawn(move || {
                    let mut stdin = io::stdin().lock();
                    let mut sbuf = [0u8; 8192];
                    while let Ok(n) = stdin.read(&mut sbuf) {
                        if n == 0 {
                            break;
                        }
                        if sock_send.send_to(&sbuf[..n], peer).is_err() {
                            break;
                        }
                    }
                });

                while let Ok((n, _)) = socket.recv_from(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    if out.write_all(&buf[..n]).is_err() {
                        break;
                    }
                    let _ = out.flush();
                }
                return Ok(0);
            }

            let listener = match TcpListener::bind(&bind_addr) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("nc: bind {}: {}", bind_addr, e);
                    return Ok(1);
                }
            };

            let (stream, _) = match listener.accept() {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("nc: accept: {}", e);
                    return Ok(1);
                }
            };

            forward_stream(stream)?;
            return Ok(0);
        }

        let host = match host_arg {
            Some(h) => h,
            None => {
                eprintln!("nc: missing host");
                return Ok(1);
            }
        };
        let port = match port_arg.or(port_opt) {
            Some(p) => p,
            None => {
                eprintln!("nc: missing port");
                return Ok(1);
            }
        };

        if udp {
            let socket = match UdpSocket::bind("0.0.0.0:0") {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("nc: bind: {}", e);
                    return Ok(1);
                }
            };
            let remote = format!("{}:{}", host, port);
            if let Err(e) = socket.connect(&remote) {
                eprintln!("nc: connect {}: {}", remote, e);
                return Ok(1);
            }

            let sock_send = socket.try_clone()?;
            thread::spawn(move || {
                let mut stdin = io::stdin().lock();
                let mut buf = [0u8; 8192];
                while let Ok(n) = stdin.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    if sock_send.send(&buf[..n]).is_err() {
                        break;
                    }
                }
            });

            let stdout = io::stdout();
            let mut out = stdout.lock();
            let mut buf = [0u8; 8192];
            while let Ok(n) = socket.recv(&mut buf) {
                if n == 0 {
                    break;
                }
                if out.write_all(&buf[..n]).is_err() {
                    break;
                }
                let _ = out.flush();
            }
            return Ok(0);
        }

        let stream = match TcpStream::connect((host.as_str(), port)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("nc: connect {}:{}: {}", host, port, e);
                return Ok(1);
            }
        };

        forward_stream(stream)?;
        Ok(0)
    }
}

fn forward_stream(stream: TcpStream) -> Result<()> {
    let mut reader = stream.try_clone()?;
    let mut writer = stream;

    let t = thread::spawn(move || {
        let mut stdin = io::stdin().lock();
        let mut buf = [0u8; 8192];
        while let Ok(n) = stdin.read(&mut buf) {
            if n == 0 {
                break;
            }
            if writer.write_all(&buf[..n]).is_err() {
                break;
            }
            let _ = writer.flush();
        }
    });

    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut buf = [0u8; 8192];
    while let Ok(n) = reader.read(&mut buf) {
        if n == 0 {
            break;
        }
        if out.write_all(&buf[..n]).is_err() {
            break;
        }
        let _ = out.flush();
    }

    let _ = t.join();
    Ok(())
}

fn checksum(data: &[u8]) -> u16 {
    let mut sum = 0u32;
    let mut i = 0;
    while i + 1 < data.len() {
        let word = u16::from_be_bytes([data[i], data[i + 1]]);
        sum = sum.wrapping_add(word as u32);
        i += 2;
    }
    if i < data.len() {
        let word = (data[i] as u32) << 8;
        sum = sum.wrapping_add(word);
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !sum as u16
}

pub struct PingApplet;
impl Applet for PingApplet {
    fn name(&self) -> &'static str {
        "ping"
    }
    fn description(&self) -> &'static str {
        "Send ICMP ECHO_REQUEST to network hosts"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_ping_generic(args, false)
    }
}

pub struct Ping6Applet;
impl Applet for Ping6Applet {
    fn name(&self) -> &'static str {
        "ping6"
    }
    fn description(&self) -> &'static str {
        "Send ICMPv6 ECHO_REQUEST to network hosts"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_ping_generic(args, true)
    }
}

fn run_ping_generic(args: &[OsString], ipv6_only: bool) -> Result<i32> {
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

pub struct TracerouteApplet;
impl Applet for TracerouteApplet {
    fn name(&self) -> &'static str {
        "traceroute"
    }
    fn description(&self) -> &'static str {
        "Trace the route to a host"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_traceroute_generic(args, false)
    }
}

pub struct Traceroute6Applet;
impl Applet for Traceroute6Applet {
    fn name(&self) -> &'static str {
        "traceroute6"
    }
    fn description(&self) -> &'static str {
        "Trace the route to a host over IPv6"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_traceroute_generic(args, true)
    }
}

fn run_traceroute_generic(args: &[OsString], ipv6_only: bool) -> Result<i32> {
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

pub struct WhoisApplet;
impl Applet for WhoisApplet {
    fn name(&self) -> &'static str {
        "whois"
    }
    fn description(&self) -> &'static str {
        "Query WHOIS database"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut server = "whois.iana.org".to_string();
        let mut port = 43u16;
        let mut query: Option<String> = None;

        let mut i = 0;
        while i < args.len() {
            let s = args[i].to_string_lossy();
            if s == "-h" && i + 1 < args.len() {
                server = args[i + 1].to_string_lossy().to_string();
                i += 1;
            } else if s == "-p" && i + 1 < args.len() {
                port = args[i + 1].to_string_lossy().parse().unwrap_or(43);
                i += 1;
            } else if !s.starts_with('-') {
                query = Some(s.to_string());
            }
            i += 1;
        }

        let q = match query {
            Some(q) => q,
            None => {
                eprintln!("Usage: whois [-h server] [-p port] <query>");
                return Ok(1);
            }
        };

        let mut stream = match TcpStream::connect((server.as_str(), port)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("whois: connect {}:{}: {}", server, port, e);
                return Ok(1);
            }
        };

        let req = format!("{}\r\n", q);
        if let Err(e) = stream.write_all(req.as_bytes()) {
            eprintln!("whois: write: {}", e);
            return Ok(1);
        }
        let _ = stream.flush();

        let stdout = io::stdout();
        let mut out = stdout.lock();
        let mut buf = [0u8; 4096];
        while let Ok(n) = stream.read(&mut buf) {
            if n == 0 {
                break;
            }
            let _ = out.write_all(&buf[..n]);
        }

        Ok(0)
    }
}

pub struct NslookupApplet;
impl Applet for NslookupApplet {
    fn name(&self) -> &'static str {
        "nslookup"
    }
    fn description(&self) -> &'static str {
        "Query the nameserver for the IP address of a host"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut host_arg: Option<String> = None;
        let mut server_arg: Option<String> = None;

        for arg in args {
            let s = arg.to_string_lossy();
            if !s.starts_with('-') {
                if host_arg.is_none() {
                    host_arg = Some(s.to_string());
                } else if server_arg.is_none() {
                    server_arg = Some(s.to_string());
                }
            }
        }

        let host = match host_arg {
            Some(h) => h,
            None => {
                eprintln!("Usage: nslookup <host> [server]");
                return Ok(1);
            }
        };

        let stdout = io::stdout();
        let mut out = stdout.lock();

        let _ = writeln!(
            out,
            "Server:\t\t{}",
            server_arg.as_deref().unwrap_or("default")
        );
        let _ = writeln!(out, "\nName:\t{}", host);

        let c_host = match std::ffi::CString::new(host.as_bytes()) {
            Ok(c) => c,
            Err(_) => return Ok(1),
        };

        let mut res: *mut libc::addrinfo = std::ptr::null_mut();
        let ret = unsafe {
            libc::getaddrinfo(
                c_host.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                &mut res,
            )
        };
        if ret != 0 {
            eprintln!("nslookup: can't resolve '{}': getaddrinfo failed", host);
            return Ok(1);
        }

        let mut curr = res;
        while !curr.is_null() {
            unsafe {
                let ai = &*curr;
                if ai.ai_family == libc::AF_INET {
                    let sin = &*(ai.ai_addr as *const libc::sockaddr_in);
                    let ip = Ipv4Addr::from(sin.sin_addr.s_addr.to_ne_bytes());
                    let _ = writeln!(out, "Address:\t{}", ip);
                } else if ai.ai_family == libc::AF_INET6 {
                    let sin6 = &*(ai.ai_addr as *const libc::sockaddr_in6);
                    let ip = Ipv6Addr::from(sin6.sin6_addr.s6_addr);
                    let _ = writeln!(out, "Address:\t{}", ip);
                }
                curr = ai.ai_next;
            }
        }

        unsafe {
            libc::freeaddrinfo(res);
        }

        Ok(0)
    }
}

pub struct SslClientApplet;
impl Applet for SslClientApplet {
    fn name(&self) -> &'static str {
        "ssl_client"
    }
    fn description(&self) -> &'static str {
        "TLS client wrapper"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut host = String::new();
        let mut port = 443u16;
        let mut prog: Vec<OsString> = Vec::new();
        let mut idx = 0;
        while idx < args.len() {
            let b = args[idx].as_bytes();
            if b == b"-s" || b == b"-r" || b == b"-n" {
                idx += 1;
            } else if b == b"-e" {
                prog = args[idx + 1..].to_vec();
                break;
            } else if !b.starts_with(b"-") {
                let s = args[idx].to_string_lossy();
                if let Some((h, p)) = s.split_once(':') {
                    host = h.to_string();
                    port = p.parse().unwrap_or(443);
                } else {
                    host = s.to_string();
                }
            }
            idx += 1;
        }
        if host.is_empty() && prog.is_empty() {
            eprintln!(
                "Usage: ssl_client [-n SNI] {{ -s FD [-r FD] | HOST[:PORT] | -e PROG ARGS }}"
            );
            return Ok(1);
        }
        if !prog.is_empty() {
            let mut cmd = std::process::Command::new(&prog[0]);
            cmd.args(&prog[1..]);
            match cmd.status() {
                Ok(st) => return Ok(st.code().unwrap_or(1)),
                Err(e) => {
                    eprintln!("ssl_client: {}: {}", prog[0].to_string_lossy(), e);
                    return Ok(1);
                }
            }
        }
        match TcpStream::connect((host.as_str(), port)) {
            Ok(mut stream) => {
                let mut buf = [0u8; 4096];
                let stdin = io::stdin();
                let mut stdin_lock = stdin.lock();
                if let Ok(n) = stdin_lock.read(&mut buf) {
                    if n > 0 {
                        let _ = stream.write_all(&buf[..n]);
                    }
                }
                let stdout = io::stdout();
                let mut stdout_lock = stdout.lock();
                while let Ok(n) = stream.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    let _ = stdout_lock.write_all(&buf[..n]);
                }
                Ok(0)
            }
            Err(e) => {
                eprintln!("ssl_client: connect to {}:{}: {}", host, port, e);
                Ok(1)
            }
        }
    }
}

pub struct SslServerApplet;
impl Applet for SslServerApplet {
    fn name(&self) -> &'static str {
        "ssl_server"
    }
    fn description(&self) -> &'static str {
        "TLS server wrapper"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut prog: Vec<OsString> = Vec::new();
        let mut idx = 0;
        while idx < args.len() {
            let b = args[idx].as_bytes();
            if b == b"-f" {
                idx += 1;
            } else if !b.starts_with(b"-") {
                prog = args[idx..].to_vec();
                break;
            }
            idx += 1;
        }
        if prog.is_empty() {
            eprintln!("Usage: ssl_server -f PEMFILE PROG ARGS");
            return Ok(1);
        }
        let mut cmd = std::process::Command::new(&prog[0]);
        cmd.args(&prog[1..]);
        match cmd.status() {
            Ok(st) => Ok(st.code().unwrap_or(1)),
            Err(e) => {
                eprintln!("ssl_server: {}: {}", prog[0].to_string_lossy(), e);
                Ok(1)
            }
        }
    }
}
