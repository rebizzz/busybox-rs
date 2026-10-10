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

pub struct TunctlApplet;
impl Applet for TunctlApplet {
    fn name(&self) -> &'static str {
        "tunctl"
    }
    fn description(&self) -> &'static str {
        "Create and manage persistent TUN/TAP interfaces"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut delete = false;
        let mut tun_name = "tap0".to_string();
        let mut user_uid: Option<u32> = None;

        let mut i = 0;
        while i < args.len() {
            let s = args[i].to_string_lossy();
            if s == "-d" && i + 1 < args.len() {
                delete = true;
                tun_name = args[i + 1].to_string_lossy().to_string();
                i += 1;
            } else if s == "-t" && i + 1 < args.len() {
                tun_name = args[i + 1].to_string_lossy().to_string();
                i += 1;
            } else if s == "-u" && i + 1 < args.len() {
                user_uid = args[i + 1].to_string_lossy().parse().ok();
                i += 1;
            }
            i += 1;
        }

        let tun_path = CString::new("/dev/net/tun").unwrap();
        let fd = unsafe { libc::open(tun_path.as_ptr(), libc::O_RDWR) };
        if fd < 0 {
            eprintln!(
                "tunctl: failed to open /dev/net/tun: {}",
                io::Error::last_os_error()
            );
            return Ok(1);
        }

        let mut ifr: libc::ifreq = unsafe { mem::zeroed() };
        set_ifr_name(&mut ifr, tun_name.as_bytes());
        ifr.ifr_ifru.ifru_flags = (IFF_TAP | IFF_NO_PI) as libc::c_short;

        let ret = unsafe { libc::ioctl(fd, TUNSETIFF as _, &ifr) };
        if ret < 0 {
            eprintln!("tunctl: TUNSETIFF: {}", io::Error::last_os_error());
            unsafe { libc::close(fd) };
            return Ok(1);
        }

        if delete {
            let ret = unsafe { libc::ioctl(fd, TUNSETPERSIST as _, 0) };
            unsafe { libc::close(fd) };
            if ret < 0 {
                eprintln!(
                    "tunctl: failed to delete {}: {}",
                    tun_name,
                    io::Error::last_os_error()
                );
                return Ok(1);
            }
            let stdout = io::stdout();
            let mut out = stdout.lock();
            let _ = writeln!(out, "Set '{}' non-persistent", tun_name);
            return Ok(0);
        }

        if let Some(uid) = user_uid {
            unsafe {
                libc::ioctl(fd, TUNSETOWNER as _, uid as libc::c_ulong);
            }
        }

        let ret = unsafe { libc::ioctl(fd, TUNSETPERSIST as _, 1) };
        unsafe { libc::close(fd) };
        if ret < 0 {
            eprintln!(
                "tunctl: failed to set persistent: {}",
                io::Error::last_os_error()
            );
            return Ok(1);
        }

        let stdout = io::stdout();
        let mut out = stdout.lock();
        let _ = writeln!(out, "Set '{}' persistent", tun_name);
        Ok(0)
    }
}
