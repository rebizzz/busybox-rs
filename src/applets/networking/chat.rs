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

pub struct ChatApplet;

impl Applet for ChatApplet {
    fn name(&self) -> &'static str {
        "chat"
    }
    fn description(&self) -> &'static str {
        "Automate conversational exchange with modem"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut pairs = Vec::new();
        let mut i = 0;
        while i < args.len() {
            if !args[i].as_bytes().starts_with(b"-") {
                pairs.push(String::from_utf8_lossy(args[i].as_bytes()).to_string());
            }
            i += 1;
        }

        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut reader = BufReader::new(stdin.lock());
        let mut writer = stdout.lock();

        let mut idx = 0;
        while idx < pairs.len() {
            let expect = &pairs[idx];
            idx += 1;

            if !expect.is_empty() && expect != "\"\"" {
                let mut matched = false;
                let mut line = String::new();
                while reader.read_line(&mut line).is_ok() && !line.is_empty() {
                    if line.contains(expect) {
                        matched = true;
                        break;
                    }
                    line.clear();
                }
                if !matched {
                    eprintln!("chat: failed to match {}", expect);
                    return Ok(1);
                }
            }

            if idx < pairs.len() {
                let send = &pairs[idx];
                idx += 1;
                let send_s = if send == "\"\"" { "" } else { send };
                let _ = writer.write_all(send_s.as_bytes());
                let _ = writer.write_all(b"\r\n");
                let _ = writer.flush();
            }
        }

        Ok(0)
    }
}
