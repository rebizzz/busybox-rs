use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

pub struct I2cgetApplet;

impl Applet for I2cgetApplet {
    fn name(&self) -> &'static str {
        "i2cget"
    }

    fn description(&self) -> &'static str {
        "Read from I2C/SMBus chip registers"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut bus_num = None;
        let mut chip_addr = None;
        let mut reg = None;

        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-y" || b == b"-f" {
            } else if !b.starts_with(b"-") {
                if bus_num.is_none() {
                    bus_num = parse_u32(b);
                } else if chip_addr.is_none() {
                    chip_addr = parse_u32(b);
                } else if reg.is_none() {
                    reg = parse_u32(b);
                }
            }
            i += 1;
        }

        let (bus, addr) = match (bus_num, chip_addr) {
            (Some(b), Some(a)) => (b, a),
            _ => {
                eprintln!("Usage: i2cget [-y] BUS CHIP-ADDRESS [DATA-ADDRESS]");
                return Ok(1);
            }
        };

        let dev_path = format!("/dev/i2c-{}", bus);
        let f = match OpenOptions::new().read(true).write(true).open(&dev_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("i2cget: {}: {}", dev_path, e);
                return Ok(1);
            }
        };

        let fd = f.as_raw_fd();
        let ret = unsafe { libc::ioctl(fd, I2C_SLAVE_FORCE, addr as libc::c_ulong) };
        if ret < 0 {
            eprintln!(
                "i2cget: set slave address failed: {}",
                io::Error::last_os_error()
            );
            return Ok(1);
        }

        let mut data = I2cSmbusData { byte: 0 };
        let (cmd, size) = match reg {
            Some(r) => (r as u8, I2C_SMBUS_BYTE_DATA),
            None => (0, I2C_SMBUS_BYTE),
        };

        let mut ioctl_data = I2cSmbusIoctlData {
            read_write: I2C_SMBUS_READ,
            command: cmd,
            size,
            data: &mut data,
        };

        let res = unsafe {
            libc::ioctl(
                fd,
                I2C_SMBUS,
                &mut ioctl_data as *mut I2cSmbusIoctlData as *mut libc::c_void,
            )
        };
        if res < 0 {
            eprintln!("i2cget: read failed: {}", io::Error::last_os_error());
            return Ok(1);
        }

        println!("0x{:02x}", unsafe { data.byte });
        Ok(0)
    }
}
