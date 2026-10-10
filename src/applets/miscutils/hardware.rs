use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

const BLKROSET: libc::c_ulong = 0x125D;
const BLKROGET: libc::c_ulong = 0x125E;
const BLKRAGET: libc::c_ulong = 0x1263;
const BLKRASET: libc::c_ulong = 0x1262;
const BLKGETSIZE: libc::c_ulong = 0x1260;
const BLKFLSBUF: libc::c_ulong = 0x1261;
const BLKSSZGET: libc::c_ulong = 0x1268;
const BLKGETSIZE64: libc::c_ulong = 0x80081272;
const BLKDISCARD: libc::c_ulong = 0x1277;
const FDFLUSH: libc::c_ulong = 0x024B;
const FDFMTTRK: libc::c_ulong = 0x0248;

#[repr(C)]
struct FormatDescr {
    device: libc::c_uint,
    head: libc::c_uint,
    track: libc::c_uint,
}

const I2C_SLAVE: libc::c_ulong = 0x0703;
const I2C_SLAVE_FORCE: libc::c_ulong = 0x0706;
const I2C_RDWR: libc::c_ulong = 0x0707;
const I2C_SMBUS: libc::c_ulong = 0x0720;

const I2C_SMBUS_READ: u8 = 1;
const I2C_SMBUS_WRITE: u8 = 0;
const I2C_SMBUS_QUICK: u32 = 0;
const I2C_SMBUS_BYTE: u32 = 1;
const I2C_SMBUS_BYTE_DATA: u32 = 2;

const I2C_M_RD: u16 = 0x0001;

#[repr(C)]
union I2cSmbusData {
    byte: u8,
    word: u16,
    block: [u8; 34],
}

#[repr(C)]
struct I2cSmbusIoctlData {
    read_write: u8,
    command: u8,
    size: u32,
    data: *mut I2cSmbusData,
}

#[repr(C)]
struct I2cMsg {
    addr: u16,
    flags: u16,
    len: u16,
    buf: *mut u8,
}

#[repr(C)]
struct I2cRdwrIoctlData {
    msgs: *mut I2cMsg,
    nmsgs: u32,
}

fn parse_u32(s: &[u8]) -> Option<u32> {
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

fn parse_u64(s: &[u8]) -> Option<u64> {
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

pub struct FdflushApplet;

impl Applet for FdflushApplet {
    fn name(&self) -> &'static str {
        "fdflush"
    }

    fn description(&self) -> &'static str {
        "Flush floppy disk buffer cache"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let dev = if args.len() > 1 {
            &args[1]
        } else {
            eprintln!("Usage: fdflush DEVICE");
            return Ok(1);
        };

        let path = Path::new(dev);
        let f = match OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(path)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("fdflush: {}: {}", path.display(), e);
                return Ok(1);
            }
        };

        let fd = f.as_raw_fd();
        let ret = unsafe { libc::ioctl(fd, FDFLUSH, 0 as libc::c_ulong) };
        if ret < 0 {
            let ret2 = unsafe { libc::ioctl(fd, BLKFLSBUF, 0 as libc::c_ulong) };
            if ret2 < 0 {
                eprintln!(
                    "fdflush: failed on {}: {}",
                    path.display(),
                    io::Error::last_os_error()
                );
                return Ok(1);
            }
        }
        Ok(0)
    }
}

pub struct FdformatApplet;

impl Applet for FdformatApplet {
    fn name(&self) -> &'static str {
        "fdformat"
    }

    fn description(&self) -> &'static str {
        "Format a floppy disk"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut no_verify = false;
        let mut dev = None;
        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-n" {
                no_verify = true;
            } else if !b.starts_with(b"-") {
                dev = Some(&args[i]);
            }
            i += 1;
        }

        let dev_path = match dev {
            Some(d) => Path::new(d),
            None => {
                eprintln!("Usage: fdformat [-n] DEVICE");
                return Ok(1);
            }
        };

        let f = match OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NDELAY)
            .open(dev_path)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("fdformat: {}: {}", dev_path.display(), e);
                return Ok(1);
            }
        };

        print!("Formatting... ");
        let _ = io::stdout().flush();
        for track in 0..80 {
            for head in 0..2 {
                let mut descr = FormatDescr {
                    device: 0,
                    head,
                    track,
                };
                let ret = unsafe {
                    libc::ioctl(
                        f.as_raw_fd(),
                        FDFMTTRK,
                        &mut descr as *mut FormatDescr as *mut libc::c_void,
                    )
                };
                if ret < 0 {
                    println!("failed");
                    eprintln!(
                        "fdformat: format track {} head {} failed: {}",
                        track,
                        head,
                        io::Error::last_os_error()
                    );
                    return Ok(1);
                }
            }
        }
        println!("done");

        if !no_verify {
            print!("Verifying... ");
            let _ = io::stdout().flush();
            let mut f_read = match File::open(dev_path) {
                Ok(fr) => fr,
                Err(e) => {
                    eprintln!("fdformat: verify open failed: {}", e);
                    return Ok(1);
                }
            };
            let mut buf = [0u8; 1024];
            while let Ok(n) = f_read.read(&mut buf) {
                if n == 0 {
                    break;
                }
            }
            println!("done");
        }

        Ok(0)
    }
}

