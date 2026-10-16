use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::net::UdpSocket;
use std::os::unix::ffi::OsStrExt;
use std::time::Duration;

pub struct Udhcpc6Applet;

impl Applet for Udhcpc6Applet {
    fn name(&self) -> &'static str {
        "udhcpc6"
    }
    fn description(&self) -> &'static str {
        "DHCPv6 client daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut iface = "eth0".to_string();
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-i" && i + 1 < args.len() {
                i += 1;
                iface = String::from_utf8_lossy(args[i].as_bytes()).to_string();
            }
            i += 1;
        }

        let sock = match UdpSocket::bind("[::]:546") {
            Ok(s) => s,
            Err(e) => {
                eprintln!("udhcpc6: cannot bind port 546: {}", e);
                return Ok(1);
            }
        };
        let _ = sock.set_read_timeout(Some(Duration::from_secs(3)));

        let solicit = [1u8, 0x12, 0x34, 0x56];

        let _ = sock.send_to(&solicit, "[ff02::1:2]:547");

        let mut buf = [0u8; 1500];
        match sock.recv_from(&mut buf) {
            Ok((len, _)) => {
                if len >= 4 && buf[0] == 2 {
                    println!("udhcpc6: received ADVERTISE on {}", iface);
                    return Ok(0);
                }
            }
            Err(_) => {
                eprintln!("udhcpc6: no response received on {}", iface);
            }
        }
        Ok(0)
    }
}
