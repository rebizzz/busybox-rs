use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;

pub struct I2ctransferApplet;

impl Applet for I2ctransferApplet {
    fn name(&self) -> &'static str {
        "i2ctransfer"
    }

    fn description(&self) -> &'static str {
        "Send user-defined I2C messages in one transfer"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut bus_num = None;
        let mut msgs_args = Vec::new();

        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-y" || b == b"-f" || b == b"-a" || b == b"-v" {
            } else if !b.starts_with(b"-") {
                if bus_num.is_none() {
                    bus_num = parse_u32(b);
                } else {
                    msgs_args.push(&args[i]);
                }
            }
            i += 1;
        }

        let bus = match bus_num {
            Some(b) => b,
            None => {
                eprintln!("Usage: i2ctransfer [-y] BUS {{r|w}}LEN[@ADDR] [DATA...]");
                return Ok(1);
            }
        };

        let dev_path = format!("/dev/i2c-{}", bus);
        let f = match OpenOptions::new().read(true).write(true).open(&dev_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("i2ctransfer: {}: {}", dev_path, e);
                return Ok(1);
            }
        };

        let mut i2c_msgs = Vec::new();
        let mut buffers = Vec::new();

        let mut idx = 0;
        while idx < msgs_args.len() {
            let desc = msgs_args[idx].as_bytes();
            idx += 1;
            let is_read = desc.starts_with(b"r") || desc.starts_with(b"R");
            let is_write = desc.starts_with(b"w") || desc.starts_with(b"W");
            if !is_read && !is_write {
                continue;
            }

            let rest = &desc[1..];
            let mut at_parts = rest.split(|&b| b == b'@');
            let len_part = at_parts.next().unwrap_or(b"");
            let addr_part = at_parts.next().unwrap_or(b"0");

            let len = parse_u32(len_part).unwrap_or(0) as usize;
            let addr = parse_u32(addr_part).unwrap_or(0) as u16;

            let mut buf = vec![0u8; len];
            if is_write {
                for b_byte in buf.iter_mut() {
                    if idx < msgs_args.len()
                        && !msgs_args[idx].as_bytes().starts_with(b"r")
                        && !msgs_args[idx].as_bytes().starts_with(b"w")
                    {
                        if let Some(v) = parse_u32(msgs_args[idx].as_bytes()) {
                            *b_byte = v as u8;
                        }
                        idx += 1;
                    }
                }
            }
            buffers.push((is_read, buf, addr));
        }

        for (is_read, buf, addr) in &mut buffers {
            i2c_msgs.push(I2cMsg {
                addr: *addr,
                flags: if *is_read { I2C_M_RD } else { 0 },
                len: buf.len() as u16,
                buf: buf.as_mut_ptr(),
            });
        }

        let mut rdwr = I2cRdwrIoctlData {
            msgs: i2c_msgs.as_mut_ptr(),
            nmsgs: i2c_msgs.len() as u32,
        };

        let ret = unsafe {
            libc::ioctl(
                f.as_raw_fd(),
                I2C_RDWR,
                &mut rdwr as *mut I2cRdwrIoctlData as *mut libc::c_void,
            )
        };
        if ret < 0 {
            eprintln!(
                "i2ctransfer: I2C_RDWR failed: {}",
                io::Error::last_os_error()
            );
            return Ok(1);
        }

        for (is_read, buf, _) in &buffers {
            if *is_read {
                for &b in buf {
                    print!("0x{:02x} ", b);
                }
                println!();
            }
        }
        Ok(0)
    }
}
