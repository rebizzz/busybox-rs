use crate::core::{Applet, Result};
use std::ffi::{CStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, TcpStream, UdpSocket};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct ZcipApplet;

impl Applet for ZcipApplet {
    fn name(&self) -> &'static str {
        "zcip"
    }
    fn description(&self) -> &'static str {
        "Manage IPv4 link-local (169.254.x.x) addresses"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut iface = "eth0".to_string();
        let mut script = "/etc/zcip.script".to_string();

        let mut pos = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if !b.starts_with(b"-") {
                pos.push(&args[i]);
            }
            i += 1;
        }

        if !pos.is_empty() {
            iface = String::from_utf8_lossy(pos[0].as_bytes()).to_string();
        }
        if pos.len() > 1 {
            script = String::from_utf8_lossy(pos[1].as_bytes()).to_string();
        }

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(12345);
        let b1 = 1 + ((ts & 0xFD) as u8 % 254);
        let b2 = (ts.wrapping_shr(8) & 0xFF) as u8;
        let chosen_ip = format!("169.254.{}.{}", b1, b2);

        println!("zcip: configuring {} with {}", iface, chosen_ip);

        if Path::new(&script).exists() {
            let _ = Command::new(&script)
                .args(["config", &iface, &chosen_ip])
                .status();
        }

        Ok(0)
    }
}

pub struct WgetApplet;

impl Applet for WgetApplet {
    fn name(&self) -> &'static str {
        "wget"
    }
    fn description(&self) -> &'static str {
        "Retrieve files via HTTP or FTP"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut output_file: Option<PathBuf> = None;
        let mut to_stdout = false;
        let mut url_str: Option<String> = None;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-O" && i + 1 < args.len() {
                i += 1;
                if args[i].as_bytes() == b"-" {
                    to_stdout = true;
                } else {
                    output_file = Some(PathBuf::from(&args[i]));
                }
            } else if b == b"-q" {
            } else if !b.starts_with(b"-") {
                url_str = Some(String::from_utf8_lossy(b).to_string());
            }
            i += 1;
        }

        let url = match url_str {
            Some(u) => u,
            None => {
                eprintln!("wget: missing URL");
                return Ok(1);
            }
        };

        let (host, port, path) = if let Some(stripped) = url.strip_prefix("http://") {
            let (hp, p) = match stripped.find('/') {
                Some(idx) => (&stripped[..idx], &stripped[idx..]),
                None => (stripped, "/"),
            };
            let (h, port) = match hp.find(':') {
                Some(idx) => (&hp[..idx], hp[idx + 1..].parse::<u16>().unwrap_or(80)),
                None => (hp, 80),
            };
            (h.to_string(), port, p.to_string())
        } else {
            eprintln!("wget: only http:// supported");
            return Ok(1);
        };

        let target = format!("{}:{}", host, port);
        let mut stream = match TcpStream::connect(&target) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("wget: connect: {}", e);
                return Ok(1);
            }
        };

        let req = format!(
            "GET {} HTTP/1.0\r\nHost: {}\r\nUser-Agent: Wget\r\nConnection: close\r\n\r\n",
            path, host
        );
        if let Err(e) = stream.write_all(req.as_bytes()) {
            eprintln!("wget: write: {}", e);
            return Ok(1);
        }

        let mut reader = BufReader::new(stream);
        let mut status_line = String::new();
        if reader.read_line(&mut status_line).is_err() || !status_line.contains("200") {
            eprintln!("wget: server response: {}", status_line.trim());
            return Ok(1);
        }

        let mut header = String::new();
        loop {
            header.clear();
            if reader.read_line(&mut header).is_err()
                || header == "\r\n"
                || header == "\n"
                || header.is_empty()
            {
                break;
            }
        }

        let mut out: Box<dyn Write> = if to_stdout {
            Box::new(io::stdout().lock())
        } else {
            let dest_name = output_file.unwrap_or_else(|| {
                let name = path
                    .trim_end_matches('/')
                    .rsplit('/')
                    .next()
                    .unwrap_or("index.html");
                if name.is_empty() {
                    PathBuf::from("index.html")
                } else {
                    PathBuf::from(name)
                }
            });
            Box::new(match File::create(&dest_name) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("wget: {}: {}", dest_name.display(), e);
                    return Ok(1);
                }
            })
        };

        let mut buf = [0u8; 8192];
        while let Ok(n) = reader.read(&mut buf) {
            if n == 0 {
                break;
            }
            if out.write_all(&buf[..n]).is_err() {
                break;
            }
        }
        let _ = out.flush();

        Ok(0)
    }
}