pub struct BlockdevApplet;

impl Applet for BlockdevApplet {
    fn name(&self) -> &'static str {
        "blockdev"
    }

    fn description(&self) -> &'static str {
        "Control block devices"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 3 {
            eprintln!("Usage: blockdev --setro|--setrw|--getro|--getss|--getsize|--getsize64|--getra|--setra N|--flushbufs DEVICE");
            return Ok(1);
        }

        let cmd = args[1].as_bytes();
        let (dev_arg, set_val) = if cmd == b"--setra" {
            if args.len() < 4 {
                eprintln!("Usage: blockdev --setra N DEVICE");
                return Ok(1);
            }
            (&args[3], parse_u32(args[2].as_bytes()).unwrap_or(0))
        } else {
            (&args[2], 0)
        };

        let path = Path::new(dev_arg);
        let f = match OpenOptions::new().read(true).write(true).open(path) {
            Ok(f) => f,
            Err(_) => match OpenOptions::new().read(true).open(path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("blockdev: {}: {}", path.display(), e);
                    return Ok(1);
                }
            },
        };

        let fd = f.as_raw_fd();
        match cmd {
            b"--setro" => {
                let val: libc::c_int = 1;
                let ret = unsafe {
                    libc::ioctl(
                        fd,
                        BLKROSET,
                        &val as *const libc::c_int as *const libc::c_void,
                    )
                };
                if ret < 0 {
                    eprintln!("blockdev: ioctl failed: {}", io::Error::last_os_error());
                    return Ok(1);
                }
            }
            b"--setrw" => {
                let val: libc::c_int = 0;
                let ret = unsafe {
                    libc::ioctl(
                        fd,
                        BLKROSET,
                        &val as *const libc::c_int as *const libc::c_void,
                    )
                };
                if ret < 0 {
                    eprintln!("blockdev: ioctl failed: {}", io::Error::last_os_error());
                    return Ok(1);
                }
            }
            b"--getro" => {
                let mut val: libc::c_int = 0;
                let ret = unsafe {
                    libc::ioctl(
                        fd,
                        BLKROGET,
                        &mut val as *mut libc::c_int as *mut libc::c_void,
                    )
                };
                if ret < 0 {
                    eprintln!("blockdev: ioctl failed: {}", io::Error::last_os_error());
                    return Ok(1);
                }
                println!("{}", val);
            }
            b"--getss" => {
                let mut val: libc::c_int = 0;
                let ret = unsafe {
                    libc::ioctl(
                        fd,
                        BLKSSZGET,
                        &mut val as *mut libc::c_int as *mut libc::c_void,
                    )
                };
                if ret < 0 {
                    eprintln!("blockdev: ioctl failed: {}", io::Error::last_os_error());
                    return Ok(1);
                }
                println!("{}", val);
            }
            b"--getsize" => {
                let mut val: libc::c_ulong = 0;
                let ret = unsafe {
                    libc::ioctl(
                        fd,
                        BLKGETSIZE,
                        &mut val as *mut libc::c_ulong as *mut libc::c_void,
                    )
                };
                if ret < 0 {
                    eprintln!("blockdev: ioctl failed: {}", io::Error::last_os_error());
                    return Ok(1);
                }
                println!("{}", val);
            }
            b"--getsize64" => {
                let mut val: u64 = 0;
                let ret = unsafe {
                    libc::ioctl(fd, BLKGETSIZE64, &mut val as *mut u64 as *mut libc::c_void)
                };
                if ret < 0 {
                    eprintln!("blockdev: ioctl failed: {}", io::Error::last_os_error());
                    return Ok(1);
                }
                println!("{}", val);
            }
            b"--getra" => {
                let mut val: libc::c_long = 0;
                let ret = unsafe {
                    libc::ioctl(
                        fd,
                        BLKRAGET,
                        &mut val as *mut libc::c_long as *mut libc::c_void,
                    )
                };
                if ret < 0 {
                    eprintln!("blockdev: ioctl failed: {}", io::Error::last_os_error());
                    return Ok(1);
                }
                println!("{}", val);
            }
            b"--setra" => {
                let val = set_val as libc::c_ulong;
                let ret = unsafe { libc::ioctl(fd, BLKRASET, val) };
                if ret < 0 {
                    eprintln!("blockdev: ioctl failed: {}", io::Error::last_os_error());
                    return Ok(1);
                }
            }
            b"--flushbufs" => {
                let ret = unsafe { libc::ioctl(fd, BLKFLSBUF, 0 as libc::c_ulong) };
                if ret < 0 {
                    eprintln!("blockdev: ioctl failed: {}", io::Error::last_os_error());
                    return Ok(1);
                }
            }
            _ => {
                eprintln!("blockdev: unknown command '{}'", args[1].to_string_lossy());
                return Ok(1);
            }
        }
        Ok(0)
    }
}

