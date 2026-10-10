use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

fn percent_decode(s: &[u8]) -> Vec<u8> {
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

pub struct HttpdApplet;

impl Applet for HttpdApplet {
    fn name(&self) -> &'static str {
        "httpd"
    }
    fn description(&self) -> &'static str {
        "Minimal HTTP 1.0/1.1 static file web server"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut port = 80u16;
        let mut home = PathBuf::from(".");
        let mut inetd_mode = false;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-p" && i + 1 < args.len() {
                i += 1;
                let s = String::from_utf8_lossy(args[i].as_bytes());
                if let Ok(p) = s.parse::<u16>() {
                    port = p;
                }
            } else if b == b"-h" && i + 1 < args.len() {
                i += 1;
                home = PathBuf::from(&args[i]);
            } else if b == b"-i" {
                inetd_mode = true;
            } else if b == b"-f" {
            }
            i += 1;
        }

        fn handle_http(mut reader: impl BufRead, mut writer: impl Write, home: &Path) {
            let mut line = String::new();
            if reader.read_line(&mut line).is_err() || line.is_empty() {
                return;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                return;
            }
            let method = parts[0];
            let raw_path = if parts.len() > 1 { parts[1] } else { "/" };
            let decoded_bytes = percent_decode(raw_path.as_bytes());
            let path_str = String::from_utf8_lossy(&decoded_bytes);

            let clean_path = path_str.split('?').next().unwrap_or("/");
            let mut rel_path = clean_path.trim_start_matches('/');
            if rel_path.is_empty() {
                rel_path = "index.html";
            }

            let mut safe = true;
            for seg in rel_path.split('/') {
                if seg == ".." {
                    safe = false;
                    break;
                }
            }

            let file_path = home.join(rel_path);

            if !safe || (method != "GET" && method != "HEAD") {
                let _ = writer.write_all(b"HTTP/1.0 400 Bad Request\r\nContent-Length: 0\r\n\r\n");
                return;
            }

            let mut target = file_path.clone();
            if target.is_dir() {
                target = target.join("index.html");
            }

            match File::open(&target) {
                Ok(mut f) => {
                    let len = f.metadata().map(|m| m.len()).unwrap_or(0);
                    let ct = if target
                        .extension()
                        .map(|e| e == "html" || e == "htm")
                        .unwrap_or(false)
                    {
                        "text/html"
                    } else if target.extension().map(|e| e == "txt").unwrap_or(false) {
                        "text/plain"
                    } else {
                        "application/octet-stream"
                    };
                    let header = format!(
                        "HTTP/1.0 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        ct, len
                    );
                    let _ = writer.write_all(header.as_bytes());
                    if method == "GET" {
                        let mut buf = [0u8; 8192];
                        while let Ok(n) = f.read(&mut buf) {
                            if n == 0 {
                                break;
                            }
                            if writer.write_all(&buf[..n]).is_err() {
                                break;
                            }
                        }
                    }
                }
                Err(_) => {
                    let _ =
                        writer.write_all(b"HTTP/1.0 404 Not Found\r\nContent-Length: 0\r\n\r\n");
                }
            }
        }

        if inetd_mode {
            let stdin = io::stdin();
            let stdout = io::stdout();
            handle_http(stdin.lock(), stdout.lock(), &home);
            return Ok(0);
        }

        let listener = match TcpListener::bind(("0.0.0.0", port)) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("httpd: cannot bind port {}: {}", port, e);
                return Ok(1);
            }
        };

        for stream in listener.incoming().flatten() {
            let reader = BufReader::new(match stream.try_clone() {
                Ok(s) => s,
                Err(_) => continue,
            });
            handle_http(reader, stream, &home);
        }
        Ok(0)
    }
}

pub struct FtpdApplet;

