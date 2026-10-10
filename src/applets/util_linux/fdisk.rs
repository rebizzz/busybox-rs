use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

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
