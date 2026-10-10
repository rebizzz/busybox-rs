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