pub struct BlkdiscardApplet;

impl Applet for BlkdiscardApplet {
    fn name(&self) -> &'static str {
        "blkdiscard"
    }

    fn description(&self) -> &'static str {
        "Discard sectors on a block device"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("Usage: blkdiscard [-o offset] [-l length] DEVICE");
            return Ok(1);
        }

        let mut offset = 0u64;
        let mut length = 0u64;
        let mut dev = None;

        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-o" && i + 1 < args.len() {
                i += 1;
                offset = parse_u64(args[i].as_bytes()).unwrap_or(0);
            } else if b == b"-l" && i + 1 < args.len() {
                i += 1;
                length = parse_u64(args[i].as_bytes()).unwrap_or(0);
            } else if !b.starts_with(b"-") {
                dev = Some(&args[i]);
            }
            i += 1;
        }

        let dev_path = match dev {
            Some(d) => Path::new(d),
            None => {
                eprintln!("blkdiscard: no device specified");
                return Ok(1);
            }
        };

        let f = match OpenOptions::new().read(true).write(true).open(dev_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("blkdiscard: {}: {}", dev_path.display(), e);
                return Ok(1);
            }
        };

        if length == 0 {
            let mut sz: u64 = 0;
            let ret = unsafe {
                libc::ioctl(
                    f.as_raw_fd(),
                    BLKGETSIZE64,
                    &mut sz as *mut u64 as *mut libc::c_void,
                )
            };
            if ret == 0 && sz > offset {
                length = sz - offset;
            }
        }

        let range: [u64; 2] = [offset, length];
        let ret = unsafe {
            libc::ioctl(
                f.as_raw_fd(),
                BLKDISCARD,
                range.as_ptr() as *const libc::c_void,
            )
        };
        if ret < 0 {
            eprintln!(
                "blkdiscard: BLKDISCARD failed on {}: {}",
                dev_path.display(),
                io::Error::last_os_error()
            );
            return Ok(1);
        }
        Ok(0)
    }
}

pub struct FreeramdiskApplet;

impl Applet for FreeramdiskApplet {
    fn name(&self) -> &'static str {
        "freeramdisk"
    }

    fn description(&self) -> &'static str {
        "Free memory held by a ramdisk"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("Usage: freeramdisk DEVICE");
            return Ok(1);
        }

        let path = Path::new(&args[1]);
        let f = match OpenOptions::new().write(true).open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("freeramdisk: {}: {}", path.display(), e);
                return Ok(1);
            }
        };

        let ret = unsafe { libc::ioctl(f.as_raw_fd(), BLKFLSBUF, 0 as libc::c_ulong) };
        if ret < 0 {
            eprintln!(
                "freeramdisk: BLKFLSBUF failed: {}",
                io::Error::last_os_error()
            );
            return Ok(1);
        }
        Ok(0)
    }
}

pub struct ReadaheadApplet;