impl Applet for FtpdApplet {
    fn name(&self) -> &'static str {
        "ftpd"
    }
    fn description(&self) -> &'static str {
        "FTP server daemon (runs from inetd or standalone with -S)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dir = PathBuf::from(".");
        let mut port = 21u16;
        let mut standalone = false;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-S" {
                standalone = true;
            } else if b == b"-p" && i + 1 < args.len() {
                i += 1;
                let s = String::from_utf8_lossy(args[i].as_bytes());
                if let Ok(p) = s.parse::<u16>() {
                    port = p;
                }
            } else if !b.starts_with(b"-") {
                dir = PathBuf::from(&args[i]);
            }
            i += 1;
        }

        fn run_session(mut reader: impl BufRead, mut writer: impl Write, root: &Path) {
            let _ = writer.write_all(b"220 Service ready for new user.\r\n");
            let _ = writer.flush();

            let mut pasv_listener: Option<TcpListener> = None;
            let mut line = String::new();

            loop {
                line.clear();
                if reader.read_line(&mut line).is_err() || line.is_empty() {
                    break;
                }
                let trimmed = line.trim();
                let mut parts = trimmed.splitn(2, ' ');
                let cmd = parts.next().unwrap_or("").to_ascii_uppercase();
                let arg = parts.next().unwrap_or("").trim();

                match cmd.as_str() {
                    "USER" => {
                        let _ = writer.write_all(b"331 User name okay, need password.\r\n");
                    }
                    "PASS" => {
                        let _ = writer.write_all(b"230 User logged in, proceed.\r\n");
                    }
                    "SYST" => {
                        let _ = writer.write_all(b"215 UNIX Type: L8\r\n");
                    }
                    "FEAT" => {
                        let _ = writer.write_all(b"211 End\r\n");
                    }
                    "PWD" => {
                        let _ = writer.write_all(b"257 \"/\"\r\n");
                    }
                    "TYPE" => {
                        let _ = writer.write_all(b"200 Command okay.\r\n");
                    }
                    "PASV" => match TcpListener::bind(("0.0.0.0", 0)) {
                        Ok(l) => {
                            if let Ok(addr) = l.local_addr() {
                                let p = addr.port();
                                let p1 = p / 256;
                                let p2 = p % 256;
                                let resp = format!(
                                    "227 Entering Passive Mode (127,0,0,1,{},{}).\r\n",
                                    p1, p2
                                );
                                let _ = writer.write_all(resp.as_bytes());
                                pasv_listener = Some(l);
                            } else {
                                let _ = writer.write_all(b"425 Can't open data connection.\r\n");
                            }
                        }
                        Err(_) => {
                            let _ = writer.write_all(b"425 Can't open data connection.\r\n");
                        }
                    },
                    "LIST" => {
                        let _ = writer
                            .write_all(b"150 File status okay; about to open data connection.\r\n");
                        let _ = writer.flush();
                        if let Some(l) = pasv_listener.take() {
                            if let Ok((mut data_stream, _)) = l.accept() {
                                if let Ok(entries) = fs::read_dir(root) {
                                    for entry in entries.flatten() {
                                        let name = entry.file_name();
                                        let line = format!(
                                            "-rw-r--r-- 1 ftp ftp 1024 Jan 01 00:00 {}\r\n",
                                            name.to_string_lossy()
                                        );
                                        let _ = data_stream.write_all(line.as_bytes());
                                    }
                                }
                            }
                        }
                        let _ = writer.write_all(b"226 Closing data connection.\r\n");
                    }
                    "RETR" => {
                        let target = root.join(arg.trim_start_matches('/'));
                        if let Ok(mut f) = File::open(&target) {
                            let _ =
                                writer.write_all(b"150 Opening binary mode data connection\r\n");
                            let _ = writer.flush();
                            if let Some(l) = pasv_listener.take() {
                                if let Ok((mut data_stream, _)) = l.accept() {
                                    let mut buf = [0u8; 8192];
                                    while let Ok(n) = f.read(&mut buf) {
                                        if n == 0 {
                                            break;
                                        }
                                        if data_stream.write_all(&buf[..n]).is_err() {
                                            break;
                                        }
                                    }
                                }
                            }
                            let _ = writer.write_all(b"226 Transfer complete.\r\n");
                        } else {
                            let _ = writer.write_all(b"550 Failed to open file.\r\n");
                        }
                    }
                    "STOR" => {
                        let target = root.join(arg.trim_start_matches('/'));
                        if let Ok(mut f) = File::create(&target) {
                            let _ =
                                writer.write_all(b"150 Opening binary mode data connection\r\n");
                            let _ = writer.flush();
                            if let Some(l) = pasv_listener.take() {
                                if let Ok((mut data_stream, _)) = l.accept() {
                                    let mut buf = [0u8; 8192];
                                    while let Ok(n) = data_stream.read(&mut buf) {
                                        if n == 0 {
                                            break;
                                        }
                                        if f.write_all(&buf[..n]).is_err() {
                                            break;
                                        }
                                    }
                                }
                            }
                            let _ = writer.write_all(b"226 Transfer complete.\r\n");
                        } else {
                            let _ = writer.write_all(b"550 Failed to create file.\r\n");
                        }
                    }
                    "QUIT" => {
                        let _ = writer.write_all(b"221 Goodbye.\r\n");
                        let _ = writer.flush();
                        break;
                    }
                    _ => {
                        let _ = writer.write_all(b"502 Command not implemented.\r\n");
                    }
                }
                let _ = writer.flush();
            }
        }

        if !standalone {
            let stdin = io::stdin();
            let stdout = io::stdout();
            run_session(stdin.lock(), stdout.lock(), &dir);
            return Ok(0);
        }

        let listener = match TcpListener::bind(("0.0.0.0", port)) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("ftpd: bind: {}", e);
                return Ok(1);
            }
        };

        for stream in listener.incoming().flatten() {
            let reader = BufReader::new(match stream.try_clone() {
                Ok(s) => s,
                Err(_) => continue,
            });
            run_session(reader, stream, &dir);
        }

        Ok(0)
    }
}

