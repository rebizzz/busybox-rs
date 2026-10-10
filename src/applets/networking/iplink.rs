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

pub struct IplinkApplet;
impl Applet for IplinkApplet {
    fn name(&self) -> &'static str {
        "iplink"
    }
    fn description(&self) -> &'static str {
        "Manage network devices"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_ip_link(args)
    }
}
