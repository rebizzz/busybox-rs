use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::net::{Ipv4Addr, UdpSocket};
use std::os::unix::ffi::OsStrExt;
use std::time::Duration;

pub struct UdhcpcApplet;

impl Applet for UdhcpcApplet {
    fn name(&self) -> &'static str {
        "udhcpc"
    }
    fn description(&self) -> &'static str {
        "DHCP client daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut _iface = "eth0".to_string();
        let mut now = false;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-i" && i + 1 < args.len() {
                i += 1;
                _iface = String::from_utf8_lossy(args[i].as_bytes()).to_string();
            } else if b == b"-n" || b == b"-q" {
                now = true;
            }
            i += 1;
        }

        let sock = match UdpSocket::bind("0.0.0.0:68") {
            Ok(s) => s,
            Err(e) => {
                eprintln!("udhcpc: cannot bind to port 68: {}", e);
                return Ok(1);
            }
        };
        let _ = sock.set_broadcast(true);
        let _ = sock.set_read_timeout(Some(Duration::from_secs(3)));

        let mut packet = vec![0u8; 300];
        packet[0] = 1;
        packet[1] = 1;
        packet[2] = 6;
        let xid = 0x12345678u32.to_be_bytes();
        packet[4..8].copy_from_slice(&xid);
        packet[10] = 0x80;

        packet[236..240].copy_from_slice(&[99, 130, 83, 99]);

        packet[240] = 53;
        packet[241] = 1;
        packet[242] = 1;

        packet[243] = 255;

        let _ = sock.send_to(&packet, ("255.255.255.255", 67));

        let mut in_buf = [0u8; 1500];
        let mut attempts = 0;
        loop {
            match sock.recv_from(&mut in_buf) {
                Ok((len, _)) => {
                    if len >= 240 && in_buf[0] == 2 && in_buf[4..8] == xid {
                        let yiaddr = Ipv4Addr::new(in_buf[16], in_buf[17], in_buf[18], in_buf[19]);
                        println!("udhcpc: obtained lease for {}", yiaddr);
                        return Ok(0);
                    }
                }
                Err(_) => {
                    attempts += 1;
                    if attempts >= 3 {
                        if now {
                            eprintln!("udhcpc: no lease, failing");
                            return Ok(1);
                        }

                        let _ = sock.send_to(&packet, ("255.255.255.255", 67));
                        attempts = 0;
                    }
                }
            }
        }
    }
}