fn ftp_send_cmd(
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

fn ftp_enter_pasv(
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

pub struct FtpgetApplet;

impl Applet for FtpgetApplet {
    fn name(&self) -> &'static str {
        "ftpget"
    }
    fn description(&self) -> &'static str {
        "Retrieve file from FTP server"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut port = 21u16;
        let mut user = "anonymous".to_string();
        let mut pass = "busybox@".to_string();
        let mut pos_args = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-p" && i + 1 < args.len() {
                i += 1;
                port = String::from_utf8_lossy(args[i].as_bytes())
                    .parse()
                    .unwrap_or(21);
            } else if b == b"-u" && i + 1 < args.len() {
                i += 1;
                user = String::from_utf8_lossy(args[i].as_bytes()).to_string();
            } else if b == b"-p" && i + 1 < args.len() {
                i += 1;
                pass = String::from_utf8_lossy(args[i].as_bytes()).to_string();
            } else if !b.starts_with(b"-") {
                pos_args.push(&args[i]);
            }
            i += 1;
        }

        if pos_args.len() < 3 {
            eprintln!("Usage: ftpget [options] host local-file remote-file");
            return Ok(1);
        }

        let host = String::from_utf8_lossy(pos_args[0].as_bytes());
        let local_file = Path::new(pos_args[1]);
        let remote_file = String::from_utf8_lossy(pos_args[2].as_bytes());

        let target_addr = format!("{}:{}", host, port);
        let stream = match TcpStream::connect(&target_addr) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("ftpget: connect: {}", e);
                return Ok(1);
            }
        };

        let mut writer = match stream.try_clone() {
            Ok(w) => w,
            Err(e) => {
                eprintln!("ftpget: {}", e);
                return Ok(1);
            }
        };
        let mut reader = BufReader::new(stream);

        if let Err(e) = (|| -> io::Result<()> {
            ftp_send_cmd(&mut reader, &mut writer, "")?;
            ftp_send_cmd(&mut reader, &mut writer, &format!("USER {}", user))?;
            ftp_send_cmd(&mut reader, &mut writer, &format!("PASS {}", pass))?;
            ftp_send_cmd(&mut reader, &mut writer, "TYPE I")?;
            let mut data_stream = ftp_enter_pasv(&mut reader, &mut writer)?;
            ftp_send_cmd(&mut reader, &mut writer, &format!("RETR {}", remote_file))?;

            let mut out: Box<dyn Write> = if local_file.as_os_str() == "-" {
                Box::new(io::stdout())
            } else {
                Box::new(File::create(local_file)?)
            };

            let mut buf = [0u8; 8192];
            while let Ok(n) = data_stream.read(&mut buf) {
                if n == 0 {
                    break;
                }
                out.write_all(&buf[..n])?;
            }
            out.flush()?;
            let _ = ftp_send_cmd(&mut reader, &mut writer, "QUIT");
            Ok(())
        })() {
            eprintln!("ftpget: error: {}", e);
            return Ok(1);
        }

        Ok(0)
    }
}

pub struct FtpputApplet;

impl Applet for FtpputApplet {
    fn name(&self) -> &'static str {
        "ftpput"
    }
    fn description(&self) -> &'static str {
        "Upload file to FTP server"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut port = 21u16;
        let mut user = "anonymous".to_string();
        let mut pass = "busybox@".to_string();
        let mut pos_args = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-p" && i + 1 < args.len() {
                i += 1;
                port = String::from_utf8_lossy(args[i].as_bytes())
                    .parse()
                    .unwrap_or(21);
            } else if b == b"-u" && i + 1 < args.len() {
                i += 1;
                user = String::from_utf8_lossy(args[i].as_bytes()).to_string();
            } else if b == b"-p" && i + 1 < args.len() {
                i += 1;
                pass = String::from_utf8_lossy(args[i].as_bytes()).to_string();
            } else if !b.starts_with(b"-") {
                pos_args.push(&args[i]);
            }
            i += 1;
        }

        if pos_args.len() < 3 {
            eprintln!("Usage: ftpput [options] host remote-file local-file");
            return Ok(1);
        }

        let host = String::from_utf8_lossy(pos_args[0].as_bytes());
        let remote_file = String::from_utf8_lossy(pos_args[1].as_bytes());
        let local_file = Path::new(pos_args[2]);

        let target_addr = format!("{}:{}", host, port);
        let stream = match TcpStream::connect(&target_addr) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("ftpput: connect: {}", e);
                return Ok(1);
            }
        };

        let mut writer = match stream.try_clone() {
            Ok(w) => w,
            Err(e) => {
                eprintln!("ftpput: {}", e);
                return Ok(1);
            }
        };
        let mut reader = BufReader::new(stream);

        if let Err(e) = (|| -> io::Result<()> {
            ftp_send_cmd(&mut reader, &mut writer, "")?;
            ftp_send_cmd(&mut reader, &mut writer, &format!("USER {}", user))?;
            ftp_send_cmd(&mut reader, &mut writer, &format!("PASS {}", pass))?;
            ftp_send_cmd(&mut reader, &mut writer, "TYPE I")?;
            let mut data_stream = ftp_enter_pasv(&mut reader, &mut writer)?;
            ftp_send_cmd(&mut reader, &mut writer, &format!("STOR {}", remote_file))?;

            let mut in_file: Box<dyn Read> = if local_file.as_os_str() == "-" {
                Box::new(io::stdin())
            } else {
                Box::new(File::open(local_file)?)
            };

            let mut buf = [0u8; 8192];
            while let Ok(n) = in_file.read(&mut buf) {
                if n == 0 {
                    break;
                }
                data_stream.write_all(&buf[..n])?;
            }
            data_stream.flush()?;
            drop(data_stream);
            let _ = ftp_send_cmd(&mut reader, &mut writer, "QUIT");
            Ok(())
        })() {
            eprintln!("ftpput: error: {}", e);
            return Ok(1);
        }

        Ok(0)
    }
}

pub struct TftpApplet;

