use std::os::unix::io::AsRawFd;
use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct FbsplashApplet;

impl Applet for FbsplashApplet {
    fn name(&self) -> &'static str {
        "fbsplash"
    }

    fn description(&self) -> &'static str {
        "Display image centered on framebuffer"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut fb_dev = "/dev/fb0";
        let mut image_file = None;
        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-d" && i + 1 < args.len() {
                i += 1;
                if let Ok(s) = std::str::from_utf8(args[i].as_bytes()) {
                    fb_dev = s;
                }
            } else if b == b"-s" && i + 1 < args.len() {
                i += 1;
                image_file = Some(&args[i]);
            } else if b == b"-i" && i + 1 < args.len() {
                i += 1;
            }
            i += 1;
        }

        let img_path = match image_file {
            Some(p) => p,
            None => {
                eprintln!("Usage: fbsplash -s IMAGE_FILE [-d FB_DEV]");
                return Ok(1);
            }
        };

        let mut img_data = Vec::new();
        let path = Path::new(img_path);
        match File::open(path) {
            Ok(mut f) => {
                let _ = f.read_to_end(&mut img_data);
            }
            Err(e) => {
                eprintln!("fbsplash: {}: {}", path.display(), e);
                return Ok(1);
            }
        }

        let fb_file = match OpenOptions::new().write(true).open(fb_dev) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("fbsplash: {}: {}", fb_dev, e);
                return Ok(1);
            }
        };

        let mut writer = io::BufWriter::new(fb_file);
        let _ = writer.write_all(&img_data);
        let _ = writer.flush();
        Ok(0)
    }
}
