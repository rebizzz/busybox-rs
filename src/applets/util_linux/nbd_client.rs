use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::{CString, OsString};
use std::io::{self};
use std::net::TcpStream;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;

pub struct NbdClientApplet;
impl Applet for NbdClientApplet {
    fn name(&self) -> &'static str {
        "nbd-client"
    }
    fn description(&self) -> &'static str {
        "Network block device client"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut disconnect = false;
        let mut block_size = 4096u32;
        let mut timeout = 0u32;
        let mut positional = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-d" {
                disconnect = true;
            } else if (b == b"-b" || b == b"-block-size") && i + 1 < args.len() {
                block_size = args[i + 1].to_string_lossy().parse().unwrap_or(4096);
                i += 2;
                continue;
            } else if (b == b"-t" || b == b"-timeout") && i + 1 < args.len() {
                timeout = args[i + 1].to_string_lossy().parse().unwrap_or(0);
                i += 2;
                continue;
            } else if !b.starts_with(b"-") {
                positional.push(args[i].clone());
            }
            i += 1;
        }

        if disconnect {
            if positional.is_empty() {
                eprintln!("nbd-client: -d BLOCKDEV");
                return Ok(1);
            }
            let c_dev = CString::new(positional[0].as_bytes()).unwrap();
            let nbd_fd = unsafe { libc::open(c_dev.as_ptr(), libc::O_RDWR) };
            if nbd_fd < 0 {
                eprintln!(
                    "nbd-client: cannot open {}",
                    positional[0].to_string_lossy()
                );
                return Ok(1);
            }
            unsafe {
                libc::ioctl(nbd_fd, NBD_DISCONNECT, 0);
                libc::ioctl(nbd_fd, NBD_CLEAR_SOCK, 0);
                libc::close(nbd_fd);
            }
            return Ok(0);
        }

        if positional.len() < 2 {
            eprintln!("nbd-client: HOST [PORT] BLOCKDEV");
            return Ok(1);
        }

        let host = positional[0].to_string_lossy().to_string();
        let (port, device) = if positional.len() >= 3 {
            (
                positional[1]
                    .to_string_lossy()
                    .parse::<u16>()
                    .unwrap_or(10809),
                positional[2].clone(),
            )
        } else {
            (10809, positional[1].clone())
        };

        let c_dev = CString::new(device.as_bytes()).unwrap();
        let nbd_fd = unsafe { libc::open(c_dev.as_ptr(), libc::O_RDWR) };
        if nbd_fd < 0 {
            eprintln!("nbd-client: open device: {}", io::Error::last_os_error());
            return Ok(1);
        }

        let stream = match TcpStream::connect((host.as_str(), port)) {
            Ok(s) => s,
            Err(e) => {
                unsafe { libc::close(nbd_fd) };
                eprintln!("nbd-client: connect to {}:{}: {}", host, port, e);
                return Ok(1);
            }
        };

        let sock_fd = stream.as_raw_fd();

        unsafe {
            if timeout > 0 {
                libc::ioctl(nbd_fd, NBD_SET_TIMEOUT, timeout as libc::c_ulong);
            }
            libc::ioctl(nbd_fd, NBD_SET_BLKSIZE, block_size as libc::c_ulong);
            libc::ioctl(nbd_fd, NBD_SET_SOCK, sock_fd);
            libc::ioctl(nbd_fd, NBD_DO_IT, 0);
            libc::ioctl(nbd_fd, NBD_CLEAR_QUE, 0);
            libc::ioctl(nbd_fd, NBD_CLEAR_SOCK, 0);
            libc::close(nbd_fd);
        }

        Ok(0)
    }
}
