use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{self, Read};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct HdparmApplet;

impl Applet for HdparmApplet {
    fn name(&self) -> &'static str {
        "hdparm"
    }

    fn description(&self) -> &'static str {
        "Get/set ATA/SATA drive parameters"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("Usage: hdparm [options] [device ...]");
            return Ok(1);
        }

        let mut get_geo = false;
        let mut standby = false;
        let mut dev_names = Vec::new();

        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-g" {
                get_geo = true;
            } else if b == b"-y" {
                standby = true;
            } else if !b.starts_with(b"-") {
                dev_names.push(&args[i]);
            }
            i += 1;
        }

        if dev_names.is_empty() {
            eprintln!("hdparm: no device specified");
            return Ok(1);
        }

        let mut ret_code = 0;
        for dev in dev_names {
            let path = Path::new(dev);
            let f = match OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NONBLOCK)
                .open(path)
            {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("hdparm: {}: {}", path.display(), e);
                    ret_code = 1;
                    continue;
                }
            };

            println!("\n{}:", path.display());

            if standby {
                let mut cmd = [0xE0u8, 0, 0, 0];
                let ret = unsafe {
                    libc::ioctl(
                        f.as_raw_fd(),
                        HDIO_DRIVE_CMD,
                        cmd.as_mut_ptr() as *mut libc::c_void,
                    )
                };
                if ret < 0 {
                    eprintln!("  standby failed: {}", io::Error::last_os_error());
                    ret_code = 1;
                } else {
                    println!("  issuing standby command");
                }
            }

            if get_geo || !standby {
                let mut geo = HdGeometry::default();
                let ret = unsafe {
                    libc::ioctl(
                        f.as_raw_fd(),
                        HDIO_GETGEO,
                        &mut geo as *mut HdGeometry as *mut libc::c_void,
                    )
                };
                if ret == 0 {
                    println!(
                        " geometry      = {}/{}/{}, sectors = {}, start = {}",
                        geo.cylinders,
                        geo.heads,
                        geo.sectors,
                        geo.cylinders as u64 * geo.heads as u64 * geo.sectors as u64,
                        geo.start
                    );
                } else if get_geo {
                    eprintln!("  HDIO_GETGEO failed: {}", io::Error::last_os_error());
                    ret_code = 1;
                }
            }
        }
        Ok(ret_code)
    }
}
