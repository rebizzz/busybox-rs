use crate::applets::archival::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;

struct MtdInfoUser {
    typ: u8,
    _pad: [u8; 3],
    flags: u32,
    size: u32,
    erasesize: u32,
    writesize: u32,
    oobsize: u32,
    _pad2: u64,
}

#[repr(C)]
struct EraseInfoUser {
    start: u32,
    length: u32,
}

#[repr(C)]
struct Mtop {
    op: i16,
    count: i32,
}

#[repr(C)]
struct UbiAttachReq {
    ubi_num: i32,
    mtd_num: i32,
    vid_hdr_offset: i32,
    padding: [u8; 12],
}

fn ioctl_err(dev: &str, e: std::io::Error) -> i32 {
    eprintln!("{dev}: ioctl: {e} (no such device or no hardware?)");
    1
}

fn open_dev(path: &str) -> std::io::Result<File> {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
}

pub struct NanddumpApplet;
impl Applet for NanddumpApplet {
    fn name(&self) -> &'static str {
        "nanddump"
    }
    fn description(&self) -> &'static str {
        "Dump NAND flash (MEMGETINFO ioctl; graceful error without hardware)"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut dev = String::from("/dev/mtd0");
        let mut file: Option<PathBuf> = None;
        let mut i = 0;
        while i < args.len() {
            let b = ab(&args[i]);
            if b == b"-f" {
                i += 1;
                if i >= args.len() {
                    eprintln!("nanddump: -f requires an argument");
                    return Ok(1);
                }
                file = Some(PathBuf::from(&args[i]));
            } else if !b.starts_with(b"-") {
                dev = String::from_utf8_lossy(b).into_owned();
            }
            i += 1;
        }
        let f = match open_dev(&dev) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("nanddump: {dev}: {e}");
                return Ok(1);
            }
        };
        let mut info = MtdInfoUser {
            typ: 0,
            _pad: [0; 3],
            flags: 0,
            size: 0,
            erasesize: 0,
            writesize: 0,
            oobsize: 0,
            _pad2: 0,
        };
        let req = ior(b'M', 1, size_of::<MtdInfoUser>());
        let r = unsafe { libc::ioctl(f.as_raw_fd(), req, &mut info) };
        if r != 0 {
            return Ok(ioctl_err(&dev, std::io::Error::last_os_error()));
        }
        eprintln!(
            "nanddump: {dev}: size={} erasesize={}",
            info.size, info.erasesize
        );
        let mut src = &f;
        let out = std::io::stdout();
        let mut lock = out.lock();
        let mut total = 0u64;
        if let Some(fp) = file {
            let mut dst = match File::create(&fp) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("nanddump: {}: {e}", fp.display());
                    return Ok(1);
                }
            };
            match copy_stream(&mut src, &mut dst) {
                Ok(n) => total = n,
                Err(e) => {
                    eprintln!("nanddump: {e}");
                    return Ok(1);
                }
            }
        } else if copy_stream(&mut src, &mut lock).is_err() {
            return Ok(1);
        }
        eprintln!("nanddump: dumped {total} bytes");
        Ok(0)
    }
}