impl Applet for TftpApplet {
    fn name(&self) -> &'static str {
        "tftp"
    }
    fn description(&self) -> &'static str {
        "Transfer file to/from TFTP server"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut is_get = true;
        let mut remote_file = None;
        let mut local_file = None;
        let mut host = None;
        let mut port = 69u16;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-g" {
                is_get = true;
            } else if b == b"-p" {
                is_get = false;
            } else if b == b"-r" && i + 1 < args.len() {
                i += 1;
                remote_file = Some(&args[i]);
            } else if b == b"-l" && i + 1 < args.len() {
                i += 1;
                local_file = Some(&args[i]);
            } else if !b.starts_with(b"-") && host.is_none() {
                host = Some(&args[i]);
            } else if !b.starts_with(b"-") {
                if let Ok(p) = String::from_utf8_lossy(b).parse::<u16>() {
                    port = p;
                }
            }
            i += 1;
        }

        let host = match host {
            Some(h) => String::from_utf8_lossy(h.as_bytes()).to_string(),
            None => {
                eprintln!("tftp: missing host");
                return Ok(1);
            }
        };

        let rem = match remote_file {
            Some(r) => String::from_utf8_lossy(r.as_bytes()).to_string(),
            None => match local_file {
                Some(l) => String::from_utf8_lossy(l.as_bytes()).to_string(),
                None => {
                    eprintln!("tftp: missing file");
                    return Ok(1);
                }
            },
        };

        let loc = local_file.unwrap_or(remote_file.unwrap());

        let sock = match UdpSocket::bind("0.0.0.0:0") {
            Ok(s) => s,
            Err(e) => {
                eprintln!("tftp: bind: {}", e);
                return Ok(1);
            }
        };
        let _ = sock.set_read_timeout(Some(Duration::from_secs(5)));

        let server_addr: SocketAddr = match format!("{}:{}", host, port).parse() {
            Ok(a) => a,
            Err(_) => {
                eprintln!("tftp: invalid server address");
                return Ok(1);
            }
        };

        if is_get {
            let mut req = vec![0, 1];
            req.extend_from_slice(rem.as_bytes());
            req.push(0);
            req.extend_from_slice(b"octet\0");
            if sock.send_to(&req, server_addr).is_err() {
                eprintln!("tftp: send error");
                return Ok(1);
            }

            let mut out = match File::create(loc) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("tftp: local file: {}", e);
                    return Ok(1);
                }
            };

            let mut expected_block = 1u16;
            let mut buf = [0u8; 1024];

            loop {
                match sock.recv_from(&mut buf) {
                    Ok((len, from)) => {
                        if len < 4 {
                            break;
                        }
                        let opcode = u16::from_be_bytes([buf[0], buf[1]]);
                        if opcode == 5 {
                            eprintln!("tftp: server returned error");
                            return Ok(1);
                        }
                        if opcode == 3 {
                            let block = u16::from_be_bytes([buf[2], buf[3]]);
                            if block == expected_block {
                                let _ = out.write_all(&buf[4..len]);

                                let ack = [0, 4, buf[2], buf[3]];
                                let _ = sock.send_to(&ack, from);
                                expected_block = expected_block.wrapping_add(1);
                                if len - 4 < 512 {
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("tftp: recv timeout: {}", e);
                        return Ok(1);
                    }
                }
            }
        } else {
            let mut req = vec![0, 2];
            req.extend_from_slice(rem.as_bytes());
            req.push(0);
            req.extend_from_slice(b"octet\0");
            if sock.send_to(&req, server_addr).is_err() {
                eprintln!("tftp: send error");
                return Ok(1);
            }

            let mut f = match File::open(loc) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("tftp: open local file: {}", e);
                    return Ok(1);
                }
            };

            let mut block = 0u16;
            let mut buf = [0u8; 516];

            let mut ack_buf = [0u8; 16];
            let remote = match sock.recv_from(&mut ack_buf) {
                Ok((len, from)) => {
                    if len < 4 || u16::from_be_bytes([ack_buf[0], ack_buf[1]]) != 4 {
                        eprintln!("tftp: WRQ rejected");
                        return Ok(1);
                    }
                    from
                }
                Err(e) => {
                    eprintln!("tftp: timeout waiting for WRQ ack: {}", e);
                    return Ok(1);
                }
            };

            loop {
                block = block.wrapping_add(1);
                buf[0] = 0;
                buf[1] = 3;
                buf[2..4].copy_from_slice(&block.to_be_bytes());
                let n = f.read(&mut buf[4..]).unwrap_or_default();
                let pkt_len = 4 + n;
                if sock.send_to(&buf[..pkt_len], remote).is_err() {
                    eprintln!("tftp: send data error");
                    return Ok(1);
                }

                match sock.recv_from(&mut ack_buf) {
                    Ok((len, _)) => {
                        if len >= 4 {
                            let ack_op = u16::from_be_bytes([ack_buf[0], ack_buf[1]]);
                            let ack_blk = u16::from_be_bytes([ack_buf[2], ack_buf[3]]);
                            if ack_op == 4 && ack_blk == block {
                                if n < 512 {
                                    break;
                                }
                                continue;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("tftp: ACK timeout: {}", e);
                        return Ok(1);
                    }
                }
            }
        }

        Ok(0)
    }
}

pub struct TftpdApplet;