pub struct SendmailApplet;

impl Applet for SendmailApplet {
    fn name(&self) -> &'static str {
        "sendmail"
    }
    fn description(&self) -> &'static str {
        "Send mail via SMTP or pipe"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut server = "127.0.0.1:25".to_string();
        let mut from_addr = "root@localhost".to_string();
        let mut recipients = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-S" && i + 1 < args.len() {
                i += 1;
                server = String::from_utf8_lossy(args[i].as_bytes()).to_string();
                if !server.contains(':') {
                    server.push_str(":25");
                }
            } else if b == b"-f" && i + 1 < args.len() {
                i += 1;
                from_addr = String::from_utf8_lossy(args[i].as_bytes()).to_string();
            } else if !b.starts_with(b"-") {
                recipients.push(String::from_utf8_lossy(b).to_string());
            }
            i += 1;
        }

        if recipients.is_empty() {
            recipients.push("root@localhost".to_string());
        }

        let stream = match TcpStream::connect(&server) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("sendmail: cannot connect to {}: {}", server, e);
                return Ok(1);
            }
        };

        let mut stream_write = match stream.try_clone() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("sendmail: {}", e);
                return Ok(1);
            }
        };
        let mut reader = BufReader::new(stream);

        fn smtp_cmd(
            r: &mut BufReader<TcpStream>,
            w: &mut TcpStream,
            cmd: &str,
        ) -> io::Result<String> {
            if !cmd.is_empty() {
                w.write_all(cmd.as_bytes())?;
                w.write_all(b"\r\n")?;
                w.flush()?;
            }
            let mut line = String::new();
            r.read_line(&mut line)?;
            Ok(line)
        }

        if let Err(e) = (|| -> io::Result<()> {
            smtp_cmd(&mut reader, &mut stream_write, "")?;
            smtp_cmd(&mut reader, &mut stream_write, "HELO localhost")?;
            smtp_cmd(
                &mut reader,
                &mut stream_write,
                &format!("MAIL FROM:<{}>", from_addr),
            )?;
            for rcpt in &recipients {
                smtp_cmd(
                    &mut reader,
                    &mut stream_write,
                    &format!("RCPT TO:<{}>", rcpt),
                )?;
            }
            smtp_cmd(&mut reader, &mut stream_write, "DATA")?;

            let stdin = io::stdin();
            let mut in_reader = BufReader::new(stdin.lock());
            let mut line = String::new();
            while in_reader.read_line(&mut line)? > 0 {
                if line.trim_end() == "." {
                    stream_write.write_all(b"..\r\n")?;
                } else {
                    stream_write.write_all(line.as_bytes())?;
                }
                line.clear();
            }
            smtp_cmd(&mut reader, &mut stream_write, "\r\n.")?;
            smtp_cmd(&mut reader, &mut stream_write, "QUIT")?;
            Ok(())
        })() {
            eprintln!("sendmail: error: {}", e);
            return Ok(1);
        }

        Ok(0)
    }
}

pub struct PopmaildirApplet;

impl Applet for PopmaildirApplet {
    fn name(&self) -> &'static str {
        "popmaildir"
    }
    fn description(&self) -> &'static str {
        "Fetch mail from POP3 server into maildir"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dir = PathBuf::from(".");
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if !b.starts_with(b"-") {
                dir = PathBuf::from(&args[i]);
            }
            i += 1;
        }

        let new_dir = dir.join("new");
        let _ = fs::create_dir_all(&new_dir);

        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut reader = BufReader::new(stdin.lock());
        let mut writer = stdout.lock();

        let mut line = String::new();

        if reader.read_line(&mut line).is_err() {
            return Ok(1);
        }

        let _ = writer.write_all(b"STAT\r\n");
        let _ = writer.flush();
        line.clear();
        if reader.read_line(&mut line).is_err() || !line.starts_with("+OK") {
            return Ok(1);
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        let msg_count: u32 = if parts.len() > 1 {
            parts[1].parse().unwrap_or(0)
        } else {
            0
        };

        for m in 1..=msg_count {
            let retr = format!("RETR {}\r\n", m);
            let _ = writer.write_all(retr.as_bytes());
            let _ = writer.flush();

            line.clear();
            if reader.read_line(&mut line).is_err() || !line.starts_with("+OK") {
                continue;
            }

            let msg_path = new_dir.join(format!("{}.msg", m));
            if let Ok(mut mf) = File::create(msg_path) {
                loop {
                    line.clear();
                    if reader.read_line(&mut line).is_err() {
                        break;
                    }
                    if line == ".\r\n" || line == ".\n" {
                        break;
                    }
                    let _ = mf.write_all(line.as_bytes());
                }
            }
        }

        let _ = writer.write_all(b"QUIT\r\n");
        let _ = writer.flush();
        Ok(0)
    }
}

