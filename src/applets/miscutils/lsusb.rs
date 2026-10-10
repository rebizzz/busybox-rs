use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

pub struct LsusbApplet;

impl Applet for LsusbApplet {
    fn name(&self) -> &'static str {
        "lsusb"
    }

    fn description(&self) -> &'static str {
        "List USB devices"
    }

    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let usb_dir = Path::new("/sys/bus/usb/devices");
        let entries = match fs::read_dir(usb_dir) {
            Ok(e) => e,
            Err(err) => {
                eprintln!("lsusb: /sys/bus/usb/devices: {}", err);
                return Ok(1);
            }
        };

        let mut devices = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            let busnum_str = fs::read_to_string(path.join("busnum")).unwrap_or_default();
            let devnum_str = fs::read_to_string(path.join("devnum")).unwrap_or_default();
            let id_vendor = fs::read_to_string(path.join("idVendor")).unwrap_or_default();
            let id_product = fs::read_to_string(path.join("idProduct")).unwrap_or_default();
            let product_name = fs::read_to_string(path.join("product")).unwrap_or_default();

            if busnum_str.is_empty() || devnum_str.is_empty() || id_vendor.is_empty() {
                continue;
            }

            let busnum: u32 = busnum_str.trim().parse().unwrap_or(0);
            let devnum: u32 = devnum_str.trim().parse().unwrap_or(0);
            let vendor = id_vendor.trim().to_string();
            let product = id_product.trim().to_string();
            let desc = product_name.trim().to_string();

            devices.push((busnum, devnum, vendor, product, desc));
        }

        devices.sort_by_key(|d| (d.0, d.1));
        for (bus, dev, vendor, product, desc) in devices {
            if desc.is_empty() {
                println!(
                    "Bus {:03} Device {:03}: ID {}:{}",
                    bus, dev, vendor, product
                );
            } else {
                println!(
                    "Bus {:03} Device {:03}: ID {}:{} {}",
                    bus, dev, vendor, product, desc
                );
            }
        }
        Ok(0)
    }
}
