use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::io::{self};
use std::os::unix::ffi::OsStrExt;

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
