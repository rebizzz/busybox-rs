use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct ShredApplet;

impl Applet for ShredApplet {
    fn name(&self) -> &'static str {
        "shred"
    }
    fn description(&self) -> &'static str {
        "Overwrite a file to hide its contents, and optionally delete it"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut iterations = 3usize;
        let mut zero_end = false;
        let mut remove = false;
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-u" || b == b"--remove" {
                remove = true;
            } else if b == b"-z" || b == b"--zero" {
                zero_end = true;
            } else if (b == b"-n" || b == b"--iterations") && i + 1 < args.len() {
                i += 1;
                iterations = std::str::from_utf8(args[i].as_bytes())
                    .unwrap_or("3")
                    .parse()
                    .unwrap_or(3);
            } else if b.starts_with(b"-n") && b.len() > 2 {
                iterations = std::str::from_utf8(&b[2..])
                    .unwrap_or("3")
                    .parse()
                    .unwrap_or(3);
            } else if b.starts_with(b"-") {
            } else {
                files.push(Path::new(&args[i]));
            }
            i += 1;
        }

        if files.is_empty() {
            eprintln!("shred: missing file operand");
            return Ok(1);
        }

        let mut ret = 0;
        for path in files {
            let mut file = match OpenOptions::new().write(true).open(path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("shred: {}: {}", path.display(), e);
                    ret = 1;
                    continue;
                }
            };

            let len = match file.metadata() {
                Ok(m) => m.len(),
                Err(e) => {
                    eprintln!("shred: {}: {}", path.display(), e);
                    ret = 1;
                    continue;
                }
            };

            let mut rand_state = 0x123456789abcdef0u64 ^ len;
            let chunk_size = 64 * 1024;
            let mut buf = vec![0u8; chunk_size];

            for _ in 0..iterations {
                let _ = file.seek(SeekFrom::Start(0));
                let mut written = 0;
                while written < len {
                    let to_write = std::cmp::min(chunk_size as u64, len - written) as usize;
                    for byte in &mut buf[..to_write] {
                        rand_state = rand_state.wrapping_mul(6364136223846793005).wrapping_add(1);
                        *byte = (rand_state >> 33) as u8;
                    }
                    if let Err(e) = file.write_all(&buf[..to_write]) {
                        eprintln!("shred: {}: {}", path.display(), e);
                        ret = 1;
                        break;
                    }
                    written += to_write as u64;
                }
                let _ = file.sync_all();
            }

            if zero_end {
                let _ = file.seek(SeekFrom::Start(0));
                buf.fill(0);
                let mut written = 0;
                while written < len {
                    let to_write = std::cmp::min(chunk_size as u64, len - written) as usize;
                    if let Err(e) = file.write_all(&buf[..to_write]) {
                        eprintln!("shred: {}: {}", path.display(), e);
                        ret = 1;
                        break;
                    }
                    written += to_write as u64;
                }
                let _ = file.sync_all();
            }

            drop(file);

            if remove {
                if let Err(e) = fs::remove_file(path) {
                    eprintln!("shred: {}: failed to remove: {}", path.display(), e);
                    ret = 1;
                }
            }
        }

        Ok(ret)
    }
}

