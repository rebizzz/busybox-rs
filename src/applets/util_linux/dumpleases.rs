use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::os::unix::ffi::OsStrExt;

pub struct DumpleasesApplet;
impl Applet for DumpleasesApplet {
    fn name(&self) -> &'static str {
        "dumpleases"
    }
    fn description(&self) -> &'static str {
        "Display DHCP leases granted by udhcpd"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut lease_file = "/var/lib/misc/udhcpd.leases";
        let mut show_abs = false;
        let mut show_remaining = false;
        let mut show_sec = false;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-f" && i + 1 < args.len() {
                lease_file = args[i + 1].to_str().unwrap_or(lease_file);
                i += 2;
                continue;
            } else if b == b"-a" {
                show_abs = true;
            } else if b == b"-r" {
                show_remaining = true;
            } else if b == b"-d" {
                show_sec = true;
            }
            i += 1;
        }

        let mut file = match File::open(lease_file) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("dumpleases: {}: {}", lease_file, e);
                return Ok(1);
            }
        };

        println!(
            "Mac Address       IP Address      Host Name       Expires {}",
            if show_abs { "at" } else { "in" }
        );

        let mut written_at_buf = [0u8; 8];
        if file.read_exact(&mut written_at_buf).is_err() {
            return Ok(0);
        }
        let written_at = u64::from_be_bytes(written_at_buf);
        let curr_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut lease_buf = [0u8; 36];
        while file.read_exact(&mut lease_buf).is_ok() {
            let expires_raw = u32::from_be_bytes(lease_buf[0..4].try_into().unwrap());
            let nip = u32::from_ne_bytes(lease_buf[4..8].try_into().unwrap());
            let mac = &lease_buf[8..14];
            let host_bytes = &lease_buf[14..34];
            let host_len = host_bytes.iter().position(|&b| b == 0).unwrap_or(20);
            let hostname = String::from_utf8_lossy(&host_bytes[..host_len]);

            let ip_octets = nip.to_le_bytes();
            let ip_str = format!(
                "{}.{}.{}.{}",
                ip_octets[0], ip_octets[1], ip_octets[2], ip_octets[3]
            );
            let mac_str = format!(
                "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
            );

            let expires_abs = expires_raw as u64 + written_at;
            let exp_str = if expires_abs <= curr_time {
                "expired".to_string()
            } else if show_sec {
                format!("{}", expires_abs - curr_time)
            } else if show_remaining || !show_abs {
                let rem = expires_abs - curr_time;
                format!("{:02}:{:02}:{:02}", rem / 3600, (rem % 3600) / 60, rem % 60)
            } else {
                format!("{}", expires_abs)
            };

            println!(
                "{:<17} {:<15} {:<15} {}",
                mac_str, ip_str, hostname, exp_str
            );
        }

        Ok(0)
    }
}
