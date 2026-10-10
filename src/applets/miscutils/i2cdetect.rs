use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

pub struct I2cdetectApplet;

impl Applet for I2cdetectApplet {
    fn name(&self) -> &'static str {
        "i2cdetect"
    }

    fn description(&self) -> &'static str {
        "Detect I2C chips"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut list_busses = false;
        let mut bus_num = None;

        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-l" {
                list_busses = true;
            } else if b == b"-y" || b == b"-q" || b == b"-r" {
            } else if !b.starts_with(b"-") {
                bus_num = parse_u32(b);
            }
            i += 1;
        }

        if list_busses || (bus_num.is_none() && args.len() <= 1) {
            if let Ok(entries) = fs::read_dir("/sys/class/i2c-dev") {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    let name_path = entry.path().join("name");
                    let desc = fs::read_to_string(&name_path).unwrap_or_else(|_| "unknown".into());
                    println!("{}\t{:<20}\tI2C adapter", name_str, desc.trim());
                }
            }
            return Ok(0);
        }

        let bus = match bus_num {
            Some(b) => b,
            None => {
                eprintln!("Usage: i2cdetect [-y] [-a] BUS-NUMBER");
                return Ok(1);
            }
        };

        let dev_path = format!("/dev/i2c-{}", bus);
        let f = match OpenOptions::new().read(true).write(true).open(&dev_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("i2cdetect: {}: {}", dev_path, e);
                return Ok(1);
            }
        };

        println!("     0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f");
        for row in (0x00..=0x70).step_by(16) {
            print!("{:02x}:", row);
            for col in 0..16 {
                let addr = row + col;
                if !(0x03..=0x77).contains(&addr) {
                    print!("   ");
                    continue;
                }
                let mut data = I2cSmbusData { byte: 0 };
                let mut ioctl_data = I2cSmbusIoctlData {
                    read_write: I2C_SMBUS_READ,
                    command: 0,
                    size: I2C_SMBUS_QUICK,
                    data: &mut data,
                };
                let ret = unsafe {
                    libc::ioctl(f.as_raw_fd(), I2C_SLAVE, addr as libc::c_ulong);
                    libc::ioctl(
                        f.as_raw_fd(),
                        I2C_SMBUS,
                        &mut ioctl_data as *mut I2cSmbusIoctlData as *mut libc::c_void,
                    )
                };
                if ret >= 0 {
                    print!(" {:02x}", addr);
                } else {
                    print!(" --");
                }
            }
            println!();
        }
        Ok(0)
    }
}
