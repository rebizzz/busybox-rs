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

pub struct BrctlApplet;
impl Applet for BrctlApplet {
    fn name(&self) -> &'static str {
        "brctl"
    }
    fn description(&self) -> &'static str {
        "Ethernet bridge administration"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("Usage: brctl addbr|delbr|addif|delif|show ...");
            return Ok(1);
        }

        let cmd = args[0].to_string_lossy();
        let fd = open_socket_dgram()?;

        match cmd.as_ref() {
            "addbr" => {
                if args.len() < 2 {
                    eprintln!("brctl: addbr <bridge>");
                    unsafe { libc::close(fd) };
                    return Ok(1);
                }
                let brname = CString::new(args[1].as_bytes()).unwrap();
                let ret = unsafe { libc::ioctl(fd, SIOCBRADDBR as _, brname.as_ptr()) };
                unsafe { libc::close(fd) };
                if ret < 0 {
                    eprintln!("brctl addbr: {}", io::Error::last_os_error());
                    return Ok(1);
                }
            }
            "delbr" => {
                if args.len() < 2 {
                    eprintln!("brctl: delbr <bridge>");
                    unsafe { libc::close(fd) };
                    return Ok(1);
                }
                let brname = CString::new(args[1].as_bytes()).unwrap();
                let ret = unsafe { libc::ioctl(fd, SIOCBRDELBR as _, brname.as_ptr()) };
                unsafe { libc::close(fd) };
                if ret < 0 {
                    eprintln!("brctl delbr: {}", io::Error::last_os_error());
                    return Ok(1);
                }
            }
            "addif" | "delif" => {
                if args.len() < 3 {
                    eprintln!("brctl {} <bridge> <device>", cmd);
                    unsafe { libc::close(fd) };
                    return Ok(1);
                }
                let brname = args[1].as_bytes();
                let devname = args[2].as_bytes();

                let mut ifr: libc::ifreq = unsafe { mem::zeroed() };
                set_ifr_name(&mut ifr, brname);

                let dev_c = CString::new(devname).unwrap();
                let ifindex = unsafe { libc::if_nametoindex(dev_c.as_ptr()) };
                ifr.ifr_ifru.ifru_ifindex = ifindex as libc::c_int;

                let req = if cmd == "addif" {
                    SIOCBRADDIF
                } else {
                    SIOCBRDELIF
                };
                let ret = unsafe { libc::ioctl(fd, req as _, &ifr) };
                unsafe { libc::close(fd) };
                if ret < 0 {
                    eprintln!("brctl {}: {}", cmd, io::Error::last_os_error());
                    return Ok(1);
                }
            }
            "show" => {
                unsafe { libc::close(fd) };
                let stdout = io::stdout();
                let mut out = stdout.lock();
                let _ = writeln!(out, "bridge name\tbridge id\t\tSTP enabled\tinterfaces");

                if let Ok(entries) = fs::read_dir("/sys/class/net") {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let br_path = path.join("bridge");
                        if br_path.exists() {
                            let name = entry.file_name().to_string_lossy().to_string();
                            let mut ifaces = Vec::new();
                            let brif_path = path.join("brif");
                            if let Ok(ifs) = fs::read_dir(brif_path) {
                                for ife in ifs.flatten() {
                                    ifaces.push(ife.file_name().to_string_lossy().to_string());
                                }
                            }
                            let _ = writeln!(
                                out,
                                "{}\t\t8000.000000000000\tno\t\t{}",
                                name,
                                ifaces.join("\n\t\t\t\t\t\t\t")
                            );
                        }
                    }
                }
            }
            _ => {
                unsafe { libc::close(fd) };
                eprintln!("brctl: unknown command '{}'", cmd);
                return Ok(1);
            }
        }

        Ok(0)
    }
}
