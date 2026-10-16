use std::os::unix::io::AsRawFd;
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;

pub struct FbsetApplet;

impl Applet for FbsetApplet {
    fn name(&self) -> &'static str {
        "fbset"
    }

    fn description(&self) -> &'static str {
        "Show or set framebuffer video modes"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut fb_dev = "/dev/fb0";
        let mut show_all = false;
        let mut set_geo = None;
        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-fb" && i + 1 < args.len() {
                i += 1;
                if let Ok(s) = std::str::from_utf8(args[i].as_bytes()) {
                    fb_dev = s;
                }
            } else if b == b"-a" || b == b"--all" {
                show_all = true;
            } else if (b == b"-g" || b == b"--geometry") && i + 5 < args.len() {
                let xr = parse_u32(args[i + 1].as_bytes()).unwrap_or(0);
                let yr = parse_u32(args[i + 2].as_bytes()).unwrap_or(0);
                let vxr = parse_u32(args[i + 3].as_bytes()).unwrap_or(xr);
                let vyr = parse_u32(args[i + 4].as_bytes()).unwrap_or(yr);
                let d = parse_u32(args[i + 5].as_bytes()).unwrap_or(32);
                set_geo = Some((xr, yr, vxr, vyr, d));
                i += 5;
            }
            let _ = show_all;
            i += 1;
        }

        let f = match OpenOptions::new().read(true).write(true).open(fb_dev) {
            Ok(f) => f,
            Err(_) => match OpenOptions::new().read(true).open(fb_dev) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("fbset: cannot open {}: {}", fb_dev, e);
                    return Ok(1);
                }
            },
        };

        let mut vinfo = FbVarScreeninfo::default();
        let ret = unsafe {
            libc::ioctl(
                f.as_raw_fd(),
                FBIOGET_VSCREENINFO,
                &mut vinfo as *mut FbVarScreeninfo as *mut libc::c_void,
            )
        };
        if ret < 0 {
            eprintln!(
                "fbset: FBIOGET_VSCREENINFO failed: {}",
                io::Error::last_os_error()
            );
            return Ok(1);
        }

        if let Some((xr, yr, vxr, vyr, d)) = set_geo {
            vinfo.xres = xr;
            vinfo.yres = yr;
            vinfo.xres_virtual = vxr;
            vinfo.yres_virtual = vyr;
            vinfo.bits_per_pixel = d;
            let ret2 = unsafe {
                libc::ioctl(
                    f.as_raw_fd(),
                    FBIOPUT_VSCREENINFO,
                    &mut vinfo as *mut FbVarScreeninfo as *mut libc::c_void,
                )
            };
            if ret2 < 0 {
                eprintln!(
                    "fbset: FBIOPUT_VSCREENINFO failed: {}",
                    io::Error::last_os_error()
                );
                return Ok(1);
            }
        } else {
            println!("mode \"{}x{}\"", vinfo.xres, vinfo.yres);
            println!(
                "    geometry {} {} {} {} {}",
                vinfo.xres,
                vinfo.yres,
                vinfo.xres_virtual,
                vinfo.yres_virtual,
                vinfo.bits_per_pixel
            );
            println!(
                "    timings {} {} {} {} {} {} {}",
                vinfo.pixclock,
                vinfo.left_margin,
                vinfo.right_margin,
                vinfo.upper_margin,
                vinfo.lower_margin,
                vinfo.hsync_len,
                vinfo.vsync_len
            );
            println!(
                "    rgba {}/{}, {}/{}, {}/{}, {}/{}",
                vinfo.red.length,
                vinfo.red.offset,
                vinfo.green.length,
                vinfo.green.offset,
                vinfo.blue.length,
                vinfo.blue.offset,
                vinfo.transp.length,
                vinfo.transp.offset
            );
            println!("endmode");
        }
        Ok(0)
    }
}