pub struct NtpdApplet;

impl Applet for NtpdApplet {
    fn name(&self) -> &'static str {
        "ntpd"
    }
    fn description(&self) -> &'static str {
        "NTP client and server daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut peer = "pool.ntp.org".to_string();
        let mut query_only = false;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-p" && i + 1 < args.len() {
                i += 1;
                peer = String::from_utf8_lossy(args[i].as_bytes()).to_string();
            } else if b == b"-q" {
                query_only = true;
            }
            i += 1;
        }

        let sock = match UdpSocket::bind("0.0.0.0:0") {
            Ok(s) => s,
            Err(e) => {
                eprintln!("ntpd: bind: {}", e);
                return Ok(1);
            }
        };
        let _ = sock.set_read_timeout(Some(Duration::from_secs(3)));

        let dest = format!("{}:123", peer);
        let mut pkt = [0u8; 48];
        pkt[0] = 0x1B;

        if sock.send_to(&pkt, &dest).is_err() {
            eprintln!("ntpd: send failed to {}", dest);
            return Ok(1);
        }

        let mut resp = [0u8; 48];
        match sock.recv_from(&mut resp) {
            Ok((len, _)) => {
                if len >= 48 {
                    let sec = u32::from_be_bytes([resp[40], resp[41], resp[42], resp[43]]);

                    if sec > 2208988800 {
                        let unix_sec = sec - 2208988800;
                        println!("ntpd: time from {}: {}", peer, unix_sec);
                        if !query_only {
                            let tv = libc::timeval {
                                tv_sec: unix_sec as libc::time_t,
                                tv_usec: 0,
                            };
                            unsafe {
                                libc::settimeofday(&tv, std::ptr::null());
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("ntpd: recv timeout: {}", e);
                return Ok(1);
            }
        }

        Ok(0)
    }
}

pub struct RdateApplet;

impl Applet for RdateApplet {
    fn name(&self) -> &'static str {
        "rdate"
    }
    fn description(&self) -> &'static str {
        "Get time from remote host via RFC 868"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut host = None;
        let mut set_time = false;
        let mut print_time = false;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-s" {
                set_time = true;
            } else if b == b"-p" {
                print_time = true;
            } else if !b.starts_with(b"-") {
                host = Some(&args[i]);
            }
            i += 1;
        }

        let host = match host {
            Some(h) => String::from_utf8_lossy(h.as_bytes()).to_string(),
            None => {
                eprintln!("Usage: rdate [-s] [-p] host");
                return Ok(1);
            }
        };

        let dest = format!("{}:37", host);
        let mut stream = match TcpStream::connect(&dest) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("rdate: connect: {}", e);
                return Ok(1);
            }
        };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));

        let mut buf = [0u8; 4];
        if stream.read_exact(&mut buf).is_err() {
            eprintln!("rdate: read error");
            return Ok(1);
        }

        let time_rfc868 = u32::from_be_bytes(buf);
        if time_rfc868 >= 2208988800 {
            let unix_sec = time_rfc868 - 2208988800;
            if print_time || !set_time {
                println!("{}", unix_sec);
            }
            if set_time {
                let tv = libc::timeval {
                    tv_sec: unix_sec as libc::time_t,
                    tv_usec: 0,
                };
                unsafe {
                    libc::settimeofday(&tv, std::ptr::null());
                }
            }
        }

        Ok(0)
    }
}

pub struct ChatApplet;

