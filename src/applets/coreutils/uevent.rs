use super::common::*;
use crate::core::Result;
use std::ffi::OsString;
use std::io::Write;

applet!(UeventApplet, "uevent", "Print kernel uevents", run_uevent);
fn run_uevent(args: &[OsString]) -> Result<i32> {
    let mut count: Option<u64> = None;
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        if b == b"-c" {
            i += 1;
            if i >= args.len() {
                eprintln!("uevent: -c needs an argument");
                return Ok(1);
            }
            match lossy(&args[i]).parse() {
                Ok(n) => count = Some(n),
                Err(_) => {
                    eprintln!("uevent: bad count");
                    return Ok(1);
                }
            }
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("uevent: invalid option '{}'", lossy(&args[i]));
            return Ok(1);
        }
        i += 1;
    }
    unsafe {
        let fd = libc::socket(
            libc::AF_NETLINK,
            libc::SOCK_DGRAM | libc::SOCK_CLOEXEC,
            libc::NETLINK_KOBJECT_UEVENT,
        );
        if fd < 0 {
            eprintln!("uevent: socket failed: {}", std::io::Error::last_os_error());
            return Ok(1);
        }
        let mut sa: libc::sockaddr_nl = std::mem::zeroed();
        sa.nl_family = libc::AF_NETLINK as _;
        sa.nl_groups = 1;
        if libc::bind(
            fd,
            &sa as *const _ as _,
            std::mem::size_of::<libc::sockaddr_nl>() as _,
        ) != 0
        {
            eprintln!("uevent: bind failed: {}", std::io::Error::last_os_error());
            libc::close(fd);
            return Ok(1);
        }

        let tv = libc::timeval {
            tv_sec: 15,
            tv_usec: 0,
        };
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_RCVTIMEO,
            &tv as *const _ as _,
            std::mem::size_of::<libc::timeval>() as _,
        );
        let mut out = wlock();
        let mut seen = 0u64;
        let mut buf = [0u8; 8192];
        loop {
            let n = libc::recv(fd, buf.as_mut_ptr() as _, buf.len(), 0);
            if n <= 0 {
                break;
            }
            let n = n as usize;

            let mut first = true;
            for part in buf[..n].split(|&c| c == 0) {
                if part.is_empty() {
                    continue;
                }
                if first {
                    let _ = writeln!(out, "--- {}", String::from_utf8_lossy(part));
                    first = false;
                } else {
                    let _ = writeln!(out, "{}", String::from_utf8_lossy(part));
                }
            }
            seen += 1;
            if count.is_some_and(|c| seen >= c) {
                break;
            }
        }
        libc::close(fd);
    }
    Ok(0)
}