impl Applet for TftpdApplet {
    fn name(&self) -> &'static str {
        "tftpd"
    }
    fn description(&self) -> &'static str {
        "TFTP server daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dir = PathBuf::from(".");
        let mut port = 69u16;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-p" && i + 1 < args.len() {
                i += 1;
                port = String::from_utf8_lossy(args[i].as_bytes())
                    .parse()
                    .unwrap_or(69);
            } else if !b.starts_with(b"-") {
                dir = PathBuf::from(&args[i]);
            }
            i += 1;
        }

        let sock = match UdpSocket::bind(("0.0.0.0", port)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("tftpd: bind: {}", e);
                return Ok(1);
            }
        };

        let mut buf = [0u8; 1024];
        while let Ok((len, client)) = sock.recv_from(&mut buf) {
            if len < 4 {
                continue;
            }
            let opcode = u16::from_be_bytes([buf[0], buf[1]]);
            if opcode == 1 {
                let slice = &buf[2..len];
                if let Some(pos) = slice.iter().position(|&b| b == 0) {
                    let filename = String::from_utf8_lossy(&slice[..pos]);
                    let filepath = dir.join(filename.trim_start_matches('/'));
                    if let Ok(mut f) = File::open(filepath) {
                        let mut block = 1u16;
                        let mut data_buf = [0u8; 516];
                        loop {
                            data_buf[0] = 0;
                            data_buf[1] = 3;
                            data_buf[2..4].copy_from_slice(&block.to_be_bytes());
                            let n = f.read(&mut data_buf[4..]).unwrap_or(0);
                            let _ = sock.send_to(&data_buf[..4 + n], client);

                            let mut ack = [0u8; 16];
                            let _ = sock.set_read_timeout(Some(Duration::from_secs(2)));
                            let _ = sock.recv_from(&mut ack);
                            if n < 512 {
                                break;
                            }
                            block = block.wrapping_add(1);
                        }
                    } else {
                        let err_pkt = [
                            0, 5, 0, 1, b'F', b'i', b'l', b'e', b' ', b'n', b'o', b't', b' ', b'f',
                            b'o', b'u', b'n', b'd', 0,
                        ];
                        let _ = sock.send_to(&err_pkt, client);
                    }
                }
            }
        }
        Ok(0)
    }
}

pub struct TelnetApplet;

impl Applet for TelnetApplet {
    fn name(&self) -> &'static str {
        "telnet"
    }
    fn description(&self) -> &'static str {
        "Connect to TELNET server"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut host = None;
        let mut port = 23u16;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if !b.starts_with(b"-") {
                if host.is_none() {
                    host = Some(&args[i]);
                } else if let Ok(p) = String::from_utf8_lossy(b).parse::<u16>() {
                    port = p;
                }
            }
            i += 1;
        }

        let host = match host {
            Some(h) => String::from_utf8_lossy(h.as_bytes()).to_string(),
            None => {
                eprintln!("Usage: telnet host [port]");
                return Ok(1);
            }
        };

        let stream = match TcpStream::connect((host.as_str(), port)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("telnet: connect: {}", e);
                return Ok(1);
            }
        };

        let mut stream_write = match stream.try_clone() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("telnet: {}", e);
                return Ok(1);
            }
        };
        let mut stream_read = stream;

        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            let stdout = io::stdout();
            let mut out = stdout.lock();
            while let Ok(n) = stream_read.read(&mut buf) {
                if n == 0 {
                    break;
                }

                let mut filtered = Vec::with_capacity(n);
                let mut idx = 0;
                while idx < n {
                    if buf[idx] == 255 && idx + 2 < n {
                        idx += 3;
                        continue;
                    }
                    filtered.push(buf[idx]);
                    idx += 1;
                }
                let _ = out.write_all(&filtered);
                let _ = out.flush();
            }
            std::process::exit(0);
        });

        let stdin = io::stdin();
        let mut in_lock = stdin.lock();
        let mut buf = [0u8; 4096];
        while let Ok(n) = in_lock.read(&mut buf) {
            if n == 0 {
                break;
            }
            if stream_write.write_all(&buf[..n]).is_err() {
                break;
            }
        }

        Ok(0)
    }
}

pub struct TelnetdApplet;

impl Applet for TelnetdApplet {
    fn name(&self) -> &'static str {
        "telnetd"
    }
    fn description(&self) -> &'static str {
        "TELNET server daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut port = 23u16;
        let mut login_prog = "/bin/sh".to_string();
        let mut inetd_mode = false;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-p" && i + 1 < args.len() {
                i += 1;
                port = String::from_utf8_lossy(args[i].as_bytes())
                    .parse()
                    .unwrap_or(23);
            } else if b == b"-l" && i + 1 < args.len() {
                i += 1;
                login_prog = String::from_utf8_lossy(args[i].as_bytes()).to_string();
            } else if b == b"-i" {
                inetd_mode = true;
            }
            i += 1;
        }

        fn run_session(stream: TcpStream, prog: &str) {
            let prog_str = prog.to_string();
            std::thread::spawn(move || {
                let mut child = match Command::new(&prog_str)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                {
                    Ok(c) => c,
                    Err(_) => return,
                };

                let mut child_in = child.stdin.take().unwrap();
                let mut child_out = child.stdout.take().unwrap();
                let mut s_read = match stream.try_clone() {
                    Ok(s) => s,
                    Err(_) => return,
                };
                let mut s_write = stream;

                std::thread::spawn(move || {
                    let mut b = [0u8; 1024];
                    while let Ok(n) = child_out.read(&mut b) {
                        if n == 0 || s_write.write_all(&b[..n]).is_err() {
                            break;
                        }
                    }
                });

                let mut b = [0u8; 1024];
                while let Ok(n) = s_read.read(&mut b) {
                    if n == 0 || child_in.write_all(&b[..n]).is_err() {
                        break;
                    }
                }
                let _ = child.wait();
            });
        }

        if inetd_mode {
            let _ = Command::new(&login_prog).spawn().and_then(|mut c| c.wait());
            return Ok(0);
        }

        let listener = match TcpListener::bind(("0.0.0.0", port)) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("telnetd: bind: {}", e);
                return Ok(1);
            }
        };

        for stream in listener.incoming().flatten() {
            run_session(stream, &login_prog);
        }

        Ok(0)
    }
}

