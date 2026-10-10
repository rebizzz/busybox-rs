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
