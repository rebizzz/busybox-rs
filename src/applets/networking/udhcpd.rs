use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self};
use std::io::BufRead;
use std::net::{Ipv4Addr, UdpSocket};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct UdhcpdApplet;

impl Applet for UdhcpdApplet {
    fn name(&self) -> &'static str {
        "udhcpd"
    }
    fn description(&self) -> &'static str {
        "DHCP server daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let conf_path = if !args.is_empty() && !args[0].as_bytes().starts_with(b"-") {
            args[0].as_os_str()
        } else {
            Path::new("/etc/udhcpd.conf").as_os_str()
        };

        let mut start_ip = Ipv4Addr::new(192, 168, 1, 100);
        let mut end_ip = Ipv4Addr::new(192, 168, 1, 200);

        if let Ok(content) = fs::read_to_string(conf_path) {
            for line in content.lines() {
                let l = line.trim();
                let parts: Vec<&str> = l.split_whitespace().collect();
                if parts.len() >= 2 {
                    if parts[0] == "start" {
                        if let Ok(ip) = parts[1].parse::<Ipv4Addr>() {
                            start_ip = ip;
                        }
                    } else if parts[0] == "end" {
                        if let Ok(ip) = parts[1].parse::<Ipv4Addr>() {
                            end_ip = ip;
                        }
                    }
                }
            }
        }

        let sock = match UdpSocket::bind("0.0.0.0:67") {
            Ok(s) => s,
            Err(e) => {
                eprintln!("udhcpd: cannot bind port 67: {}", e);
                return Ok(1);
            }
        };
        let _ = sock.set_broadcast(true);

        let mut cur_ip = u32::from(start_ip);
        let max_ip = u32::from(end_ip);

        let mut buf = [0u8; 1500];
        while let Ok((len, _)) = sock.recv_from(&mut buf) {
            if len < 240 || buf[0] != 1 {
                continue;
            }

            let mut is_discover = false;
            let mut is_request = false;
            let mut opt_idx = 240;
            while opt_idx < len && buf[opt_idx] != 255 {
                let opt = buf[opt_idx];
                if opt == 0 {
                    opt_idx += 1;
                    continue;
                }
                if opt_idx + 1 >= len {
                    break;
                }
                let olen = buf[opt_idx + 1] as usize;
                if opt == 53 && olen >= 1 && opt_idx + 2 < len {
                    if buf[opt_idx + 2] == 1 {
                        is_discover = true;
                    } else if buf[opt_idx + 2] == 3 {
                        is_request = true;
                    }
                }
                opt_idx += 2 + olen;
            }

            if !is_discover && !is_request {
                continue;
            }

            let mut reply = vec![0u8; 300];
            reply[0] = 2;
            reply[1] = buf[1];
            reply[2] = buf[2];
            reply[4..8].copy_from_slice(&buf[4..8]);
            reply[28..44].copy_from_slice(&buf[28..44]);

            let offer_ip = Ipv4Addr::from(cur_ip);
            reply[16..20].copy_from_slice(&offer_ip.octets());

            reply[236..240].copy_from_slice(&[99, 130, 83, 99]);
            reply[240] = 53;
            reply[241] = 1;
            reply[242] = if is_discover { 2 } else { 5 };

            reply[243] = 51;
            reply[244] = 4;
            reply[245..249].copy_from_slice(&86400u32.to_be_bytes());

            reply[249] = 255;

            let _ = sock.send_to(&reply, ("255.255.255.255", 68));

            if is_request {
                if cur_ip < max_ip {
                    cur_ip += 1;
                } else {
                    cur_ip = u32::from(start_ip);
                }
            }
        }
        Ok(0)
    }
}