impl Applet for ChatApplet {
    fn name(&self) -> &'static str {
        "chat"
    }
    fn description(&self) -> &'static str {
        "Automate conversational exchange with modem"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut pairs = Vec::new();
        let mut i = 0;
        while i < args.len() {
            if !args[i].as_bytes().starts_with(b"-") {
                pairs.push(String::from_utf8_lossy(args[i].as_bytes()).to_string());
            }
            i += 1;
        }

        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut reader = BufReader::new(stdin.lock());
        let mut writer = stdout.lock();

        let mut idx = 0;
        while idx < pairs.len() {
            let expect = &pairs[idx];
            idx += 1;

            if !expect.is_empty() && expect != "\"\"" {
                let mut matched = false;
                let mut line = String::new();
                while reader.read_line(&mut line).is_ok() && !line.is_empty() {
                    if line.contains(expect) {
                        matched = true;
                        break;
                    }
                    line.clear();
                }
                if !matched {
                    eprintln!("chat: failed to match {}", expect);
                    return Ok(1);
                }
            }

            if idx < pairs.len() {
                let send = &pairs[idx];
                idx += 1;
                let send_s = if send == "\"\"" { "" } else { send };
                let _ = writer.write_all(send_s.as_bytes());
                let _ = writer.write_all(b"\r\n");
                let _ = writer.flush();
            }
        }

        Ok(0)
    }
}

pub struct MicrocomApplet;

impl Applet for MicrocomApplet {
    fn name(&self) -> &'static str {
        "microcom"
    }
    fn description(&self) -> &'static str {
        "Simple serial terminal emulator"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut speed = 9600u32;
        let mut device = None;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-s" && i + 1 < args.len() {
                i += 1;
                speed = String::from_utf8_lossy(args[i].as_bytes())
                    .parse()
                    .unwrap_or(9600);
            } else if !b.starts_with(b"-") {
                device = Some(&args[i]);
            }
            i += 1;
        }

        let dev_path = match device {
            Some(d) => d,
            None => {
                eprintln!("Usage: microcom [-s speed] /dev/ttyX");
                return Ok(1);
            }
        };

        let file = match OpenOptions::new().read(true).write(true).open(dev_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("microcom: {}: {}", dev_path.to_string_lossy(), e);
                return Ok(1);
            }
        };

        let fd = file.as_raw_fd();
        unsafe {
            let mut tio: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(fd, &mut tio) == 0 {
                libc::cfmakeraw(&mut tio);
                let speed_c = match speed {
                    9600 => libc::B9600,
                    19200 => libc::B19200,
                    38400 => libc::B38400,
                    57600 => libc::B57600,
                    115200 => libc::B115200,
                    _ => libc::B9600,
                };
                libc::cfsetispeed(&mut tio, speed_c);
                libc::cfsetospeed(&mut tio, speed_c);
                libc::tcsetattr(fd, libc::TCSANOW, &tio);
            }
        }

        let mut f_write = match file.try_clone() {
            Ok(f) => f,
            Err(_) => return Ok(1),
        };
        let mut f_read = file;

        std::thread::spawn(move || {
            let stdout = io::stdout();
            let mut out = stdout.lock();
            let mut buf = [0u8; 1024];
            while let Ok(n) = f_read.read(&mut buf) {
                if n == 0 {
                    break;
                }
                let _ = out.write_all(&buf[..n]);
                let _ = out.flush();
            }
        });

        let stdin = io::stdin();
        let mut in_lock = stdin.lock();
        let mut buf = [0u8; 1024];
        while let Ok(n) = in_lock.read(&mut buf) {
            if n == 0 {
                break;
            }
            if f_write.write_all(&buf[..n]).is_err() {
                break;
            }
        }

        Ok(0)
    }
}

pub struct PscanApplet;

impl Applet for PscanApplet {
    fn name(&self) -> &'static str {
        "pscan"
    }
    fn description(&self) -> &'static str {
        "Scan a host's ports"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut start_port = 1u16;
        let mut end_port = 1024u16;
        let mut host = None;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-p" && i + 1 < args.len() {
                i += 1;
                let s = String::from_utf8_lossy(args[i].as_bytes());
                if let Some(dash) = s.find('-') {
                    start_port = s[..dash].parse().unwrap_or(1);
                    end_port = s[dash + 1..].parse().unwrap_or(1024);
                } else if let Ok(p) = s.parse::<u16>() {
                    start_port = p;
                    end_port = p;
                }
            } else if !b.starts_with(b"-") {
                host = Some(&args[i]);
            }
            i += 1;
        }

        let host = match host {
            Some(h) => String::from_utf8_lossy(h.as_bytes()).to_string(),
            None => {
                eprintln!("Usage: pscan [-p start-end] host");
                return Ok(1);
            }
        };

        println!("Scanning {} ports {} to {}", host, start_port, end_port);

        for port in start_port..=end_port {
            let target = format!("{}:{}", host, port);
            if let Ok(addrs) = std::net::ToSocketAddrs::to_socket_addrs(&target) {
                for addr in addrs {
                    if TcpStream::connect_timeout(&addr, Duration::from_millis(150)).is_ok() {
                        println!("{:5} open", port);
                        break;
                    }
                }
            }
        }

        Ok(0)
    }
}

