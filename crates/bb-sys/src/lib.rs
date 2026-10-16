#![deny(unsafe_op_in_unsafe_fn)]

use std::ffi::CStr;
use std::mem::MaybeUninit;

pub mod fs {
    use super::*;

    pub fn stat(path: &CStr) -> std::io::Result<libc::stat> {
        let mut st = MaybeUninit::<libc::stat>::uninit();
        let res = unsafe { libc::stat(path.as_ptr(), st.as_mut_ptr()) };
        if res == 0 {
            Ok(unsafe { st.assume_init() })
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    pub fn lstat(path: &CStr) -> std::io::Result<libc::stat> {
        let mut st = MaybeUninit::<libc::stat>::uninit();
        let res = unsafe { libc::lstat(path.as_ptr(), st.as_mut_ptr()) };
        if res == 0 {
            Ok(unsafe { st.assume_init() })
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    pub fn mkdir(path: &CStr, mode: libc::mode_t) -> std::io::Result<()> {
        let res = unsafe { libc::mkdir(path.as_ptr(), mode) };
        if res == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    pub fn chmod(path: &CStr, mode: libc::mode_t) -> std::io::Result<()> {
        let res = unsafe { libc::chmod(path.as_ptr(), mode) };
        if res == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    pub fn chown(path: &CStr, uid: libc::uid_t, gid: libc::gid_t) -> std::io::Result<()> {
        let res = unsafe { libc::chown(path.as_ptr(), uid, gid) };
        if res == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    pub fn mknod(path: &CStr, mode: libc::mode_t, dev: libc::dev_t) -> std::io::Result<()> {
        let res = unsafe { libc::mknod(path.as_ptr(), mode, dev) };
        if res == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    pub fn umask(mask: libc::mode_t) -> libc::mode_t {
        unsafe { libc::umask(mask) }
    }

    pub fn utimensat(
        dirfd: libc::c_int,
        path: &CStr,
        times: &[libc::timespec; 2],
        flags: libc::c_int,
    ) -> std::io::Result<()> {
        let res = unsafe { libc::utimensat(dirfd, path.as_ptr(), times.as_ptr(), flags) };
        if res == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }
}

pub mod platform {
    use super::*;

    pub fn uname() -> std::io::Result<libc::utsname> {
        let mut uts = MaybeUninit::<libc::utsname>::uninit();
        let res = unsafe { libc::uname(uts.as_mut_ptr()) };
        if res == 0 {
            Ok(unsafe { uts.assume_init() })
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    pub fn sync() {
        unsafe { libc::sync() };
    }

    pub fn reboot(cmd: libc::c_int) -> std::io::Result<()> {
        let res = unsafe { libc::reboot(cmd) };
        if res == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }
}
