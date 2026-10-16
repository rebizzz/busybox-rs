use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::net::UdpSocket;
use std::os::unix::ffi::OsStrExt;
use std::time::Duration;

pub struct NtpdApplet;

impl Applet for NtpdApplet {
    fn name(&self) -> &'static str {
        "ntpd"
    }
    fn description(&self) -> &'static str {
        "NTP client and server daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut peer = "pool.ntp.org".to_string();
        let mut query_only = false;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-p" && i + 1 < args.len() {
                i += 1;
                peer = String::from_utf8_lossy(args[i].as_bytes()).to_string();
            } else if b == b"-q" {
                query_only = true;
            }
            i += 1;
        }

        let sock = match UdpSocket::bind("0.0.0.0:0") {
            Ok(s) => s,
            Err(e) => {
                eprintln!("ntpd: bind: {}", e);
                return Ok(1);
            }
        };
        let _ = sock.set_read_timeout(Some(Duration::from_secs(3)));

        let dest = format!("{}:123", peer);
        let mut pkt = [0u8; 48];
        pkt[0] = 0x1B;

        if sock.send_to(&pkt, &dest).is_err() {
            eprintln!("ntpd: send failed to {}", dest);
            return Ok(1);
        }

        let mut resp = [0u8; 48];
        match sock.recv_from(&mut resp) {
            Ok((len, _)) => {
                if len >= 48 {
                    let sec = u32::from_be_bytes([resp[40], resp[41], resp[42], resp[43]]);

                    if sec > 2208988800 {
                        let unix_sec = sec - 2208988800;
                        println!("ntpd: time from {}: {}", peer, unix_sec);
                        if !query_only {
                            let tv = libc::timeval {
                                tv_sec: unix_sec as libc::time_t,
                                tv_usec: 0,
                            };
                            unsafe {
                                libc::settimeofday(&tv, std::ptr::null());
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("ntpd: recv timeout: {}", e);
                return Ok(1);
            }
        }

        Ok(0)
    }
}