pub struct InetdApplet;

impl Applet for InetdApplet {
    fn name(&self) -> &'static str {
        "inetd"
    }
    fn description(&self) -> &'static str {
        "Internet super-server daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let conf_file = if !args.is_empty() && !args[0].as_bytes().starts_with(b"-") {
            args[0].as_os_str()
        } else {
            Path::new("/etc/inetd.conf").as_os_str()
        };

        let file = match File::open(conf_file) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("inetd: {}: {}", conf_file.to_string_lossy(), e);
                return Ok(1);
            }
        };

        let reader = BufReader::new(file);
        for line in reader.lines().map_while(std::result::Result::ok) {
            let l = line.trim();
            if l.is_empty() || l.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = l.split_whitespace().collect();

            if parts.len() < 6 {
                continue;
            }
            let service = parts[0];
            let proto = parts[2];
            let prog = parts[5].to_string();
            let prog_args: Vec<String> = if parts.len() > 6 {
                parts[6..].iter().map(|s| s.to_string()).collect()
            } else {
                vec![prog.clone()]
            };

            let port: u16 = service.parse().unwrap_or(0);
            if proto == "tcp" && port > 0 {
                let prog_clone = prog.clone();
                let prog_args_clone = prog_args.clone();
                std::thread::spawn(move || {
                    if let Ok(listener) = TcpListener::bind(("0.0.0.0", port)) {
                        for stream in listener.incoming().flatten() {
                            let fd = stream.as_raw_fd();
                            let stdin_fd = unsafe { libc::dup(fd) };
                            let stdout_fd = unsafe { libc::dup(fd) };
                            if stdin_fd >= 0 && stdout_fd >= 0 {
                                let in_file = unsafe { File::from_raw_fd(stdin_fd) };
                                let out_file = unsafe { File::from_raw_fd(stdout_fd) };
                                let _ = Command::new(&prog_clone)
                                    .args(&prog_args_clone[1..])
                                    .stdin(Stdio::from(in_file))
                                    .stdout(Stdio::from(out_file))
                                    .spawn();
                            }
                        }
                    }
                });
            }
        }

        loop {
            std::thread::sleep(Duration::from_secs(3600));
        }
    }
}

pub struct FakeidentdApplet;

impl Applet for FakeidentdApplet {
    fn name(&self) -> &'static str {
        "fakeidentd"
    }
    fn description(&self) -> &'static str {
        "Fake identd daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut ident_user = "nobody".to_string();
        let mut port = 113u16;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-p" && i + 1 < args.len() {
                i += 1;
                port = String::from_utf8_lossy(args[i].as_bytes())
                    .parse()
                    .unwrap_or(113);
            } else if !b.starts_with(b"-") {
                ident_user = String::from_utf8_lossy(args[i].as_bytes()).to_string();
            }
            i += 1;
        }

        let listener = match TcpListener::bind(("0.0.0.0", port)) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("fakeidentd: bind: {}", e);
                return Ok(1);
            }
        };

        for mut stream in listener.incoming().flatten() {
            let user = ident_user.clone();
            std::thread::spawn(move || {
                let mut reader = BufReader::new(match stream.try_clone() {
                    Ok(s) => s,
                    Err(_) => return,
                });
                let mut line = String::new();
                if reader.read_line(&mut line).is_ok() {
                    let parts: Vec<&str> = line.trim().split(',').collect();
                    if parts.len() == 2 {
                        let resp = format!(
                            "{}, {} : USERID : UNIX : {}\r\n",
                            parts[0].trim(),
                            parts[1].trim(),
                            user
                        );
                        let _ = stream.write_all(resp.as_bytes());
                    }
                }
            });
        }
        Ok(0)
    }
}

pub struct DnsdApplet;

