use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct FsckMinixApplet;

impl Applet for FsckMinixApplet {
    fn name(&self) -> &'static str {
        "fsck.minix"
    }
    fn description(&self) -> &'static str {
        "Check MINIX filesystem"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dev: Option<&Path> = None;
        for arg in args {
            let b = arg.as_bytes();
            if !b.is_empty() && b[0] == b'-' {
            } else if dev.is_none() {
                dev = Some(Path::new(arg));
            }
        }
        let target = match dev {
            Some(d) => d,
            None => {
                eprintln!("fsck.minix: device required");
                return Ok(1);
            }
        };

        let mut f = match File::open(target) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("fsck.minix: {}: {}", target.display(), e);
                return Ok(8);
            }
        };

        if f.seek(SeekFrom::Start(1024)).is_err() {
            eprintln!("fsck.minix: seek error");
            return Ok(8);
        }
        let mut sb = [0u8; 64];
        if f.read_exact(&mut sb).is_err() {
            eprintln!("fsck.minix: unable to read superblock");
            return Ok(8);
        }

        let magic = u16::from_le_bytes(sb[16..18].try_into().unwrap_or([0; 2]));
        let valid = magic == 0x137f || magic == 0x138f || magic == 0x2468 || magic == 0x2478;
        if !valid {
            eprintln!("fsck.minix: bad magic number in superblock");
            return Ok(4);
        }

        let ninodes = u16::from_le_bytes(sb[0..2].try_into().unwrap_or([0; 2]));
        let nzones = u16::from_le_bytes(sb[2..4].try_into().unwrap_or([0; 2]));
        let mut out = Vec::new();
        out.extend_from_slice(b"MINIX superblock verified: ");
        put_num(&mut out, ninodes as u64);
        out.extend_from_slice(b" inodes, ");
        put_num(&mut out, nzones as u64);
        out.extend_from_slice(b" zones\n");
        print_bytes(&out);
        Ok(0)
    }
}
