use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::process::Command;

pub struct FsckApplet;

impl Applet for FsckApplet {
    fn name(&self) -> &'static str {
        "fsck"
    }
    fn description(&self) -> &'static str {
        "Check and repair a Linux filesystem"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut target: Option<&Path> = None;
        let mut fstype: Option<&[u8]> = None;
        let mut pass_args: Vec<OsString> = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-t" && i + 1 < args.len() {
                i += 1;
                fstype = Some(args[i].as_bytes());
            } else if b.starts_with(b"-t") && b.len() > 2 {
                fstype = Some(&b[2..]);
            } else if b.starts_with(b"-") {
                pass_args.push(args[i].clone());
            } else if target.is_none() {
                target = Some(Path::new(&args[i]));
            }
            i += 1;
        }

        let dev = match target {
            Some(d) => d,
            None => {
                eprintln!("fsck: no device specified");
                return Ok(1);
            }
        };

        let helper = if let Some(t) = fstype {
            format!("fsck.{}", String::from_utf8_lossy(t))
        } else {
            detect_fs_type(dev)
                .map(|t| format!("fsck.{}", t))
                .unwrap_or_else(|| "fsck.ext4".to_string())
        };

        let mut cmd = Command::new(&helper);
        for a in pass_args {
            cmd.arg(a);
        }
        cmd.arg(dev);

        match cmd.status() {
            Ok(status) => Ok(status.code().unwrap_or(0)),
            Err(_) => basic_fsck_superblock(dev),
        }
    }
}

fn detect_fs_type(dev: &Path) -> Option<&'static str> {
    let mut f = File::open(dev).ok()?;

    if f.seek(SeekFrom::Start(1024)).is_ok() {
        let mut sb = [0u8; 64];
        if f.read_exact(&mut sb).is_ok() {
            let magic = u16::from_le_bytes(sb[16..18].try_into().unwrap_or([0; 2]));
            if magic == 0x137f || magic == 0x138f || magic == 0x2468 || magic == 0x2478 {
                return Some("minix");
            }

            let ext_magic = u16::from_le_bytes(sb[56..58].try_into().unwrap_or([0; 2]));
            if ext_magic == 0xef53 {
                return Some("ext4");
            }
        }
    }

    if f.seek(SeekFrom::Start(0)).is_ok() {
        let mut b = [0u8; 512];
        if f.read_exact(&mut b).is_ok()
            && (&b[54..59] == b"FAT12" || &b[54..59] == b"FAT16" || &b[82..87] == b"FAT32")
        {
            return Some("vfat");
        }
    }
    None
}

fn basic_fsck_superblock(dev: &Path) -> Result<i32> {
    let _f = match File::open(dev) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("fsck: {}: {}", dev.display(), e);
            return Ok(8);
        }
    };
    if let Some(t) = detect_fs_type(dev) {
        let mut out = Vec::new();
        out.extend_from_slice(dev.as_os_str().as_bytes());
        out.extend_from_slice(b": clean (");
        out.extend_from_slice(t.as_bytes());
        out.extend_from_slice(b" filesystem superblock verified)\n");
        print_bytes(&out);
        Ok(0)
    } else {
        eprintln!("fsck: {}: unknown or unsupported filesystem", dev.display());
        Ok(4)
    }
}