impl Applet for ReadaheadApplet {
    fn name(&self) -> &'static str {
        "readahead"
    }

    fn description(&self) -> &'static str {
        "Preload files into page cache"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("Usage: readahead FILE...");
            return Ok(1);
        }

        let mut ret_code = 0;
        for arg in &args[1..] {
            let path = Path::new(arg);
            let f = match File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("readahead: {}: {}", path.display(), e);
                    ret_code = 1;
                    continue;
                }
            };

            let len = f.metadata().map(|m| m.len()).unwrap_or(0);
            let fd = f.as_raw_fd();
            let ret = unsafe {
                libc::posix_fadvise(fd, 0, len as libc::off_t, libc::POSIX_FADV_WILLNEED)
            };
            if ret != 0 {
                unsafe {
                    libc::readahead(fd, 0, len as libc::size_t);
                }
            }
        }
        Ok(ret_code)
    }
}

pub struct FsyncApplet;

impl Applet for FsyncApplet {
    fn name(&self) -> &'static str {
        "fsync"
    }

    fn description(&self) -> &'static str {
        "Synchronize file state with storage"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("Usage: fsync [-d] FILE...");
            return Ok(1);
        }

        let mut data_only = false;
        let mut ret_code = 0;

        let mut files = Vec::new();
        for arg in &args[1..] {
            let b = arg.as_bytes();
            if b == b"-d" {
                data_only = true;
            } else {
                files.push(arg);
            }
        }

        for file_arg in files {
            let path = Path::new(file_arg);
            let f = match File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("fsync: {}: {}", path.display(), e);
                    ret_code = 1;
                    continue;
                }
            };

            let fd = f.as_raw_fd();
            let ret = if data_only {
                unsafe { libc::fdatasync(fd) }
            } else {
                unsafe { libc::fsync(fd) }
            };
            if ret < 0 {
                eprintln!("fsync: {}: {}", path.display(), io::Error::last_os_error());
                ret_code = 1;
            }
        }
        Ok(ret_code)
    }
}

pub struct DevmemApplet;

impl Applet for DevmemApplet {
    fn name(&self) -> &'static str {
        "devmem"
    }

    fn description(&self) -> &'static str {
        "Read/write physical memory"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("Usage: devmem ADDRESS [WIDTH [VALUE]]");
            return Ok(1);
        }

        let addr = match parse_u64(args[1].as_bytes()) {
            Some(a) => a,
            None => {
                eprintln!("devmem: invalid address");
                return Ok(1);
            }
        };

        let width = if args.len() > 2 {
            let w = args[2].as_bytes();
            match w {
                b"b" | b"8" => 8,
                b"h" | b"16" => 16,
                b"w" | b"32" => 32,
                b"q" | b"64" => 64,
                _ => 32,
            }
        } else {
            32
        };

        let write_val = if args.len() > 3 {
            parse_u64(args[3].as_bytes())
        } else {
            None
        };

        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as u64;
        let page_base = addr & !(page_size - 1);
        let page_offset = (addr - page_base) as usize;

        let open_flags = if write_val.is_some() {
            libc::O_RDWR | libc::O_SYNC
        } else {
            libc::O_RDONLY | libc::O_SYNC
        };

        let fd = unsafe { libc::open(c"/dev/mem".as_ptr(), open_flags) };
        if fd < 0 {
            eprintln!("devmem: open(/dev/mem): {}", io::Error::last_os_error());
            return Ok(1);
        }

        let prot = if write_val.is_some() {
            libc::PROT_READ | libc::PROT_WRITE
        } else {
            libc::PROT_READ
        };

        let map = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                page_size as usize,
                prot,
                libc::MAP_SHARED,
                fd,
                page_base as libc::off_t,
            )
        };
        unsafe { libc::close(fd) };

        if map == libc::MAP_FAILED {
            eprintln!("devmem: mmap: {}", io::Error::last_os_error());
            return Ok(1);
        }

        let ptr = (map as usize + page_offset) as *mut u8;

        if let Some(val) = write_val {
            unsafe {
                match width {
                    8 => std::ptr::write_volatile(ptr, val as u8),
                    16 => std::ptr::write_volatile(ptr as *mut u16, val as u16),
                    32 => std::ptr::write_volatile(ptr as *mut u32, val as u32),
                    64 => std::ptr::write_volatile(ptr as *mut u64, val),
                    _ => {}
                }
            }
        } else {
            unsafe {
                match width {
                    8 => {
                        let v = std::ptr::read_volatile(ptr);
                        println!("0x{:02X}", v);
                    }
                    16 => {
                        let v = std::ptr::read_volatile(ptr as *const u16);
                        println!("0x{:04X}", v);
                    }
                    32 => {
                        let v = std::ptr::read_volatile(ptr as *const u32);
                        println!("0x{:08X}", v);
                    }
                    64 => {
                        let v = std::ptr::read_volatile(ptr as *const u64);
                        println!("0x{:016X}", v);
                    }
                    _ => {}
                }
            }
        }

        unsafe {
            libc::munmap(map, page_size as usize);
        }
        Ok(0)
    }
}

