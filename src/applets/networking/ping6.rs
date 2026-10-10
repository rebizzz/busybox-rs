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

pub struct Ping6Applet;
impl Applet for Ping6Applet {
    fn name(&self) -> &'static str {
        "ping6"
    }
    fn description(&self) -> &'static str {
        "Send ICMPv6 ECHO_REQUEST to network hosts"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_ping_generic(args, true)
    }
}
