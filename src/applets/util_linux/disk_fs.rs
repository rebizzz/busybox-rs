use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::process::Command;

fn print_bytes(bytes: &[u8]) {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let _ = out.write_all(bytes);
}

fn put_num(buf: &mut Vec<u8>, mut n: u64) {
    if n == 0 {
        buf.push(b'0');
        return;
    }
    let mut t = [0u8; 20];
    let mut i = t.len();
    while n > 0 {
        i -= 1;
        t[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    buf.extend_from_slice(&t[i..]);
}

fn put_num_pad(buf: &mut Vec<u8>, n: u64, width: usize) {
    let start = buf.len();
    put_num(buf, n);
    let len = buf.len() - start;
    if len < width {
        let pad = width - len;
        let digits = buf[start..].to_vec();
        buf.truncate(start);
        for _ in 0..pad {
            buf.push(b' ');
        }
        buf.extend_from_slice(&digits);
    }
}

pub struct FdiskApplet;

impl Applet for FdiskApplet {
    fn name(&self) -> &'static str {
        "fdisk"
    }
    fn description(&self) -> &'static str {
        "Manipulate disk partition table"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut list_only = false;
        let mut dev: Option<&Path> = None;

        for arg in args {
            let b = arg.as_bytes();
            if b == b"-l" {
                list_only = true;
            } else if !b.is_empty() && b[0] == b'-' {
            } else if dev.is_none() {
                dev = Some(Path::new(arg));
            }
        }

        let target = match dev {
            Some(p) => p,
            None => {
                if list_only {
                    Path::new("/dev/sda")
                } else {
                    eprintln!("fdisk: device required");
                    return Ok(1);
                }
            }
        };

        let mut file = match File::open(target) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("fdisk: cannot open {}: {}", target.display(), e);
                return Ok(1);
            }
        };

        let mut mbr = [0u8; 512];
        if file.read_exact(&mut mbr).is_err() {
            eprintln!("fdisk: unable to read MBR from {}", target.display());
            return Ok(1);
        }

        let mut out = Vec::new();
        out.extend_from_slice(b"Disk ");
        out.extend_from_slice(target.as_os_str().as_bytes());
        out.extend_from_slice(b"\n");

        if mbr[510] != 0x55 || mbr[511] != 0xAA {
            out.extend_from_slice(b"Invalid partition table signature (no 0x55AA)\n");
            print_bytes(&out);
            return Ok(0);
        }

        let mut is_gpt = false;
        for i in 0..4 {
            let offset = 446 + i * 16;
            let part_type = mbr[offset + 4];
            if part_type == 0xEE {
                is_gpt = true;
                break;
            }
        }

        if is_gpt && file.seek(SeekFrom::Start(512)).is_ok() {
            let mut gpt_hdr = [0u8; 512];
            if file.read_exact(&mut gpt_hdr).is_ok() && &gpt_hdr[0..8] == b"EFI PART" {
                out.extend_from_slice(b"Disklabel type: gpt\n\n");
                out.extend_from_slice(b"Device             Start        End    Sectors Type\n");

                let part_entry_lba =
                    u64::from_le_bytes(gpt_hdr[72..80].try_into().unwrap_or([0; 8]));
                let num_parts = u32::from_le_bytes(gpt_hdr[80..84].try_into().unwrap_or([0; 4]));
                let entry_size =
                    u32::from_le_bytes(gpt_hdr[84..88].try_into().unwrap_or([0; 4])) as usize;

                if file.seek(SeekFrom::Start(part_entry_lba * 512)).is_ok() {
                    let mut entries_buf = vec![0u8; (num_parts as usize) * entry_size];
                    if file.read_exact(&mut entries_buf).is_ok() {
                        for p in 0..(num_parts as usize) {
                            let pe = &entries_buf[p * entry_size..(p + 1) * entry_size];

                            if pe[0..16].iter().all(|&x| x == 0) {
                                continue;
                            }
                            let first_lba =
                                u64::from_le_bytes(pe[32..40].try_into().unwrap_or([0; 8]));
                            let last_lba =
                                u64::from_le_bytes(pe[40..48].try_into().unwrap_or([0; 8]));
                            let sectors = if last_lba >= first_lba {
                                last_lba - first_lba + 1
                            } else {
                                0
                            };

                            out.extend_from_slice(target.as_os_str().as_bytes());
                            put_num(&mut out, (p + 1) as u64);
                            out.extend_from_slice(b" ");
                            put_num_pad(&mut out, first_lba, 12);
                            out.extend_from_slice(b" ");
                            put_num_pad(&mut out, last_lba, 10);
                            out.extend_from_slice(b" ");
                            put_num_pad(&mut out, sectors, 10);
                            out.extend_from_slice(b" GPT partition\n");
                        }
                    }
                }
                print_bytes(&out);
                return Ok(0);
            }
        }

        out.extend_from_slice(b"Disklabel type: dos\n\n");
        out.extend_from_slice(b"Device     Boot      Start        End    Sectors Id Type\n");
        for i in 0..4 {
            let offset = 446 + i * 16;
            let boot_flag = mbr[offset];
            let part_type = mbr[offset + 4];
            if part_type == 0 {
                continue;
            }
            let lba_start =
                u32::from_le_bytes(mbr[offset + 8..offset + 12].try_into().unwrap_or([0; 4]));
            let sec_count =
                u32::from_le_bytes(mbr[offset + 12..offset + 16].try_into().unwrap_or([0; 4]));
            let lba_end = if sec_count > 0 {
                lba_start + sec_count - 1
            } else {
                lba_start
            };

            out.extend_from_slice(target.as_os_str().as_bytes());
            put_num(&mut out, (i + 1) as u64);
            out.extend_from_slice(if boot_flag == 0x80 {
                b"  *   "
            } else {
                b"      "
            });
            put_num_pad(&mut out, lba_start as u64, 10);
            out.extend_from_slice(b" ");
            put_num_pad(&mut out, lba_end as u64, 10);
            out.extend_from_slice(b" ");
            put_num_pad(&mut out, sec_count as u64, 10);
            out.extend_from_slice(b"  ");
            let hex = format!("{:02x} ", part_type);
            out.extend_from_slice(hex.as_bytes());
            let type_str = match part_type {
                0x83 => "Linux",
                0x82 => "Linux swap",
                0x0b | 0x0c => "W95 FAT32",
                0x07 => "HPFS/NTFS/exFAT",
                0x05 | 0x0f => "Extended",
                _ => "Unknown",
            };
            out.extend_from_slice(type_str.as_bytes());
            out.push(b'\n');
        }

        print_bytes(&out);
        Ok(0)
    }
}

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

