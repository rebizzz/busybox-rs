use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

pub struct SeedrngApplet;
impl Applet for SeedrngApplet {
    fn name(&self) -> &'static str {
        "seedrng"
    }
    fn description(&self) -> &'static str {
        "Seed the kernel random number generator"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.iter().any(|a| a.as_bytes().first() == Some(&b'-')) {
            eprintln!("seedrng: no options supported in this subset");
            return Ok(1);
        }

        let mut seed = [0u8; 512];
        let mut got = 0;
        while got < seed.len() {
            let r = unsafe {
                libc::getrandom(
                    seed[got..].as_mut_ptr() as *mut libc::c_void,
                    seed.len() - got,
                    0,
                )
            };
            if r < 0 {
                let e = std::io::Error::last_os_error();
                if e.raw_os_error() == Some(libc::EINTR) {
                    continue;
                }
                eprintln!("seedrng: getrandom: {}", e);
                return Ok(1);
            }
            got += r as usize;
        }

        const RNDADDENTROPY: libc::c_ulong = 0x4008_5203;
        use std::ffi::CString;
        let mut rc = 0;
        if let Ok(dev) = CString::new("/dev/urandom") {
            let fd = unsafe { libc::open(dev.as_ptr(), libc::O_WRONLY) };
            if fd < 0 {
                eprintln!(
                    "seedrng: cannot open /dev/urandom: {}",
                    std::io::Error::last_os_error()
                );
                return Ok(1);
            }

            let mut req = Vec::with_capacity(8 + 512);
            req.extend_from_slice(&256u32.to_ne_bytes());
            req.extend_from_slice(&512u32.to_ne_bytes());
            req.extend_from_slice(&seed);
            if unsafe { libc::ioctl(fd, RNDADDENTROPY, req.as_ptr()) } != 0 {
                eprintln!(
                    "seedrng: RNDADDENTROPY: {}",
                    std::io::Error::last_os_error()
                );
                rc = 1;
            }
            unsafe { libc::close(fd) };
        }
        Ok(rc)
    }
}