impl Applet for DnsdApplet {
    fn name(&self) -> &'static str {
        "dnsd"
    }
    fn description(&self) -> &'static str {
        "Small static DNS server daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut port = 53u16;
        let mut conf_file = "/etc/hosts".to_string();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-p" && i + 1 < args.len() {
                i += 1;
                port = String::from_utf8_lossy(args[i].as_bytes())
                    .parse()
                    .unwrap_or(53);
            } else if b == b"-c" && i + 1 < args.len() {
                i += 1;
                conf_file = String::from_utf8_lossy(args[i].as_bytes()).to_string();
            }
            i += 1;
        }

        let sock = match UdpSocket::bind(("0.0.0.0", port)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("dnsd: bind: {}", e);
                return Ok(1);
            }
        };

        let mut buf = [0u8; 1024];
        while let Ok((len, src)) = sock.recv_from(&mut buf) {
            if len < 12 {
                continue;
            }

            let mut qname = String::new();
            let mut pos = 12;
            while pos < len && buf[pos] != 0 {
                let lab_len = buf[pos] as usize;
                pos += 1;
                if pos + lab_len <= len {
                    if !qname.is_empty() {
                        qname.push('.');
                    }
                    qname.push_str(&String::from_utf8_lossy(&buf[pos..pos + lab_len]));
                    pos += lab_len;
                } else {
                    break;
                }
            }
            if pos >= len || buf[pos] != 0 {
                continue;
            }
            pos += 1;
            if pos + 4 > len {
                continue;
            }
            let qtype = u16::from_be_bytes([buf[pos], buf[pos + 1]]);
            let _qclass = u16::from_be_bytes([buf[pos + 2], buf[pos + 3]]);
            pos += 4;

            let mut resolved_ip: Option<Ipv4Addr> = None;
            if qtype == 1 {
                if let Ok(content) = fs::read_to_string(&conf_file) {
                    for line in content.lines() {
                        let l = line.trim();
                        if l.is_empty() || l.starts_with('#') {
                            continue;
                        }
                        let items: Vec<&str> = l.split_whitespace().collect();
                        if items.len() >= 2 {
                            if let Ok(ip) = items[0].parse::<Ipv4Addr>() {
                                for name in &items[1..] {
                                    if name.eq_ignore_ascii_case(&qname) {
                                        resolved_ip = Some(ip);
                                        break;
                                    }
                                }
                            }
                        }
                        if resolved_ip.is_some() {
                            break;
                        }
                    }
                }
            }

            let mut resp = Vec::with_capacity(512);
            resp.extend_from_slice(&buf[0..2]);

            let flags: u16 = if resolved_ip.is_some() {
                0x8180
            } else {
                0x8183
            };
            resp.extend_from_slice(&flags.to_be_bytes());
            resp.extend_from_slice(&1u16.to_be_bytes());
            let ancount: u16 = if resolved_ip.is_some() { 1 } else { 0 };
            resp.extend_from_slice(&ancount.to_be_bytes());
            resp.extend_from_slice(&0u16.to_be_bytes());
            resp.extend_from_slice(&0u16.to_be_bytes());

            resp.extend_from_slice(&buf[12..pos]);

            if let Some(ip) = resolved_ip {
                resp.extend_from_slice(&[0xC0, 0x0C]);
                resp.extend_from_slice(&1u16.to_be_bytes());
                resp.extend_from_slice(&1u16.to_be_bytes());
                resp.extend_from_slice(&300u32.to_be_bytes());
                resp.extend_from_slice(&4u16.to_be_bytes());
                resp.extend_from_slice(&ip.octets());
            }

            let _ = sock.send_to(&resp, src);
        }

        Ok(0)
    }
}

pub struct DhcprelayApplet;

impl Applet for DhcprelayApplet {
    fn name(&self) -> &'static str {
        "dhcprelay"
    }
    fn description(&self) -> &'static str {
        "DHCP relay agent"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut server_ip: Option<Ipv4Addr> = None;
        let mut client_port = 67u16;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if !b.starts_with(b"-") {
                if let Ok(ip) = String::from_utf8_lossy(b).parse::<Ipv4Addr>() {
                    server_ip = Some(ip);
                }
            } else if b == b"-p" && i + 1 < args.len() {
                i += 1;
                client_port = String::from_utf8_lossy(args[i].as_bytes())
                    .parse()
                    .unwrap_or(67);
            }
            i += 1;
        }

        let server_ip = match server_ip {
            Some(ip) => ip,
            None => {
                eprintln!("Usage: dhcprelay [options] server_ip");
                return Ok(1);
            }
        };

        let sock = match UdpSocket::bind(("0.0.0.0", client_port)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("dhcprelay: bind: {}", e);
                return Ok(1);
            }
        };
        let _ = sock.set_broadcast(true);

        let mut buf = [0u8; 1500];
        while let Ok((len, from)) = sock.recv_from(&mut buf) {
            if len < 240 {
                continue;
            }

            if from.ip() == IpAddr::V4(server_ip) {
                let _ = sock.send_to(&buf[..len], ("255.255.255.255", 68));
            } else {
                let _ = sock.send_to(&buf[..len], (server_ip, 67));
            }
        }
        Ok(0)
    }
}

pub struct UdhcpcApplet;

impl Applet for UdhcpcApplet {
    fn name(&self) -> &'static str {
        "udhcpc"
    }
    fn description(&self) -> &'static str {
        "DHCP client daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut _iface = "eth0".to_string();
        let mut now = false;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-i" && i + 1 < args.len() {
                i += 1;
                _iface = String::from_utf8_lossy(args[i].as_bytes()).to_string();
            } else if b == b"-n" || b == b"-q" {
                now = true;
            }
            i += 1;
        }

        let sock = match UdpSocket::bind("0.0.0.0:68") {
            Ok(s) => s,
            Err(e) => {
                eprintln!("udhcpc: cannot bind to port 68: {}", e);
                return Ok(1);
            }
        };
        let _ = sock.set_broadcast(true);
        let _ = sock.set_read_timeout(Some(Duration::from_secs(3)));

        let mut packet = vec![0u8; 300];
        packet[0] = 1;
        packet[1] = 1;
        packet[2] = 6;
        let xid = 0x12345678u32.to_be_bytes();
        packet[4..8].copy_from_slice(&xid);
        packet[10] = 0x80;

        packet[236..240].copy_from_slice(&[99, 130, 83, 99]);

        packet[240] = 53;
        packet[241] = 1;
        packet[242] = 1;

        packet[243] = 255;

        let _ = sock.send_to(&packet, ("255.255.255.255", 67));

        let mut in_buf = [0u8; 1500];
        let mut attempts = 0;
        loop {
            match sock.recv_from(&mut in_buf) {
                Ok((len, _)) => {
                    if len >= 240 && in_buf[0] == 2 && in_buf[4..8] == xid {
                        let yiaddr = Ipv4Addr::new(in_buf[16], in_buf[17], in_buf[18], in_buf[19]);
                        println!("udhcpc: obtained lease for {}", yiaddr);
                        return Ok(0);
                    }
                }
                Err(_) => {
                    attempts += 1;
                    if attempts >= 3 {
                        if now {
                            eprintln!("udhcpc: no lease, failing");
                            return Ok(1);
                        }

                        let _ = sock.send_to(&packet, ("255.255.255.255", 67));
                        attempts = 0;
                    }
                }
            }
        }
    }
}