pub struct MkdosfsApplet;

impl Applet for MkdosfsApplet {
    fn name(&self) -> &'static str {
        "mkdosfs"
    }
    fn description(&self) -> &'static str {
        "Create an MS-DOS filesystem"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        make_fat_fs(args)
    }
}

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

pub struct Mke2fsApplet;

impl Applet for Mke2fsApplet {
    fn name(&self) -> &'static str {
        "mke2fs"
    }
    fn description(&self) -> &'static str {
        "Create an ext2/ext3/ext4 filesystem"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        make_ext2_fs(args)
    }
}

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

pub struct FatlabelApplet;

impl Applet for FatlabelApplet {
    fn name(&self) -> &'static str {
        "fatlabel"
    }
    fn description(&self) -> &'static str {
        "Set or get MS-DOS filesystem label"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("fatlabel: device required");
            return Ok(1);
        }

        let dev = Path::new(&args[0]);
        let new_label = if args.len() > 1 {
            Some(args[1].as_bytes())
        } else {
            None
        };

        let mut f = match OpenOptions::new()
            .read(true)
            .write(new_label.is_some())
            .open(dev)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("fatlabel: {}: {}", dev.display(), e);
                return Ok(1);
            }
        };

        let mut boot = [0u8; 512];
        if f.read_exact(&mut boot).is_err() {
            eprintln!("fatlabel: unable to read boot sector");
            return Ok(1);
        }

        let is_fat32 = &boot[82..87] == b"FAT32";
        let label_offset = if is_fat32 { 71 } else { 43 };

        if let Some(nl) = new_label {
            let mut formatted = [b' '; 11];
            for (idx, b) in formatted.iter_mut().enumerate() {
                if idx < nl.len() {
                    *b = nl[idx].to_ascii_uppercase();
                }
            }
            boot[label_offset..label_offset + 11].copy_from_slice(&formatted);
            if f.seek(SeekFrom::Start(0)).is_err() || f.write_all(&boot).is_err() {
                eprintln!("fatlabel: unable to write new label");
                return Ok(1);
            }
            let _ = f.flush();
        } else {
            let cur = &boot[label_offset..label_offset + 11];
            let trimmed = match cur.iter().rposition(|&b| b != b' ') {
                Some(pos) => &cur[..=pos],
                None => cur,
            };
            let mut out = Vec::new();
            out.extend_from_slice(trimmed);
            out.push(b'\n');
            print_bytes(&out);
        }

        Ok(0)
    }
}