pub struct I2cdetectApplet;

impl Applet for I2cdetectApplet {
    fn name(&self) -> &'static str {
        "i2cdetect"
    }

    fn description(&self) -> &'static str {
        "Detect I2C chips"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut list_busses = false;
        let mut bus_num = None;

        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-l" {
                list_busses = true;
            } else if b == b"-y" || b == b"-q" || b == b"-r" {
            } else if !b.starts_with(b"-") {
                bus_num = parse_u32(b);
            }
            i += 1;
        }

        if list_busses || (bus_num.is_none() && args.len() <= 1) {
            if let Ok(entries) = fs::read_dir("/sys/class/i2c-dev") {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    let name_path = entry.path().join("name");
                    let desc = fs::read_to_string(&name_path).unwrap_or_else(|_| "unknown".into());
                    println!("{}\t{:<20}\tI2C adapter", name_str, desc.trim());
                }
            }
            return Ok(0);
        }

        let bus = match bus_num {
            Some(b) => b,
            None => {
                eprintln!("Usage: i2cdetect [-y] [-a] BUS-NUMBER");
                return Ok(1);
            }
        };

        let dev_path = format!("/dev/i2c-{}", bus);
        let f = match OpenOptions::new().read(true).write(true).open(&dev_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("i2cdetect: {}: {}", dev_path, e);
                return Ok(1);
            }
        };

        println!("     0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f");
        for row in (0x00..=0x70).step_by(16) {
            print!("{:02x}:", row);
            for col in 0..16 {
                let addr = row + col;
                if !(0x03..=0x77).contains(&addr) {
                    print!("   ");
                    continue;
                }
                let mut data = I2cSmbusData { byte: 0 };
                let mut ioctl_data = I2cSmbusIoctlData {
                    read_write: I2C_SMBUS_READ,
                    command: 0,
                    size: I2C_SMBUS_QUICK,
                    data: &mut data,
                };
                let ret = unsafe {
                    libc::ioctl(f.as_raw_fd(), I2C_SLAVE, addr as libc::c_ulong);
                    libc::ioctl(
                        f.as_raw_fd(),
                        I2C_SMBUS,
                        &mut ioctl_data as *mut I2cSmbusIoctlData as *mut libc::c_void,
                    )
                };
                if ret >= 0 {
                    print!(" {:02x}", addr);
                } else {
                    print!(" --");
                }
            }
            println!();
        }
        Ok(0)
    }
}

pub struct I2cdumpApplet;

impl Applet for I2cdumpApplet {
    fn name(&self) -> &'static str {
        "i2cdump"
    }

    fn description(&self) -> &'static str {
        "Examine I2C registers"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut bus_num = None;
        let mut chip_addr = None;

        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-y" || b == b"-f" {
            } else if !b.starts_with(b"-") {
                if bus_num.is_none() {
                    bus_num = parse_u32(b);
                } else if chip_addr.is_none() {
                    chip_addr = parse_u32(b);
                }
            }
            i += 1;
        }

        let (bus, addr) = match (bus_num, chip_addr) {
            (Some(b), Some(a)) => (b, a),
            _ => {
                eprintln!("Usage: i2cdump [-y] BUS CHIP-ADDRESS");
                return Ok(1);
            }
        };

        let dev_path = format!("/dev/i2c-{}", bus);
        let f = match OpenOptions::new().read(true).write(true).open(&dev_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("i2cdump: {}: {}", dev_path, e);
                return Ok(1);
            }
        };

        let fd = f.as_raw_fd();
        let ret = unsafe { libc::ioctl(fd, I2C_SLAVE_FORCE, addr as libc::c_ulong) };
        if ret < 0 {
            eprintln!(
                "i2cdump: failed setting slave addr: {}",
                io::Error::last_os_error()
            );
            return Ok(1);
        }

        println!("     0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f    0123456789abcdef");
        for row in (0x00..=0xF0).step_by(16) {
            print!("{:02x}:", row);
            let mut ascii = [b'.'; 16];
            for (col, ascii_ch) in ascii.iter_mut().enumerate() {
                let cmd = (row + col as u32) as u8;
                let mut data = I2cSmbusData { byte: 0 };
                let mut ioctl_data = I2cSmbusIoctlData {
                    read_write: I2C_SMBUS_READ,
                    command: cmd,
                    size: I2C_SMBUS_BYTE_DATA,
                    data: &mut data,
                };
                let res = unsafe {
                    libc::ioctl(
                        fd,
                        I2C_SMBUS,
                        &mut ioctl_data as *mut I2cSmbusIoctlData as *mut libc::c_void,
                    )
                };
                if res >= 0 {
                    let b = unsafe { data.byte };
                    print!(" {:02x}", b);
                    if b.is_ascii_graphic() {
                        *ascii_ch = b;
                    }
                } else {
                    print!(" XX");
                    *ascii_ch = b'X';
                }
            }
            print!("    ");
            let _ = io::stdout().write_all(&ascii);
            println!();
        }
        Ok(0)
    }
}

