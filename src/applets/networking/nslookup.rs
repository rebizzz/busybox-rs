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

pub struct NslookupApplet;
impl Applet for NslookupApplet {
    fn name(&self) -> &'static str {
        "nslookup"
    }
    fn description(&self) -> &'static str {
        "Query the nameserver for the IP address of a host"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut host_arg: Option<String> = None;
        let mut server_arg: Option<String> = None;

        for arg in args {
            let s = arg.to_string_lossy();
            if !s.starts_with('-') {
                if host_arg.is_none() {
                    host_arg = Some(s.to_string());
                } else if server_arg.is_none() {
                    server_arg = Some(s.to_string());
                }
            }
        }

        let host = match host_arg {
            Some(h) => h,
            None => {
                eprintln!("Usage: nslookup <host> [server]");
                return Ok(1);
            }
        };

        let stdout = io::stdout();
        let mut out = stdout.lock();

        let _ = writeln!(
            out,
            "Server:\t\t{}",
            server_arg.as_deref().unwrap_or("default")
        );
        let _ = writeln!(out, "\nName:\t{}", host);

        let c_host = match std::ffi::CString::new(host.as_bytes()) {
            Ok(c) => c,
            Err(_) => return Ok(1),
        };

        let mut res: *mut libc::addrinfo = std::ptr::null_mut();
        let ret = unsafe {
            libc::getaddrinfo(
                c_host.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                &mut res,
            )
        };
        if ret != 0 {
            eprintln!("nslookup: can't resolve '{}': getaddrinfo failed", host);
            return Ok(1);
        }

        let mut curr = res;
        while !curr.is_null() {
            unsafe {
                let ai = &*curr;
                if ai.ai_family == libc::AF_INET {
                    let sin = &*(ai.ai_addr as *const libc::sockaddr_in);
                    let ip = Ipv4Addr::from(sin.sin_addr.s_addr.to_ne_bytes());
                    let _ = writeln!(out, "Address:\t{}", ip);
                } else if ai.ai_family == libc::AF_INET6 {
                    let sin6 = &*(ai.ai_addr as *const libc::sockaddr_in6);
                    let ip = Ipv6Addr::from(sin6.sin6_addr.s6_addr);
                    let _ = writeln!(out, "Address:\t{}", ip);
                }
                curr = ai.ai_next;
            }
        }

        unsafe {
            libc::freeaddrinfo(res);
        }

        Ok(0)
    }
}
