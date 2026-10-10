use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct MkfsVfatApplet;

impl Applet for MkfsVfatApplet {
    fn name(&self) -> &'static str {
        "mkfs.vfat"
    }
    fn description(&self) -> &'static str {
        "Create a VFAT filesystem"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        make_fat_fs(args)
    }
}

fn make_fat_fs(args: &[OsString]) -> Result<i32> {
    let mut dev: Option<&Path> = None;
    let mut label: [u8; 11] = *b"NO NAME    ";
    let mut fat_bits: u8 = 0;

    let mut i = 0;
    while i < args.len() {
        let b = args[i].as_bytes();
        if b == b"-n" && i + 1 < args.len() {
            i += 1;
            let l = args[i].as_bytes();
            for (idx, ch) in label.iter_mut().enumerate() {
                *ch = if idx < l.len() {
                    l[idx].to_ascii_uppercase()
                } else {
                    b' '
                };
            }
        } else if b == b"-F" && i + 1 < args.len() {
            i += 1;
            match args[i].as_bytes() {
                b"12" => fat_bits = 12,
                b"16" => fat_bits = 16,
                b"32" => fat_bits = 32,
                _ => {}
            }
        } else if !b.is_empty() && b[0] == b'-' {
        } else if dev.is_none() {
            dev = Some(Path::new(&args[i]));
        }
        i += 1;
    }

    let target = match dev {
        Some(d) => d,
        None => {
            eprintln!("mkfs.vfat: device required");
            return Ok(1);
        }
    };

    let mut f = match OpenOptions::new().read(true).write(true).open(target) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("mkfs.vfat: {}: {}", target.display(), e);
            return Ok(1);
        }
    };

    let dev_size = f.metadata().map(|m| m.len()).unwrap_or(0);
    let total_sectors = if dev_size > 0 { dev_size / 512 } else { 2880 };

    if fat_bits == 0 {
        if total_sectors < 8400 {
            fat_bits = 12;
        } else if total_sectors < 66600 {
            fat_bits = 16;
        } else {
            fat_bits = 32;
        }
    }

    let mut boot = [0u8; 512];

    boot[0] = 0xEB;
    boot[1] = 0x3C;
    boot[2] = 0x90;
    boot[3..11].copy_from_slice(b"MSDOS5.0");

    boot[11..13].copy_from_slice(&512u16.to_le_bytes());

    let spc: u8 = if total_sectors < 8400 {
        1
    } else if total_sectors < 66600 {
        4
    } else {
        8
    };
    boot[13] = spc;

    let reserved: u16 = if fat_bits == 32 { 32 } else { 1 };
    boot[14..16].copy_from_slice(&reserved.to_le_bytes());

    boot[16] = 2;

    let root_entries: u16 = if fat_bits == 32 { 0 } else { 512 };
    boot[17..19].copy_from_slice(&root_entries.to_le_bytes());

    if total_sectors < 65536 && fat_bits != 32 {
        boot[19..21].copy_from_slice(&(total_sectors as u16).to_le_bytes());
    } else {
        boot[32..36].copy_from_slice(&(total_sectors as u32).to_le_bytes());
    }

    boot[21] = 0xF8;

    if fat_bits == 32 {
        let fat_size = ((total_sectors / (spc as u64) * 4).div_ceil(512)) as u32;
        boot[36..40].copy_from_slice(&fat_size.to_le_bytes());
        boot[44..48].copy_from_slice(&2u32.to_le_bytes());
        boot[66] = 0x29;
        boot[71..82].copy_from_slice(&label);
        boot[82..90].copy_from_slice(b"FAT32   ");
    } else {
        let fat_size: u16 = if fat_bits == 12 {
            9
        } else {
            ((total_sectors / (spc as u64) * 2).div_ceil(512)) as u16
        };
        boot[22..24].copy_from_slice(&fat_size.to_le_bytes());
        boot[38] = 0x29;
        boot[43..54].copy_from_slice(&label);
        let ftype = if fat_bits == 12 {
            b"FAT12   "
        } else {
            b"FAT16   "
        };
        boot[54..62].copy_from_slice(ftype);
    }

    boot[510] = 0x55;
    boot[511] = 0xAA;

    if f.seek(SeekFrom::Start(0)).is_err() || f.write_all(&boot).is_err() {
        eprintln!("mkfs.vfat: failed to write boot sector");
        return Ok(1);
    }
    let _ = f.flush();
    Ok(0)
}
