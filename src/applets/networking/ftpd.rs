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