pub struct I2cgetApplet;

impl Applet for I2cgetApplet {
    fn name(&self) -> &'static str {
        "i2cget"
    }

    fn description(&self) -> &'static str {
        "Read from I2C/SMBus chip registers"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut bus_num = None;
        let mut chip_addr = None;
        let mut reg = None;

        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-y" || b == b"-f" {
            } else if !b.starts_with(b"-") {
                if bus_num.is_none() {
                    bus_num = parse_u32(b);
                } else if chip_addr.is_none() {
                    chip_addr = parse_u32(b);
                } else if reg.is_none() {
                    reg = parse_u32(b);
                }
            }
            i += 1;
        }

        let (bus, addr) = match (bus_num, chip_addr) {
            (Some(b), Some(a)) => (b, a),
            _ => {
                eprintln!("Usage: i2cget [-y] BUS CHIP-ADDRESS [DATA-ADDRESS]");
                return Ok(1);
            }
        };

        let dev_path = format!("/dev/i2c-{}", bus);
        let f = match OpenOptions::new().read(true).write(true).open(&dev_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("i2cget: {}: {}", dev_path, e);
                return Ok(1);
            }
        };

        let fd = f.as_raw_fd();
        let ret = unsafe { libc::ioctl(fd, I2C_SLAVE_FORCE, addr as libc::c_ulong) };
        if ret < 0 {
            eprintln!(
                "i2cget: set slave address failed: {}",
                io::Error::last_os_error()
            );
            return Ok(1);
        }

        let mut data = I2cSmbusData { byte: 0 };
        let (cmd, size) = match reg {
            Some(r) => (r as u8, I2C_SMBUS_BYTE_DATA),
            None => (0, I2C_SMBUS_BYTE),
        };

        let mut ioctl_data = I2cSmbusIoctlData {
            read_write: I2C_SMBUS_READ,
            command: cmd,
            size,
            data: &mut data,
        };

        let res = unsafe {
            libc::ioctl(
                fd,
                I2C_SMBUS,
                &mut ioctl_data as *mut I2cSmbusIoctlData as *mut libc::c_void,
            )
        };
        if res < 0 {
            eprintln!("i2cget: read failed: {}", io::Error::last_os_error());
            return Ok(1);
        }

        println!("0x{:02x}", unsafe { data.byte });
        Ok(0)
    }
}

pub struct I2csetApplet;

