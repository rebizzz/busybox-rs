use crate::core::Result;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

pub fn run_pager(is_less: bool, args: &[OsString]) -> Result<i32> {
    let mut files = Vec::new();
    for arg in args {
        let b = arg.as_bytes();
        if !b.starts_with(b"-") {
            files.push(PathBuf::from(arg));
        }
    }

    let mut lines = Vec::new();
    if files.is_empty() {
        let stdin = io::stdin();
        for l in stdin.lock().lines().map_while(std::result::Result::ok) {
            lines.push(l);
        }
    } else {
        for f in &files {
            if let Ok(file) = File::open(f) {
                for l in BufReader::new(file)
                    .lines()
                    .map_while(std::result::Result::ok)
                {
                    lines.push(l);
                }
            }
        }
    }

    let is_tty = unsafe { libc::isatty(libc::STDIN_FILENO) == 1 };
    if !is_tty {
        for l in &lines {
            println!("{}", l);
        }
        return Ok(0);
    }

    let mut orig_termios = MaybeUninit::<libc::termios>::uninit();
    unsafe {
        libc::tcgetattr(libc::STDIN_FILENO, orig_termios.as_mut_ptr());
        let mut raw = orig_termios.assume_init();
        libc::cfmakeraw(&mut raw);
        libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &raw);
    }

    let term_height = 24usize;
    let mut top = 0usize;

    let display = |top: usize| {
        print!("\x1b[2J\x1b[H");
        let end = (top + term_height - 1).min(lines.len());
        for line in lines.iter().take(end).skip(top) {
            print!("{}\r\n", line);
        }
        print!(":");
        let _ = io::stdout().flush();
    };

    display(top);

    let mut stdin = io::stdin();
    let mut buf = [0u8; 4];

    while let Ok(n) = stdin.read(&mut buf) {
        if n == 0 {
            break;
        }
        match buf[0] {
            b'q' | b'Q' => break,
            b' ' => {
                if top + term_height - 1 < lines.len() {
                    top += term_height - 1;
                }
                display(top);
            }
            b'\r' | b'\n' => {
                if top + 1 < lines.len() {
                    top += 1;
                }
                display(top);
            }
            b'b' if is_less => {
                top = top.saturating_sub(term_height - 1);
                display(top);
            }
            _ => {}
        }
    }

    unsafe {
        libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, orig_termios.as_ptr());
    }
    print!("\r\x1b[K");
    let _ = io::stdout().flush();

    Ok(0)
}

// Hardware constants & structs

pub const BLKROSET: libc::c_ulong = 0x125D;
pub const BLKROGET: libc::c_ulong = 0x125E;
pub const BLKRAGET: libc::c_ulong = 0x1263;
pub const BLKRASET: libc::c_ulong = 0x1262;
pub const BLKGETSIZE: libc::c_ulong = 0x1260;
pub const BLKFLSBUF: libc::c_ulong = 0x1261;
pub const BLKSSZGET: libc::c_ulong = 0x1268;
pub const BLKGETSIZE64: libc::c_ulong = 0x80081272;
pub const BLKDISCARD: libc::c_ulong = 0x1277;
pub const FDFLUSH: libc::c_ulong = 0x024B;
pub const FDFMTTRK: libc::c_ulong = 0x0248;

#[repr(C)]
pub struct FormatDescr {
    pub device: libc::c_uint,
    pub head: libc::c_uint,
    pub track: libc::c_uint,
}

pub const I2C_SLAVE: libc::c_ulong = 0x0703;
pub const I2C_SLAVE_FORCE: libc::c_ulong = 0x0706;
pub const I2C_RDWR: libc::c_ulong = 0x0707;
pub const I2C_SMBUS: libc::c_ulong = 0x0720;

pub const I2C_SMBUS_READ: u8 = 1;
pub const I2C_SMBUS_WRITE: u8 = 0;
pub const I2C_SMBUS_QUICK: u32 = 0;
pub const I2C_SMBUS_BYTE: u32 = 1;
pub const I2C_SMBUS_BYTE_DATA: u32 = 2;

pub const I2C_M_RD: u16 = 0x0001;

#[repr(C)]
pub union I2cSmbusData {
    pub byte: u8,
    pub word: u16,
    pub block: [u8; 34],
}

#[repr(C)]
pub struct I2cSmbusIoctlData {
    pub read_write: u8,
    pub command: u8,
    pub size: u32,
    pub data: *mut I2cSmbusData,
}

#[repr(C)]
pub struct I2cMsg {
    pub addr: u16,
    pub flags: u16,
    pub len: u16,
    pub buf: *mut u8,
}

#[repr(C)]
pub struct I2cRdwrIoctlData {
    pub msgs: *mut I2cMsg,
    pub nmsgs: u32,
}

pub fn parse_u32(s: &[u8]) -> Option<u32> {
    if s.is_empty() {
        return None;
    }
    if s.starts_with(b"0x") || s.starts_with(b"0X") {
        return u32::from_str_radix(std::str::from_utf8(&s[2..]).ok()?, 16).ok();
    }
    let mut val = 0u32;
    for &b in s {
        if !b.is_ascii_digit() {
            return None;
        }
        val = val.checked_mul(10)?.checked_add((b - b'0') as u32)?;
    }
    Some(val)
}

pub fn parse_u64(s: &[u8]) -> Option<u64> {
    if s.is_empty() {
        return None;
    }
    if s.starts_with(b"0x") || s.starts_with(b"0X") {
        return u64::from_str_radix(std::str::from_utf8(&s[2..]).ok()?, 16).ok();
    }
    let mut val = 0u64;
    for &b in s {
        if !b.is_ascii_digit() {
            return None;
        }
        val = val.checked_mul(10)?.checked_add((b - b'0') as u64)?;
    }
    Some(val)
}

