use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::net::Ipv4Addr;
use std::os::unix::ffi::OsStrExt;

pub struct IpcalcApplet;

impl Applet for IpcalcApplet {
    fn name(&self) -> &'static str {
        "ipcalc"
    }
    fn description(&self) -> &'static str {
        "Calculate IP network, broadcast, and netmask"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut opt_net = false;
        let mut opt_bcast = false;
        let mut opt_mask = false;
        let mut opt_prefix = false;
        let mut target = None;
        let mut netmask_arg = None;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-n" || b == b"--network" {
                opt_net = true;
            } else if b == b"-b" || b == b"--broadcast" {
                opt_bcast = true;
            } else if b == b"-m" || b == b"--netmask" {
                opt_mask = true;
            } else if b == b"-p" || b == b"--prefix" {
                opt_prefix = true;
            } else if !b.starts_with(b"-") {
                if target.is_none() {
                    target = Some(&args[i]);
                } else if netmask_arg.is_none() {
                    netmask_arg = Some(&args[i]);
                }
            }
            i += 1;
        }

        let target = match target {
            Some(t) => String::from_utf8_lossy(t.as_bytes()).to_string(),
            None => {
                eprintln!("Usage: ipcalc [options] <address>[/prefix] [netmask]");
                return Ok(1);
            }
        };

        let (ip_str, prefix_len) = if let Some(slash) = target.find('/') {
            let p: u32 = target[slash + 1..].parse().unwrap_or(32);
            (&target[..slash], p)
        } else if let Some(m_arg) = netmask_arg {
            let ms = String::from_utf8_lossy(m_arg.as_bytes()).to_string();
            if let Ok(mask_ip) = ms.parse::<Ipv4Addr>() {
                let u = u32::from(mask_ip);
                (target.as_str(), u.count_ones())
            } else {
                (target.as_str(), ms.parse().unwrap_or(32))
            }
        } else {
            (target.as_str(), 24)
        };

        let ip = match ip_str.parse::<Ipv4Addr>() {
            Ok(ip) => ip,
            Err(e) => {
                eprintln!("ipcalc: invalid IP: {}", e);
                return Ok(1);
            }
        };

        let ip_u = u32::from(ip);
        let mask_u = if prefix_len == 0 {
            0
        } else {
            !0u32 << (32 - prefix_len)
        };
        let net_u = ip_u & mask_u;
        let bcast_u = net_u | !mask_u;

        let net_ip = Ipv4Addr::from(net_u);
        let bcast_ip = Ipv4Addr::from(bcast_u);
        let mask_ip = Ipv4Addr::from(mask_u);

        let any_flag = opt_net || opt_bcast || opt_mask || opt_prefix;

        if !any_flag || opt_net {
            println!("NETWORK={}", net_ip);
        }
        if !any_flag || opt_mask {
            println!("NETMASK={}", mask_ip);
        }
        if !any_flag || opt_bcast {
            println!("BROADCAST={}", bcast_ip);
        }
        if !any_flag || opt_prefix {
            println!("PREFIX={}", prefix_len);
        }

        Ok(0)
    }
}