impl Applet for I2csetApplet {
    fn name(&self) -> &'static str {
        "i2cset"
    }

    fn description(&self) -> &'static str {
        "Set I2C registers"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut bus_num = None;
        let mut chip_addr = None;
        let mut reg = None;
        let mut value = None;

        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-y" || b == b"-f" || b == b"-m" || b == b"-r" {
            } else if !b.starts_with(b"-") {
                if bus_num.is_none() {
                    bus_num = parse_u32(b);
                } else if chip_addr.is_none() {
                    chip_addr = parse_u32(b);
                } else if reg.is_none() {
                    reg = parse_u32(b);
                } else if value.is_none() {
                    value = parse_u32(b);
                }
            }
            i += 1;
        }

        let (bus, addr, data_addr, val) = match (bus_num, chip_addr, reg, value) {
            (Some(b), Some(a), Some(r), Some(v)) => (b, a, r, v),
            (Some(b), Some(a), Some(v), None) => (b, a, 0, v),
            _ => {
                eprintln!("Usage: i2cset [-y] BUS CHIP-ADDRESS DATA-ADDRESS [VALUE]");
                return Ok(1);
            }
        };

        let dev_path = format!("/dev/i2c-{}", bus);
        let f = match OpenOptions::new().read(true).write(true).open(&dev_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("i2cset: {}: {}", dev_path, e);
                return Ok(1);
            }
        };

        let fd = f.as_raw_fd();
        let ret = unsafe { libc::ioctl(fd, I2C_SLAVE_FORCE, addr as libc::c_ulong) };
        if ret < 0 {
            eprintln!(
                "i2cset: set slave address failed: {}",
                io::Error::last_os_error()
            );
            return Ok(1);
        }

        let mut data = I2cSmbusData { byte: val as u8 };
        let mut ioctl_data = I2cSmbusIoctlData {
            read_write: I2C_SMBUS_WRITE,
            command: data_addr as u8,
            size: I2C_SMBUS_BYTE_DATA,
            data: &mut data,
        };

        let res = unsafe {
            libc::ioctl(
                fd,
                I2C_SMBUS,
                &mut ioctl_data as *mut I2cSmbusIoctlData as *mut libc::c_void,
            )
        };
        if res < 0 {
            eprintln!("i2cset: write failed: {}", io::Error::last_os_error());
            return Ok(1);
        }
        Ok(0)
    }
}

pub struct I2ctransferApplet;

impl Applet for I2ctransferApplet {
    fn name(&self) -> &'static str {
        "i2ctransfer"
    }

    fn description(&self) -> &'static str {
        "Send user-defined I2C messages in one transfer"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut bus_num = None;
        let mut msgs_args = Vec::new();

        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-y" || b == b"-f" || b == b"-a" || b == b"-v" {
            } else if !b.starts_with(b"-") {
                if bus_num.is_none() {
                    bus_num = parse_u32(b);
                } else {
                    msgs_args.push(&args[i]);
                }
            }
            i += 1;
        }

        let bus = match bus_num {
            Some(b) => b,
            None => {
                eprintln!("Usage: i2ctransfer [-y] BUS {{r|w}}LEN[@ADDR] [DATA...]");
                return Ok(1);
            }
        };

        let dev_path = format!("/dev/i2c-{}", bus);
        let f = match OpenOptions::new().read(true).write(true).open(&dev_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("i2ctransfer: {}: {}", dev_path, e);
                return Ok(1);
            }
        };

        let mut i2c_msgs = Vec::new();
        let mut buffers = Vec::new();

        let mut idx = 0;
        while idx < msgs_args.len() {
            let desc = msgs_args[idx].as_bytes();
            idx += 1;
            let is_read = desc.starts_with(b"r") || desc.starts_with(b"R");
            let is_write = desc.starts_with(b"w") || desc.starts_with(b"W");
            if !is_read && !is_write {
                continue;
            }

            let rest = &desc[1..];
            let mut at_parts = rest.split(|&b| b == b'@');
            let len_part = at_parts.next().unwrap_or(b"");
            let addr_part = at_parts.next().unwrap_or(b"0");

            let len = parse_u32(len_part).unwrap_or(0) as usize;
            let addr = parse_u32(addr_part).unwrap_or(0) as u16;

            let mut buf = vec![0u8; len];
            if is_write {
                for b_byte in buf.iter_mut() {
                    if idx < msgs_args.len()
                        && !msgs_args[idx].as_bytes().starts_with(b"r")
                        && !msgs_args[idx].as_bytes().starts_with(b"w")
                    {
                        if let Some(v) = parse_u32(msgs_args[idx].as_bytes()) {
                            *b_byte = v as u8;
                        }
                        idx += 1;
                    }
                }
            }
            buffers.push((is_read, buf, addr));
        }

        for (is_read, buf, addr) in &mut buffers {
            i2c_msgs.push(I2cMsg {
                addr: *addr,
                flags: if *is_read { I2C_M_RD } else { 0 },
                len: buf.len() as u16,
                buf: buf.as_mut_ptr(),
            });
        }

        let mut rdwr = I2cRdwrIoctlData {
            msgs: i2c_msgs.as_mut_ptr(),
            nmsgs: i2c_msgs.len() as u32,
        };

        let ret = unsafe {
            libc::ioctl(
                f.as_raw_fd(),
                I2C_RDWR,
                &mut rdwr as *mut I2cRdwrIoctlData as *mut libc::c_void,
            )
        };
        if ret < 0 {
            eprintln!(
                "i2ctransfer: I2C_RDWR failed: {}",
                io::Error::last_os_error()
            );
            return Ok(1);
        }

        for (is_read, buf, _) in &buffers {
            if *is_read {
                for &b in buf {
                    print!("0x{:02x} ", b);
                }
                println!();
            }
        }
        Ok(0)
    }
}

