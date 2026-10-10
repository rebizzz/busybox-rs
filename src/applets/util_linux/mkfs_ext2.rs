use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct MkfsExt2Applet;

impl Applet for MkfsExt2Applet {
    fn name(&self) -> &'static str {
        "mkfs.ext2"
    }
    fn description(&self) -> &'static str {
        "Create an ext2 filesystem"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        make_ext2_fs(args)
    }
}

fn make_ext2_fs(args: &[OsString]) -> Result<i32> {
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
            eprintln!("mke2fs: device required");
            return Ok(1);
        }
    };

    let mut f = match OpenOptions::new().read(true).write(true).open(target) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("mke2fs: {}: {}", target.display(), e);
            return Ok(1);
        }
    };

    let dev_size = f.metadata().map(|m| m.len()).unwrap_or(0);
    let block_size = 1024u32;
    let total_blocks = match blocks_arg {
        Some(b) => b as u32,
        None => {
            if dev_size > 0 {
                (dev_size / (block_size as u64)) as u32
            } else {
                4096
            }
        }
    };

    if total_blocks < 100 {
        eprintln!("mke2fs: device too small");
        return Ok(1);
    }

    let inodes_count = (total_blocks / 4).max(16);
    let blocks_per_group = 8192u32;
    let inodes_per_group = inodes_count;

    let mut sb = [0u8; 1024];
    sb[0..4].copy_from_slice(&inodes_count.to_le_bytes());
    sb[4..8].copy_from_slice(&total_blocks.to_le_bytes());
    sb[8..12].copy_from_slice(&(total_blocks * 5 / 100).to_le_bytes());
    sb[12..16].copy_from_slice(&(total_blocks - 10).to_le_bytes());
    sb[16..20].copy_from_slice(&(inodes_count - 11).to_le_bytes());
    sb[20..24].copy_from_slice(&1u32.to_le_bytes());
    sb[24..28].copy_from_slice(&0u32.to_le_bytes());
    sb[32..36].copy_from_slice(&blocks_per_group.to_le_bytes());
    sb[40..44].copy_from_slice(&inodes_per_group.to_le_bytes());
    sb[56..58].copy_from_slice(&0xEF53u16.to_le_bytes());
    sb[58..60].copy_from_slice(&1u16.to_le_bytes());
    sb[60..62].copy_from_slice(&1u16.to_le_bytes());
    sb[76..80].copy_from_slice(&0u32.to_le_bytes());
    sb[84..88].copy_from_slice(&11u32.to_le_bytes());
    sb[88..90].copy_from_slice(&128u16.to_le_bytes());

    if f.seek(SeekFrom::Start(1024)).is_err() || f.write_all(&sb).is_err() {
        eprintln!("mke2fs: failed to write superblock");
        return Ok(1);
    }
    let _ = f.flush();

    let mut out = Vec::new();
    out.extend_from_slice(b"Creating filesystem with ");
    put_num(&mut out, total_blocks as u64);
    out.extend_from_slice(b" 1k blocks and ");
    put_num(&mut out, inodes_count as u64);
    out.extend_from_slice(b" inodes\n");
    print_bytes(&out);
    Ok(0)
}