pub struct DnsdomainnameApplet;

impl Applet for DnsdomainnameApplet {
    fn name(&self) -> &'static str {
        "dnsdomainname"
    }
    fn description(&self) -> &'static str {
        "Show DNS domain name"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let mut buf = [0u8; 256];
        let res = unsafe { libc::getdomainname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) };
        if res == 0 {
            let name = unsafe { CStr::from_ptr(buf.as_ptr() as *const libc::c_char) };
            let s = name.to_string_lossy();
            if s != "(none)" {
                println!("{}", s);
                return Ok(0);
            }
        }

        let mut hbuf = [0u8; 256];
        let hres = unsafe { libc::gethostname(hbuf.as_mut_ptr() as *mut libc::c_char, hbuf.len()) };
        if hres == 0 {
            let hname = unsafe { CStr::from_ptr(hbuf.as_ptr() as *const libc::c_char) };
            let hs = hname.to_string_lossy();
            if let Some(idx) = hs.find('.') {
                println!("{}", &hs[idx + 1..]);
                return Ok(0);
            }
        }
        println!();
        Ok(0)
    }
}

pub struct IpcalcApplet;

impl Applet for IpcalcApplet {
    fn name(&self) -> &'static str {
        "ipcalc"
    }
    fn description(&self) -> &'static str {
        "Calculate IP network, broadcast, and netmask"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut opt_net = false;
        let mut opt_bcast = false;
        let mut opt_mask = false;
        let mut opt_prefix = false;
        let mut target = None;
        let mut netmask_arg = None;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-n" || b == b"--network" {
                opt_net = true;
            } else if b == b"-b" || b == b"--broadcast" {
                opt_bcast = true;
            } else if b == b"-m" || b == b"--netmask" {
                opt_mask = true;
            } else if b == b"-p" || b == b"--prefix" {
                opt_prefix = true;
            } else if !b.starts_with(b"-") {
                if target.is_none() {
                    target = Some(&args[i]);
                } else if netmask_arg.is_none() {
                    netmask_arg = Some(&args[i]);
                }
            }
            i += 1;
        }

        let target = match target {
            Some(t) => String::from_utf8_lossy(t.as_bytes()).to_string(),
            None => {
                eprintln!("Usage: ipcalc [options] <address>[/prefix] [netmask]");
                return Ok(1);
            }
        };

        let (ip_str, prefix_len) = if let Some(slash) = target.find('/') {
            let p: u32 = target[slash + 1..].parse().unwrap_or(32);
            (&target[..slash], p)
        } else if let Some(m_arg) = netmask_arg {
            let ms = String::from_utf8_lossy(m_arg.as_bytes()).to_string();
            if let Ok(mask_ip) = ms.parse::<Ipv4Addr>() {
                let u = u32::from(mask_ip);
                (target.as_str(), u.count_ones())
            } else {
                (target.as_str(), ms.parse().unwrap_or(32))
            }
        } else {
            (target.as_str(), 24)
        };

        let ip = match ip_str.parse::<Ipv4Addr>() {
            Ok(ip) => ip,
            Err(e) => {
                eprintln!("ipcalc: invalid IP: {}", e);
                return Ok(1);
            }
        };

        let ip_u = u32::from(ip);
        let mask_u = if prefix_len == 0 {
            0
        } else {
            !0u32 << (32 - prefix_len)
        };
        let net_u = ip_u & mask_u;
        let bcast_u = net_u | !mask_u;

        let net_ip = Ipv4Addr::from(net_u);
        let bcast_ip = Ipv4Addr::from(bcast_u);
        let mask_ip = Ipv4Addr::from(mask_u);

        let any_flag = opt_net || opt_bcast || opt_mask || opt_prefix;

        if !any_flag || opt_net {
            println!("NETWORK={}", net_ip);
        }
        if !any_flag || opt_mask {
            println!("NETMASK={}", mask_ip);
        }
        if !any_flag || opt_bcast {
            println!("BROADCAST={}", bcast_ip);
        }
        if !any_flag || opt_prefix {
            println!("PREFIX={}", prefix_len);
        }

        Ok(0)
    }
}

pub struct WatchdogApplet;