pub struct Udhcpc6Applet;

impl Applet for Udhcpc6Applet {
    fn name(&self) -> &'static str {
        "udhcpc6"
    }
    fn description(&self) -> &'static str {
        "DHCPv6 client daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut iface = "eth0".to_string();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-i" && i + 1 < args.len() {
                i += 1;
                iface = String::from_utf8_lossy(args[i].as_bytes()).to_string();
            }
            i += 1;
        }

        let sock = match UdpSocket::bind("[::]:546") {
            Ok(s) => s,
            Err(e) => {
                eprintln!("udhcpc6: cannot bind port 546: {}", e);
                return Ok(1);
            }
        };
        let _ = sock.set_read_timeout(Some(Duration::from_secs(3)));

        let solicit = [1u8, 0x12, 0x34, 0x56];

        let _ = sock.send_to(&solicit, "[ff02::1:2]:547");

        let mut buf = [0u8; 1500];
        match sock.recv_from(&mut buf) {
            Ok((len, _)) => {
                if len >= 4 && buf[0] == 2 {
                    println!("udhcpc6: received ADVERTISE on {}", iface);
                    return Ok(0);
                }
            }
            Err(_) => {
                eprintln!("udhcpc6: no response received on {}", iface);
            }
        }
        Ok(0)
    }
}

pub struct UdhcpdApplet;

impl Applet for UdhcpdApplet {
    fn name(&self) -> &'static str {
        "udhcpd"
    }
    fn description(&self) -> &'static str {
        "DHCP server daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let conf_path = if !args.is_empty() && !args[0].as_bytes().starts_with(b"-") {
            args[0].as_os_str()
        } else {
            Path::new("/etc/udhcpd.conf").as_os_str()
        };

        let mut start_ip = Ipv4Addr::new(192, 168, 1, 100);
        let mut end_ip = Ipv4Addr::new(192, 168, 1, 200);

        if let Ok(content) = fs::read_to_string(conf_path) {
            for line in content.lines() {
                let l = line.trim();
                let parts: Vec<&str> = l.split_whitespace().collect();
                if parts.len() >= 2 {
                    if parts[0] == "start" {
                        if let Ok(ip) = parts[1].parse::<Ipv4Addr>() {
                            start_ip = ip;
                        }
                    } else if parts[0] == "end" {
                        if let Ok(ip) = parts[1].parse::<Ipv4Addr>() {
                            end_ip = ip;
                        }
                    }
                }
            }
        }

        let sock = match UdpSocket::bind("0.0.0.0:67") {
            Ok(s) => s,
            Err(e) => {
                eprintln!("udhcpd: cannot bind port 67: {}", e);
                return Ok(1);
            }
        };
        let _ = sock.set_broadcast(true);

        let mut cur_ip = u32::from(start_ip);
        let max_ip = u32::from(end_ip);

        let mut buf = [0u8; 1500];
        while let Ok((len, _)) = sock.recv_from(&mut buf) {
            if len < 240 || buf[0] != 1 {
                continue;
            }

            let mut is_discover = false;
            let mut is_request = false;
            let mut opt_idx = 240;
            while opt_idx < len && buf[opt_idx] != 255 {
                let opt = buf[opt_idx];
                if opt == 0 {
                    opt_idx += 1;
                    continue;
                }
                if opt_idx + 1 >= len {
                    break;
                }
                let olen = buf[opt_idx + 1] as usize;
                if opt == 53 && olen >= 1 && opt_idx + 2 < len {
                    if buf[opt_idx + 2] == 1 {
                        is_discover = true;
                    } else if buf[opt_idx + 2] == 3 {
                        is_request = true;
                    }
                }
                opt_idx += 2 + olen;
            }

            if !is_discover && !is_request {
                continue;
            }

            let mut reply = vec![0u8; 300];
            reply[0] = 2;
            reply[1] = buf[1];
            reply[2] = buf[2];
            reply[4..8].copy_from_slice(&buf[4..8]);
            reply[28..44].copy_from_slice(&buf[28..44]);

            let offer_ip = Ipv4Addr::from(cur_ip);
            reply[16..20].copy_from_slice(&offer_ip.octets());

            reply[236..240].copy_from_slice(&[99, 130, 83, 99]);
            reply[240] = 53;
            reply[241] = 1;
            reply[242] = if is_discover { 2 } else { 5 };

            reply[243] = 51;
            reply[244] = 4;
            reply[245..249].copy_from_slice(&86400u32.to_be_bytes());

            reply[249] = 255;

            let _ = sock.send_to(&reply, ("255.255.255.255", 68));

            if is_request {
                if cur_ip < max_ip {
                    cur_ip += 1;
                } else {
                    cur_ip = u32::from(start_ip);
                }
            }
        }
        Ok(0)
    }
}
