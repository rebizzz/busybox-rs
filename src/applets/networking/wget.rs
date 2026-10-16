use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

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
