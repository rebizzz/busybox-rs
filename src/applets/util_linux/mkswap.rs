use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;

pub struct MkswapApplet;
impl Applet for MkswapApplet {
    fn name(&self) -> &'static str {
        "mkswap"
    }
    fn description(&self) -> &'static str {
        "Set up a Linux swap area"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut label = [0u8; 16];
        let mut target = None;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-L" && i + 1 < args.len() {
                let lbl = args[i + 1].as_bytes();
                let len = lbl.len().min(16);
                label[..len].copy_from_slice(&lbl[..len]);
                i += 2;
                continue;
            } else if !b.starts_with(b"-") && target.is_none() {
                target = Some(args[i].clone());
            }
            i += 1;
        }

        let dev_path = match target {
            Some(t) => t,
            None => {
                eprintln!("mkswap: device or file required");
                return Ok(1);
            }
        };

        let mut f = match OpenOptions::new().read(true).write(true).open(&dev_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("mkswap: {}: {}", dev_path.to_string_lossy(), e);
                return Ok(1);
            }
        };

        let page_size = 4096usize;
        let file_len = match f.metadata() {
            Ok(m) => m.len(),
            Err(e) => {
                eprintln!("mkswap: metadata failed: {}", e);
                return Ok(1);
            }
        };

        if file_len < (page_size * 2) as u64 {
            eprintln!(
                "mkswap: error: swap area needs to be at least {} KiB",
                page_size * 2 / 1024
            );
            return Ok(1);
        }

        let num_pages = (file_len / page_size as u64) as u32;

        let _ = f.write_all(&[0u8; 1024]);

        let mut hdr = [0u8; 1024];
        hdr[0..4].copy_from_slice(&1u32.to_le_bytes());
        hdr[4..8].copy_from_slice(&(num_pages - 1).to_le_bytes());
        hdr[8..12].copy_from_slice(&0u32.to_le_bytes());

        hdr[28..44].copy_from_slice(&label);

        let _ = f.write_all(&hdr);

        let sig_pos = (page_size - 10) as u64;
        let _ = io::Seek::seek(&mut f, io::SeekFrom::Start(sig_pos));
        let _ = f.write_all(b"SWAPSPACE2");
        let _ = f.sync_all();

        println!(
            "Setting up swapspace version 1, size = {} bytes",
            file_len - page_size as u64
        );

        Ok(0)
    }
}
