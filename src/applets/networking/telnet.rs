use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::os::unix::ffi::OsStrExt;

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