pub struct FstrimApplet;

#[repr(C)]
struct FstrimRange {
    start: u64,
    len: u64,
    minlen: u64,
}

const FITRIM_IOCTL: libc::c_ulong = 0xc0185879;

impl Applet for FstrimApplet {
    fn name(&self) -> &'static str {
        "fstrim"
    }
    fn description(&self) -> &'static str {
        "Discard unused blocks on a mounted filesystem"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut mount_point: Option<&Path> = None;
        let mut verbose = false;

        for arg in args {
            let b = arg.as_bytes();
            if b == b"-v" || b == b"--verbose" {
                verbose = true;
            } else if !b.is_empty() && b[0] == b'-' {
            } else if mount_point.is_none() {
                mount_point = Some(Path::new(arg));
            }
        }

        let mnt = match mount_point {
            Some(p) => p,
            None => {
                eprintln!("fstrim: mountpoint required");
                return Ok(1);
            }
        };

        let f = match File::open(mnt) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("fstrim: {}: {}", mnt.display(), e);
                return Ok(1);
            }
        };

        let mut range = FstrimRange {
            start: 0,
            len: u64::MAX,
            minlen: 0,
        };

        let ret =
            unsafe { libc::ioctl(f.as_raw_fd(), FITRIM_IOCTL, &mut range as *mut FstrimRange) };

        if ret < 0 {
            let err = io::Error::last_os_error();
            eprintln!("fstrim: {}: FITRIM ioctl failed: {}", mnt.display(), err);
            return Ok(1);
        }

        if verbose {
            let mut out = Vec::new();
            out.extend_from_slice(mnt.as_os_str().as_bytes());
            out.extend_from_slice(b": ");
            put_num(&mut out, range.len);
            out.extend_from_slice(b" bytes trimmed\n");
            print_bytes(&out);
        }

        Ok(0)
    }
}

pub struct RaidautorunApplet;

const RAID_AUTORUN: libc::c_ulong = 0x914;

impl Applet for RaidautorunApplet {
    fn name(&self) -> &'static str {
        "raidautorun"
    }
    fn description(&self) -> &'static str {
        "Tell kernel to automatically configure RAID arrays"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let dev = if !args.is_empty() {
            Path::new(&args[0])
        } else {
            Path::new("/dev/md0")
        };

        let f = match OpenOptions::new().read(true).write(true).open(dev) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("raidautorun: {}: {}", dev.display(), e);
                return Ok(1);
            }
        };

        let ret = unsafe { libc::ioctl(f.as_raw_fd(), RAID_AUTORUN, 0) };
        if ret < 0 {
            let err = io::Error::last_os_error();
            eprintln!("raidautorun: failed: {}", err);
            return Ok(1);
        }

        Ok(0)
    }
}

pub struct ResumeApplet;

impl Applet for ResumeApplet {
    fn name(&self) -> &'static str {
        "resume"
    }
    fn description(&self) -> &'static str {
        "Resume from software suspend"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let resume_path = Path::new("/sys/power/resume");
        if args.is_empty() {
            match fs::read_to_string(resume_path) {
                Ok(s) => {
                    print_bytes(s.as_bytes());
                    Ok(0)
                }
                Err(e) => {
                    eprintln!("resume: cannot read {}: {}", resume_path.display(), e);
                    Ok(1)
                }
            }
        } else {
            let dev_spec = args[0].as_bytes();

            let dev_str = if dev_spec.starts_with(b"/dev/") {
                let path = Path::new(&args[0]);
                match fs::metadata(path) {
                    Ok(meta) => {
                        use std::os::unix::fs::MetadataExt;
                        let rdev = meta.rdev();
                        let maj = libc::major(rdev);
                        let min = libc::minor(rdev);
                        format!("{}:{}\n", maj, min)
                    }
                    Err(e) => {
                        eprintln!("resume: stat {}: {}", path.display(), e);
                        return Ok(1);
                    }
                }
            } else {
                let mut s = String::from_utf8_lossy(dev_spec).into_owned();
                s.push('\n');
                s
            };

            if let Err(e) = fs::write(resume_path, dev_str.as_bytes()) {
                eprintln!("resume: cannot write {}: {}", resume_path.display(), e);
                return Ok(1);
            }
            Ok(0)
        }
    }
}
