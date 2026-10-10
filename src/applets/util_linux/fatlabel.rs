use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

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
