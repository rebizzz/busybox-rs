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