pub struct LspciApplet;

impl Applet for LspciApplet {
    fn name(&self) -> &'static str {
        "lspci"
    }

    fn description(&self) -> &'static str {
        "List all PCI devices"
    }

    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let pci_dir = Path::new("/sys/bus/pci/devices");
        let entries = match fs::read_dir(pci_dir) {
            Ok(e) => e,
            Err(err) => {
                eprintln!("lspci: /sys/bus/pci/devices: {}", err);
                return Ok(1);
            }
        };

        let mut dev_list = Vec::new();
        for entry in entries.flatten() {
            let slot_name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path();

            let vendor = fs::read_to_string(path.join("vendor"))
                .unwrap_or_default()
                .trim()
                .strip_prefix("0x")
                .unwrap_or("")
                .to_string();
            let device = fs::read_to_string(path.join("device"))
                .unwrap_or_default()
                .trim()
                .strip_prefix("0x")
                .unwrap_or("")
                .to_string();
            let class_code = fs::read_to_string(path.join("class"))
                .unwrap_or_default()
                .trim()
                .strip_prefix("0x")
                .unwrap_or("")
                .to_string();

            let class_desc = match class_code.get(..4) {
                Some("0100") => "SCSI storage controller",
                Some("0101") => "IDE interface",
                Some("0106") => "SATA controller",
                Some("0108") => "Non-Volatile memory controller",
                Some("0200") => "Ethernet controller",
                Some("0280") => "Network controller",
                Some("0300") => "VGA compatible controller",
                Some("0401") => "Multimedia audio controller",
                Some("0403") => "Audio device",
                Some("0600") => "Host bridge",
                Some("0601") => "ISA bridge",
                Some("0604") => "PCI bridge",
                Some("0c03") => "USB controller",
                Some("0c05") => "SMBus",
                _ => "Device",
            };

            dev_list.push((slot_name, class_desc, vendor, device));
        }

        dev_list.sort_by(|a, b| a.0.cmp(&b.0));
        for (slot, class_desc, vendor, device) in dev_list {
            let short_slot = slot.strip_prefix("0000:").unwrap_or(&slot);
            println!("{}: {} [{}:{}]", short_slot, class_desc, vendor, device);
        }
        Ok(0)
    }
}

pub struct LsusbApplet;

impl Applet for LsusbApplet {
    fn name(&self) -> &'static str {
        "lsusb"
    }

    fn description(&self) -> &'static str {
        "List USB devices"
    }

    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let usb_dir = Path::new("/sys/bus/usb/devices");
        let entries = match fs::read_dir(usb_dir) {
            Ok(e) => e,
            Err(err) => {
                eprintln!("lsusb: /sys/bus/usb/devices: {}", err);
                return Ok(1);
            }
        };

        let mut devices = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            let busnum_str = fs::read_to_string(path.join("busnum")).unwrap_or_default();
            let devnum_str = fs::read_to_string(path.join("devnum")).unwrap_or_default();
            let id_vendor = fs::read_to_string(path.join("idVendor")).unwrap_or_default();
            let id_product = fs::read_to_string(path.join("idProduct")).unwrap_or_default();
            let product_name = fs::read_to_string(path.join("product")).unwrap_or_default();

            if busnum_str.is_empty() || devnum_str.is_empty() || id_vendor.is_empty() {
                continue;
            }

            let busnum: u32 = busnum_str.trim().parse().unwrap_or(0);
            let devnum: u32 = devnum_str.trim().parse().unwrap_or(0);
            let vendor = id_vendor.trim().to_string();
            let product = id_product.trim().to_string();
            let desc = product_name.trim().to_string();

            devices.push((busnum, devnum, vendor, product, desc));
        }

        devices.sort_by_key(|d| (d.0, d.1));
        for (bus, dev, vendor, product, desc) in devices {
            if desc.is_empty() {
                println!(
                    "Bus {:03} Device {:03}: ID {}:{}",
                    bus, dev, vendor, product
                );
            } else {
                println!(
                    "Bus {:03} Device {:03}: ID {}:{} {}",
                    bus, dev, vendor, product, desc
                );
            }
        }
        Ok(0)
    }
}
