use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self};
use std::io::BufRead;
use std::net::{Ipv4Addr, UdpSocket};
use std::os::unix::ffi::OsStrExt;

pub struct DnsdApplet;

impl Applet for DnsdApplet {
    fn name(&self) -> &'static str {
        "dnsd"
    }
    fn description(&self) -> &'static str {
        "Small static DNS server daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut port = 53u16;
        let mut conf_file = "/etc/hosts".to_string();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-p" && i + 1 < args.len() {
                i += 1;
                port = String::from_utf8_lossy(args[i].as_bytes())
                    .parse()
                    .unwrap_or(53);
            } else if b == b"-c" && i + 1 < args.len() {
                i += 1;
                conf_file = String::from_utf8_lossy(args[i].as_bytes()).to_string();
            }
            i += 1;
        }

        let sock = match UdpSocket::bind(("0.0.0.0", port)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("dnsd: bind: {}", e);
                return Ok(1);
            }
        };

        let mut buf = [0u8; 1024];
        while let Ok((len, src)) = sock.recv_from(&mut buf) {
            if len < 12 {
                continue;
            }

            let mut qname = String::new();
            let mut pos = 12;
            while pos < len && buf[pos] != 0 {
                let lab_len = buf[pos] as usize;
                pos += 1;
                if pos + lab_len <= len {
                    if !qname.is_empty() {
                        qname.push('.');
                    }
                    qname.push_str(&String::from_utf8_lossy(&buf[pos..pos + lab_len]));
                    pos += lab_len;
                } else {
                    break;
                }
            }
            if pos >= len || buf[pos] != 0 {
                continue;
            }
            pos += 1;
            if pos + 4 > len {
                continue;
            }
            let qtype = u16::from_be_bytes([buf[pos], buf[pos + 1]]);
            let _qclass = u16::from_be_bytes([buf[pos + 2], buf[pos + 3]]);
            pos += 4;

            let mut resolved_ip: Option<Ipv4Addr> = None;
            if qtype == 1 {
                if let Ok(content) = fs::read_to_string(&conf_file) {
                    for line in content.lines() {
                        let l = line.trim();
                        if l.is_empty() || l.starts_with('#') {
                            continue;
                        }
                        let items: Vec<&str> = l.split_whitespace().collect();
                        if items.len() >= 2 {
                            if let Ok(ip) = items[0].parse::<Ipv4Addr>() {
                                for name in &items[1..] {
                                    if name.eq_ignore_ascii_case(&qname) {
                                        resolved_ip = Some(ip);
                                        break;
                                    }
                                }
                            }
                        }
                        if resolved_ip.is_some() {
                            break;
                        }
                    }
                }
            }

            let mut resp = Vec::with_capacity(512);
            resp.extend_from_slice(&buf[0..2]);

            let flags: u16 = if resolved_ip.is_some() {
                0x8180
            } else {
                0x8183
            };
            resp.extend_from_slice(&flags.to_be_bytes());
            resp.extend_from_slice(&1u16.to_be_bytes());
            let ancount: u16 = if resolved_ip.is_some() { 1 } else { 0 };
            resp.extend_from_slice(&ancount.to_be_bytes());
            resp.extend_from_slice(&0u16.to_be_bytes());
            resp.extend_from_slice(&0u16.to_be_bytes());

            resp.extend_from_slice(&buf[12..pos]);

            if let Some(ip) = resolved_ip {
                resp.extend_from_slice(&[0xC0, 0x0C]);
                resp.extend_from_slice(&1u16.to_be_bytes());
                resp.extend_from_slice(&1u16.to_be_bytes());
                resp.extend_from_slice(&300u32.to_be_bytes());
                resp.extend_from_slice(&4u16.to_be_bytes());
                resp.extend_from_slice(&ip.octets());
            }

            let _ = sock.send_to(&resp, src);
        }

        Ok(0)
    }
}
