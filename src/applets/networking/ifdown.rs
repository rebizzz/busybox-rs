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

pub struct IfdownApplet;
impl Applet for IfdownApplet {
    fn name(&self) -> &'static str {
        "ifdown"
    }
    fn description(&self) -> &'static str {
        "Take a network interface down"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("Usage: ifdown <interface>");
            return Ok(1);
        }
        let ifname = &args[0];
        super::ifconfig::IfconfigApplet.run(&[ifname.clone(), OsString::from("down")])
    }
}
