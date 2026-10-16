use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct MkfsMinixApplet;

impl Applet for MkfsMinixApplet {
    fn name(&self) -> &'static str {
        "mkfs.minix"
    }
    fn description(&self) -> &'static str {
        "Make a MINIX filesystem"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dev: Option<&Path> = None;
        let mut blocks_arg: Option<u64> = None;

        for arg in args {
            let b = arg.as_bytes();
            if !b.is_empty() && b[0] == b'-' {
            } else if dev.is_none() {
                dev = Some(Path::new(arg));
            } else if blocks_arg.is_none() {
                if let Ok(s) = std::str::from_utf8(b) {
                    blocks_arg = s.parse::<u64>().ok();
                }
            }
        }

        let target = match dev {
            Some(d) => d,
            None => {
                eprintln!("mkfs.minix: device required");
                return Ok(1);
            }
        };

        let mut f = match OpenOptions::new().read(true).write(true).open(target) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("mkfs.minix: {}: {}", target.display(), e);
                return Ok(1);
            }
        };

        let total_blocks = match blocks_arg {
            Some(b) => b,
            None => {
                let meta = f.metadata().map_err(|e| crate::core::BbError::Io {
                    path: Some(target.to_path_buf()),
                    source: e,
                })?;
                let len = meta.len();
                if len == 0 {
                    1440
                } else {
                    len / 1024
                }
            }
        };

        if total_blocks < 10 {
            eprintln!("mkfs.minix: device too small");
            return Ok(1);
        }

        let ninodes: u16 = (total_blocks / 3).clamp(16, 65535) as u16;
        let imaps: u16 = ((ninodes as u32).div_ceil(8192)) as u16;
        let zmaps: u16 = ((total_blocks as u32).div_ceil(8192)) as u16;
        let first_data_zone = 2 + imaps + zmaps + ((ninodes as u32 * 32).div_ceil(1024)) as u16;

        let mut boot_and_sb = vec![0u8; 2048];

        let sb = &mut boot_and_sb[1024..];
        sb[0..2].copy_from_slice(&ninodes.to_le_bytes());
        sb[2..4].copy_from_slice(&(total_blocks as u16).to_le_bytes());
        sb[4..6].copy_from_slice(&imaps.to_le_bytes());
        sb[6..8].copy_from_slice(&zmaps.to_le_bytes());
        sb[8..10].copy_from_slice(&first_data_zone.to_le_bytes());
        sb[10..12].copy_from_slice(&0u16.to_le_bytes());
        sb[12..16].copy_from_slice(&0x10000000u32.to_le_bytes());
        sb[16..18].copy_from_slice(&0x138fu16.to_le_bytes());

        if f.seek(SeekFrom::Start(0)).is_err() || f.write_all(&boot_and_sb).is_err() {
            eprintln!("mkfs.minix: failed to write superblock");
            return Ok(1);
        }
        let _ = f.flush();

        let mut out = Vec::new();
        put_num(&mut out, ninodes as u64);
        out.extend_from_slice(b" inodes\n");
        put_num(&mut out, total_blocks);
        out.extend_from_slice(b" blocks\nFirstdatazone=");
        put_num(&mut out, first_data_zone as u64);
        out.extend_from_slice(b"\n");
        print_bytes(&out);
        Ok(0)
    }
}