impl Applet for WatchdogApplet {
    fn name(&self) -> &'static str {
        "watchdog"
    }
    fn description(&self) -> &'static str {
        "Software watchdog daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dev = "/dev/watchdog".to_string();
        let mut reset_sec = 30u64;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-t" && i + 1 < args.len() {
                i += 1;
                reset_sec = String::from_utf8_lossy(args[i].as_bytes())
                    .parse()
                    .unwrap_or(30);
            } else if !b.starts_with(b"-") {
                dev = String::from_utf8_lossy(b).to_string();
            }
            i += 1;
        }

        let mut f = match OpenOptions::new().write(true).open(&dev) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("watchdog: {}: {}", dev, e);
                return Ok(1);
            }
        };

        loop {
            if f.write_all(b"\0").is_err() || f.flush().is_err() {
                eprintln!("watchdog: ping failed");
                return Ok(1);
            }
            std::thread::sleep(Duration::from_secs(reset_sec));
        }
    }
}

pub struct ConspyApplet;

impl Applet for ConspyApplet {
    fn name(&self) -> &'static str {
        "conspy"
    }
    fn description(&self) -> &'static str {
        "Spy on Linux virtual consoles"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut console_num = 1usize;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if !b.starts_with(b"-") {
                if let Ok(n) = String::from_utf8_lossy(b).parse::<usize>() {
                    console_num = n;
                }
            }
            i += 1;
        }

        let dev_name = format!("/dev/vcs{}", console_num);
        let mut f = match File::open(&dev_name) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("conspy: {}: {}", dev_name, e);
                return Ok(1);
            }
        };

        print!("\x1b[2J\x1b[H");
        let _ = io::stdout().flush();
        let mut buf = Vec::new();
        let _ = f.read_to_end(&mut buf);
        let _ = io::stdout().write_all(&buf);
        let _ = io::stdout().flush();

        Ok(0)
    }
}

pub struct SetconsoleApplet;

impl Applet for SetconsoleApplet {
    fn name(&self) -> &'static str {
        "setconsole"
    }
    fn description(&self) -> &'static str {
        "Redirect system console output to a device"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut reset = false;
        let mut dev = None;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-r" {
                reset = true;
            } else if !b.starts_with(b"-") {
                dev = Some(&args[i]);
            }
            i += 1;
        }

        let target = if reset {
            "/dev/console"
        } else {
            match dev {
                Some(d) => d.to_str().unwrap_or("/dev/tty"),
                None => "/dev/tty",
            }
        };

        let file = match OpenOptions::new().write(true).open(target) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("setconsole: {}: {}", target, e);
                return Ok(1);
            }
        };

        const TIOCCONS: libc::c_ulong = 0x541D;
        let res = unsafe { libc::ioctl(file.as_raw_fd(), TIOCCONS) };
        if res < 0 {
            let err = io::Error::last_os_error();
            eprintln!("setconsole: ioctl: {}", err);
            return Ok(1);
        }

        Ok(0)
    }
}

pub struct ResizeApplet;

impl Applet for ResizeApplet {
    fn name(&self) -> &'static str {
        "resize"
    }
    fn description(&self) -> &'static str {
        "Determine and set terminal size"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
        let res = unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) };

        let (lines, cols) = if res == 0 && ws.ws_row > 0 && ws.ws_col > 0 {
            (ws.ws_row, ws.ws_col)
        } else {
            print!("\x1b[7\x1b[r\x1b[999;999H\x1b[6n");
            let _ = io::stdout().flush();

            let mut resp = Vec::new();
            let stdin = io::stdin();
            let mut handle = stdin.lock();
            let mut b = [0u8; 1];
            while let Ok(1) = handle.read(&mut b) {
                resp.push(b[0]);
                if b[0] == b'R' {
                    break;
                }
            }
            print!("\x1b[8");
            let _ = io::stdout().flush();

            let mut parsed = (24u16, 80u16);
            if let Ok(s) = std::str::from_utf8(&resp) {
                if let Some(pos) = s.find('[') {
                    let sub = &s[pos + 1..s.len().saturating_sub(1)];
                    let parts: Vec<&str> = sub.split(';').collect();
                    if parts.len() == 2 {
                        let r = parts[0].parse().unwrap_or(24);
                        let c = parts[1].parse().unwrap_or(80);
                        parsed = (r, c);
                    }
                }
            }
            parsed
        };

        println!("COLUMNS={};\nLINES={};\nexport COLUMNS LINES;", cols, lines);
        Ok(0)
    }
}
