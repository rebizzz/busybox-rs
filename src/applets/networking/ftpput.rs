use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use std::net::TcpStream;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

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
