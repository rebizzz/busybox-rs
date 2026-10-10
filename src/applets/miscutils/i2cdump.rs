use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

pub struct I2cdumpApplet;

impl Applet for I2cdumpApplet {
    fn name(&self) -> &'static str {
        "i2cdump"
    }

    fn description(&self) -> &'static str {
        "Examine I2C registers"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut bus_num = None;
        let mut chip_addr = None;

        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-y" || b == b"-f" {
            } else if !b.starts_with(b"-") {
                if bus_num.is_none() {
                    bus_num = parse_u32(b);
                } else if chip_addr.is_none() {
                    chip_addr = parse_u32(b);
                }
            }
            i += 1;
        }

        let (bus, addr) = match (bus_num, chip_addr) {
            (Some(b), Some(a)) => (b, a),
            _ => {
                eprintln!("Usage: i2cdump [-y] BUS CHIP-ADDRESS");
                return Ok(1);
            }
        };

        let dev_path = format!("/dev/i2c-{}", bus);
        let f = match OpenOptions::new().read(true).write(true).open(&dev_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("i2cdump: {}: {}", dev_path, e);
                return Ok(1);
            }
        };

        let fd = f.as_raw_fd();
        let ret = unsafe { libc::ioctl(fd, I2C_SLAVE_FORCE, addr as libc::c_ulong) };
        if ret < 0 {
            eprintln!(
                "i2cdump: failed setting slave addr: {}",
                io::Error::last_os_error()
            );
            return Ok(1);
        }

        println!("     0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f    0123456789abcdef");
        for row in (0x00..=0xF0).step_by(16) {
            print!("{:02x}:", row);
            let mut ascii = [b'.'; 16];
            for (col, ascii_ch) in ascii.iter_mut().enumerate() {
                let cmd = (row + col as u32) as u8;
                let mut data = I2cSmbusData { byte: 0 };
                let mut ioctl_data = I2cSmbusIoctlData {
                    read_write: I2C_SMBUS_READ,
                    command: cmd,
                    size: I2C_SMBUS_BYTE_DATA,
                    data: &mut data,
                };
                let res = unsafe {
                    libc::ioctl(
                        fd,
                        I2C_SMBUS,
                        &mut ioctl_data as *mut I2cSmbusIoctlData as *mut libc::c_void,
                    )
                };
                if res >= 0 {
                    let b = unsafe { data.byte };
                    print!(" {:02x}", b);
                    if b.is_ascii_graphic() {
                        *ascii_ch = b;
                    }
                } else {
                    print!(" XX");
                    *ascii_ch = b'X';
                }
            }
            print!("    ");
            let _ = io::stdout().write_all(&ascii);
            println!();
        }
        Ok(0)
    }
}
