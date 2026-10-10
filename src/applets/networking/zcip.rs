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
